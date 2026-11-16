//! Read-only Niri output discovery and refresh scheduling.
//! Normalized fields mirror the active QuickShell monitors.py helper.
//! Refresh requests debounce for 120ms and coalesce while a query is running.

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const DEBOUNCE: Duration = Duration::from_millis(120);
const QUERY_TIMEOUT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(20);

#[derive(Debug, Clone, PartialEq)]
pub struct NiriOutput {
    pub name: String,
    pub description: String,
    pub make: String,
    pub model: String,
    pub serial: String,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub refresh_rate: f64,
    pub mode: String,
    pub scale: f64,
    pub transform: String,
    pub disabled: bool,
    pub vrr: bool,
    pub vrr_supported: bool,
    pub resolutions: Vec<NiriResolution>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NiriResolution {
    pub width: i64,
    pub height: i64,
    pub refreshes: Vec<f64>,
}

/// Focused output wins even when disabled; otherwise use the first enabled
/// output in the original compositor-provided order.
pub fn primary_name(outputs: &[NiriOutput], focused_output: &str) -> String {
    if outputs.iter().any(|output| output.name == focused_output) {
        return focused_output.to_owned();
    }
    outputs
        .iter()
        .find(|output| !output.disabled)
        .map(|output| output.name.clone())
        .unwrap_or_default()
}

/// Internal panel heuristic copied from MonitorService.qml.
pub fn internal_name(outputs: &[NiriOutput]) -> String {
    outputs
        .iter()
        .find(|output| {
            output.name.starts_with("eDP")
                || output.name.starts_with("LVDS")
                || output.name.starts_with("DSI")
        })
        .map(|output| output.name.clone())
        .unwrap_or_default()
}

pub fn monitor_for<'a>(outputs: &'a [NiriOutput], description: &str) -> Option<&'a NiriOutput> {
    outputs
        .iter()
        .find(|output| output.description == description || output.name == description)
}

pub fn description_for<'a>(outputs: &'a [NiriOutput], name: &str) -> &'a str {
    outputs
        .iter()
        .find(|output| output.name == name)
        .map(|output| output.description.as_str())
        .unwrap_or("")
}

pub fn is_primary(outputs: &[NiriOutput], focused_output: &str, screen_name: &str) -> bool {
    !screen_name.is_empty() && screen_name == primary_name(outputs, focused_output)
}

#[derive(Debug, Clone, PartialEq)]
pub enum NiriOutputEvent {
    Refreshed(Vec<NiriOutput>),
    Failed(String),
}

/// Apply one Niri JSONL event and request an output refresh for workspace
/// topology changes. UI hotplug/geometry observers call request_refresh().
pub fn apply_niri_line_with_output_refresh(
    state: &mut crate::niri::NiriState,
    line: &str,
    outputs: &NiriOutputService,
) -> Result<crate::niri::NiriLineResult, serde_json::Error> {
    let result = state.apply_line(line)?;
    if let crate::niri::NiriLineResult::Event(signals) = &result {
        let workspace_topology_changed = signals.iter().any(|signal| {
            matches!(
                signal,
                crate::niri::NiriSignal::EventReceived { kind, .. }
                    if kind == "WorkspacesChanged"
            )
        });
        if workspace_topology_changed {
            let _ = outputs.request_refresh();
        }
    }
    Ok(result)
}

enum WorkerMessage {
    Refresh,
    Stop,
    Finished(Result<Vec<NiriOutput>, String>),
}

/// Owns a read-only output query worker with debounce and refresh coalescing.
pub struct NiriOutputService {
    sender: Option<Sender<WorkerMessage>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl NiriOutputService {
    pub fn spawn() -> (Self, Receiver<NiriOutputEvent>) {
        Self::with_program("niri")
    }

    pub fn with_program(program: impl Into<String>) -> (Self, Receiver<NiriOutputEvent>) {
        let (sender, receiver) = mpsc::channel();
        let (event_sender, events) = mpsc::channel();
        let program = program.into();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker_sender = sender.clone();
        let worker = thread::Builder::new()
            .name("velatura-niri-outputs".into())
            .spawn(move || {
                refresh_loop(program, receiver, worker_sender, event_sender, worker_stop)
            })
            .expect("failed to start Niri output worker");
        (
            Self {
                sender: Some(sender),
                stop,
                worker: Some(worker),
            },
            events,
        )
    }

    pub fn request_refresh(&self) -> Result<(), mpsc::SendError<()>> {
        self.sender
            .as_ref()
            .expect("output service stopped")
            .send(WorkerMessage::Refresh)
            .map_err(|_| mpsc::SendError(()))
    }
}

impl Drop for NiriOutputService {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(WorkerMessage::Stop);
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn refresh_loop(
    program: String,
    receiver: Receiver<WorkerMessage>,
    sender: Sender<WorkerMessage>,
    events: Sender<NiriOutputEvent>,
    stop: Arc<AtomicBool>,
) {
    let mut in_flight = true;
    let mut pending = false;
    let mut deadline: Option<Instant> = None;
    launch_query(&program, sender.clone(), Arc::clone(&stop)); // initial load is immediate

    loop {
        let timeout = deadline
            .map(|at| at.saturating_duration_since(Instant::now()))
            .unwrap_or(Duration::from_secs(24 * 60 * 60));
        match receiver.recv_timeout(timeout) {
            Ok(WorkerMessage::Refresh) => deadline = Some(Instant::now() + DEBOUNCE),
            Ok(WorkerMessage::Stop) | Err(RecvTimeoutError::Disconnected) => break,
            Ok(WorkerMessage::Finished(result)) => {
                in_flight = false;
                let event = match result {
                    Ok(outputs) => NiriOutputEvent::Refreshed(outputs),
                    Err(error) => NiriOutputEvent::Failed(error),
                };
                if events.send(event).is_err() {
                    return;
                }
                if pending {
                    pending = false;
                    deadline = None;
                    launch_query(&program, sender.clone(), Arc::clone(&stop));
                    in_flight = true;
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if deadline.is_some_and(|at| Instant::now() >= at) {
                    deadline = None;
                    if in_flight {
                        pending = true;
                    } else {
                        launch_query(&program, sender.clone(), Arc::clone(&stop));
                        in_flight = true;
                    }
                }
            }
        }
    }
}

fn launch_query(program: &str, sender: Sender<WorkerMessage>, stop: Arc<AtomicBool>) {
    let program = program.to_owned();
    let _ = thread::Builder::new()
        .name("velatura-niri-outputs-query".into())
        .spawn(move || {
            let result = query_outputs(&program, &stop);
            let _ = sender.send(WorkerMessage::Finished(result));
        });
}

fn query_outputs(program: &str, stop: &AtomicBool) -> Result<Vec<NiriOutput>, String> {
    let mut child = Command::new(program)
        .args(["msg", "--json", "outputs"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot query Niri outputs: {error}"))?;

    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let out_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout.read_to_end(&mut bytes);
        (bytes, result)
    });
    let err_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stderr.read_to_end(&mut bytes);
        (bytes, result)
    });

    let started = Instant::now();
    let status = loop {
        if stop.load(Ordering::Acquire) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = out_reader.join();
            let _ = err_reader.join();
            return Err("Niri output query cancelled".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < QUERY_TIMEOUT => thread::sleep(POLL),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = out_reader.join();
                let _ = err_reader.join();
                return Err("cannot query Niri outputs: timed out after 5 seconds".into());
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = out_reader.join();
                let _ = err_reader.join();
                return Err(format!("cannot query Niri outputs: {error}"));
            }
        }
    };

    let (stdout, out_result) = out_reader
        .join()
        .map_err(|_| "Niri outputs stdout reader panicked".to_owned())?;
    let (stderr, err_result) = err_reader
        .join()
        .map_err(|_| "Niri outputs stderr reader panicked".to_owned())?;
    out_result.map_err(|error| format!("cannot read Niri outputs: {error}"))?;
    err_result.map_err(|error| format!("cannot read Niri outputs: {error}"))?;
    if !status.success() {
        let diagnostic = String::from_utf8_lossy(&stderr).trim().to_owned();
        return Err(if diagnostic.is_empty() {
            format!("niri msg outputs exited ({})", status.code().unwrap_or(-1))
        } else {
            diagnostic
        });
    }
    parse_outputs(&stdout).map_err(|error| error.to_string())
}

pub fn parse_outputs(bytes: &[u8]) -> Result<Vec<NiriOutput>, OutputParseError> {
    let text = std::str::from_utf8(bytes).map_err(|_| OutputParseError::InvalidUtf8)?;
    let root: OrderedObject = serde_json::from_str(text)
        .map_err(|error| OutputParseError::InvalidJson(error.to_string()))?;
    if !root.is_object {
        return Err(OutputParseError::UnexpectedResponse);
    }
    if let Some((_, error)) = root.entries.iter().find(|(key, _)| key == "Err") {
        return Err(OutputParseError::Compositor(error.to_string()));
    }

    // The real CLI returns a direct object. Preserve textual key order because
    // MonitorService falls back to the first enabled output when focus is empty.
    let entries = if let Some((_, ok)) = root.entries.iter().find(|(key, _)| key == "Ok") {
        let object = ok.as_object().ok_or(OutputParseError::UnexpectedResponse)?;
        if let Some(outputs) = object.get("Outputs") {
            outputs
                .as_object()
                .ok_or(OutputParseError::UnexpectedResponse)?
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        } else {
            object
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        }
    } else if let Some((_, outputs)) = root.entries.iter().find(|(key, _)| key == "Outputs") {
        outputs
            .as_object()
            .ok_or(OutputParseError::UnexpectedResponse)?
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    } else {
        root.entries
    };

    entries
        .iter()
        .filter_map(|(name, value)| value.as_object().map(|object| parse_output(name, object)))
        .collect()
}

struct OrderedObject {
    is_object: bool,
    entries: Vec<(String, Value)>,
}

impl<'de> Deserialize<'de> for OrderedObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderedObjectVisitor;
        impl<'de> Visitor<'de> for OrderedObjectVisitor {
            type Value = OrderedObject;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a JSON object containing Niri outputs")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, Value>()? {
                    entries.push(entry);
                }
                Ok(OrderedObject {
                    is_object: true,
                    entries,
                })
            }

            fn visit_seq<A>(self, _sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Ok(OrderedObject {
                    is_object: false,
                    entries: Vec::new(),
                })
            }
        }
        deserializer.deserialize_any(OrderedObjectVisitor)
    }
}

fn parse_output(name: &str, output: &Map<String, Value>) -> Result<NiriOutput, OutputParseError> {
    let modes = output
        .get("modes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let current = match output.get("current_mode") {
        Some(Value::Number(index)) => index
            .as_u64()
            .and_then(|i| modes.get(i as usize))
            .and_then(Value::as_object),
        Some(Value::Object(mode)) => Some(mode),
        _ => None,
    };
    let make = string_field(output, "make");
    let model = string_field(output, "model");
    let serial = string_field(output, "serial");
    let description = [make.as_str(), model.as_str(), serial.as_str()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let description = if description.is_empty() {
        name.to_owned()
    } else {
        description
    };
    let logical = output.get("logical").and_then(Value::as_object);
    let physical_w = current.and_then(|m| int_field(m, "width")).unwrap_or(0);
    let physical_h = current.and_then(|m| int_field(m, "height")).unwrap_or(0);
    let width = logical.and_then(|v| int_field(v, "width")).unwrap_or(0);
    let height = logical.and_then(|v| int_field(v, "height")).unwrap_or(0);
    let refresh_rate = current
        .and_then(|m| m.get("refresh_rate"))
        .and_then(Value::as_f64)
        .map(refresh_hz)
        .unwrap_or(0.0);

    let mut resolutions: Vec<NiriResolution> = Vec::new();
    for candidate in &modes {
        let Some(candidate) = candidate.as_object() else {
            continue;
        };
        let w = int_field(candidate, "width").unwrap_or(0);
        let h = int_field(candidate, "height").unwrap_or(0);
        if w == 0 || h == 0 {
            continue;
        }
        let rate = candidate
            .get("refresh_rate")
            .and_then(Value::as_f64)
            .map(refresh_hz)
            .unwrap_or(0.0);
        let index = resolutions
            .iter()
            .position(|r| r.width == w && r.height == h);
        let resolution = if let Some(index) = index {
            &mut resolutions[index]
        } else {
            resolutions.push(NiriResolution {
                width: w,
                height: h,
                refreshes: Vec::new(),
            });
            resolutions.last_mut().expect("just inserted")
        };
        if rate != 0.0 {
            resolution.refreshes.push(rate);
        }
    }

    let mut mode = if physical_w != 0 && physical_h != 0 {
        format!("{physical_w}x{physical_h}")
    } else {
        "off".to_owned()
    };
    if refresh_rate != 0.0 {
        mode.push_str(&format!("@{refresh_rate:.3}"));
    }

    Ok(NiriOutput {
        name: name.to_owned(),
        description,
        make,
        model,
        serial,
        x: logical.and_then(|v| int_field(v, "x")).unwrap_or(0),
        y: logical.and_then(|v| int_field(v, "y")).unwrap_or(0),
        width,
        height,
        refresh_rate,
        mode,
        scale: logical
            .and_then(|v| v.get("scale"))
            .and_then(Value::as_f64)
            .unwrap_or(1.0),
        transform: logical
            .and_then(|v| v.get("transform"))
            .and_then(Value::as_str)
            .unwrap_or("Normal")
            .to_owned(),
        disabled: output.get("logical").is_none_or(Value::is_null),
        vrr: output
            .get("vrr_enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        vrr_supported: output
            .get("vrr_supported")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        resolutions,
    })
}

fn string_field(object: &Map<String, Value>, key: &str) -> String {
    object
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}
fn int_field(object: &Map<String, Value>, key: &str) -> Option<i64> {
    object.get(key).and_then(Value::as_i64)
}
fn refresh_hz(value: f64) -> f64 {
    (value / 1000.0 * 1000.0).round() / 1000.0
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputParseError {
    InvalidUtf8,
    InvalidJson(String),
    Compositor(String),
    UnexpectedResponse,
}
impl std::fmt::Display for OutputParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUtf8 => write!(f, "Niri outputs response is not UTF-8"),
            Self::InvalidJson(e) => write!(f, "cannot parse Niri outputs: {e}"),
            Self::Compositor(e) => write!(f, "Niri outputs rejected: {e}"),
            Self::UnexpectedResponse => write!(f, "unexpected response from niri msg outputs"),
        }
    }
}
impl std::error::Error for OutputParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
      "eDP-1":{"make":"Apple Computer Inc","model":"Color LCD","serial":null,
      "modes":[{"width":1440,"height":900,"refresh_rate":59996},
      {"width":1280,"height":800,"refresh_rate":60000},
      {"width":1280,"height":800,"refresh_rate":50000}],
      "current_mode":0,"vrr_supported":false,"vrr_enabled":false,
      "logical":{"x":0,"y":0,"width":1440,"height":900,"scale":1.0,"transform":"Normal"}}
    }"#;

    #[test]
    fn normalizes_like_the_original_python_helper() {
        let outputs = parse_outputs(FIXTURE.as_bytes()).unwrap();
        assert_eq!(outputs.len(), 1);
        let output = &outputs[0];
        assert_eq!(output.name, "eDP-1");
        assert_eq!(output.description, "Apple Computer Inc Color LCD");
        assert_eq!(output.mode, "1440x900@59.996");
        assert_eq!(output.refresh_rate, 59.996);
        assert_eq!(output.width, 1440);
        assert!(!output.disabled);
        assert_eq!(output.resolutions.len(), 2);
        assert_eq!(output.resolutions[1].refreshes, vec![60.0, 50.0]);
    }

    #[test]
    fn handles_wrapped_responses_and_object_current_mode() {
        let wrapped = br#"{"Ok":{"Outputs":{"DP-1":{"modes":[{"width":1920,"height":1080,"refresh_rate":144000}],"current_mode":{"width":1920,"height":1080,"refresh_rate":144000},"logical":null}}}}"#;
        let outputs = parse_outputs(wrapped).unwrap();
        assert_eq!(outputs[0].mode, "1920x1080@144.000");
        assert!(outputs[0].disabled);
        assert_eq!(outputs[0].transform, "Normal");
    }

    #[test]
    fn preserves_output_order_for_primary_fallback() {
        let payload = br#"{
            "DP-2":{"logical":{"width":1920,"height":1080}},
            "eDP-1":{"logical":{"width":1440,"height":900}}
        }"#;
        let outputs = parse_outputs(payload).unwrap();
        assert_eq!(
            outputs
                .iter()
                .map(|output| output.name.as_str())
                .collect::<Vec<_>>(),
            vec!["DP-2", "eDP-1"]
        );
    }

    #[test]
    fn derives_primary_internal_and_description_helpers_like_original() {
        let mut outputs = parse_outputs(FIXTURE.as_bytes()).unwrap();
        outputs[0].disabled = true;
        assert_eq!(primary_name(&outputs, "eDP-1"), "eDP-1");
        assert_eq!(primary_name(&outputs, "missing"), "");
        assert_eq!(internal_name(&outputs), "eDP-1");
        assert_eq!(
            description_for(&outputs, "eDP-1"),
            "Apple Computer Inc Color LCD"
        );
        assert_eq!(
            monitor_for(&outputs, "Apple Computer Inc Color LCD")
                .unwrap()
                .name,
            "eDP-1"
        );
        assert!(is_primary(&outputs, "eDP-1", "eDP-1"));
        assert!(!is_primary(&outputs, "missing", "eDP-1"));
    }

    #[test]
    fn workspace_change_requests_a_debounced_output_refresh() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("velatura-workspace-outputs-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        let program = dir.join("niri");
        fs::write(&program, "#!/bin/sh\nsleep 0.25\nprintf '%s\\n' '{\"DP-1\":{\"logical\":{\"width\":100,\"height\":100}}}'\n").unwrap();
        let mut permissions = fs::metadata(&program).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&program, permissions).unwrap();

        let (service, events) = NiriOutputService::with_program(program.to_string_lossy());
        let mut state = crate::niri::NiriState::default();
        assert!(matches!(
            events.recv_timeout(Duration::from_secs(2)).unwrap(),
            NiriOutputEvent::Refreshed(_)
        ));
        let result = apply_niri_line_with_output_refresh(
            &mut state,
            r#"{"WorkspacesChanged":{"workspaces":[]}}"#,
            &service,
        )
        .unwrap();
        assert!(matches!(result, crate::niri::NiriLineResult::Event(_)));
        assert!(matches!(
            events.recv_timeout(Duration::from_secs(2)).unwrap(),
            NiriOutputEvent::Refreshed(outputs) if outputs.len() == 1
        ));
        drop(service);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn reports_protocol_and_shape_errors() {
        assert!(matches!(
            parse_outputs(br#"{"Err":{"PermissionDenied":null}}"#),
            Err(OutputParseError::Compositor(_))
        ));
        assert_eq!(
            parse_outputs(br#"[]"#),
            Err(OutputParseError::UnexpectedResponse)
        );
        assert!(matches!(
            parse_outputs(b"{invalid"),
            Err(OutputParseError::InvalidJson(_))
        ));
    }

    #[test]
    fn dropping_service_cancels_a_running_output_query() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("velatura-output-stop-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        let program = dir.join("niri");
        fs::write(&program, "#!/bin/sh\nexec sleep 30\n").unwrap();
        let mut permissions = fs::metadata(&program).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&program, permissions).unwrap();

        let (service, _events) = NiriOutputService::with_program(program.to_string_lossy());
        thread::sleep(Duration::from_millis(80));
        let started = Instant::now();
        drop(service);
        assert!(started.elapsed() < Duration::from_secs(1));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn refresh_service_initializes_and_coalesces_pending_refresh() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("velatura-outputs-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        let program = dir.join("niri");
        fs::write(&program, "#!/bin/sh\nsleep 0.25\nprintf '{\"DP-1\":{\"logical\":{\"width\":100,\"height\":100}}}'\n").unwrap();
        let mut permissions = fs::metadata(&program).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&program, permissions).unwrap();
        let (service, events) = NiriOutputService::with_program(program.to_string_lossy());
        service.request_refresh().unwrap();
        service.request_refresh().unwrap();
        assert!(
            matches!(events.recv_timeout(Duration::from_secs(2)).unwrap(),
            NiriOutputEvent::Refreshed(outputs) if outputs.len() == 1)
        );
        assert!(
            matches!(events.recv_timeout(Duration::from_secs(2)).unwrap(),
            NiriOutputEvent::Refreshed(outputs) if outputs.len() == 1)
        );
        assert!(events.recv_timeout(Duration::from_millis(250)).is_err());
        drop(service);
        let _ = fs::remove_dir_all(dir);
    }
}
