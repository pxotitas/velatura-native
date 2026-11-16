//! Unified Niri session adapter.
//!
//! Owns the event stream, reducer, output refresh worker and serialized action
//! queue behind one lifecycle and one consumer-facing event channel. This
//! adapter deliberately has no GTK/GLib/Tokio dependency.
use crate::niri::{NiriLineResult, NiriSignal, NiriState};
use crate::niri_actions::{NiriActionQueue, NiriActionResult};
use crate::niri_outputs::{NiriOutput, NiriOutputEvent, NiriOutputService};
use crate::niri_process::{NiriEventStream, NiriProcessEvent};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NiriSessionSnapshot {
    /// Monotonic revision assigned when this coherent snapshot is committed.
    pub revision: u64,
    pub niri: NiriState,
    /// None means the initial output query has not completed successfully.
    pub outputs: Option<Vec<NiriOutput>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NiriSessionEvent {
    SnapshotChanged(NiriSessionSnapshot),
    Signal(NiriSignal),
    Diagnostic(String),
    ProtocolError(String),
    ProcessExited { code: Option<i32>, success: bool },
    ProcessSpawnFailed(String),
    OutputsRefreshFailed(String),
    ActionFinished(NiriActionResult),
}

enum SessionCommand {
    Action(Vec<String>),
    RefreshOutputs,
    Stop,
}

/// A single owned Niri runtime. Drop requests shutdown and joins its worker.
pub struct NiriSession {
    commands: Option<Sender<SessionCommand>>,
    events: Option<Receiver<NiriSessionEvent>>,
    snapshot: Arc<RwLock<NiriSessionSnapshot>>,
    worker: Option<JoinHandle<()>>,
}

impl NiriSession {
    pub fn spawn() -> Self {
        Self::with_program("niri")
    }

    /// Constructor with an injectable executable, intended for deterministic
    /// integration tests as well as alternate Niri binary locations.
    pub fn with_program(program: impl Into<String>) -> Self {
        let program = program.into();
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let snapshot = Arc::new(RwLock::new(NiriSessionSnapshot::default()));
        let snapshot_for_worker = Arc::clone(&snapshot);
        let worker = thread::Builder::new()
            .name("velatura-niri-session".into())
            .spawn(move || run_session(program, command_rx, event_tx, snapshot_for_worker))
            .expect("failed to start Niri session worker");
        Self {
            commands: Some(command_tx),
            events: Some(event_rx),
            snapshot,
            worker: Some(worker),
        }
    }

    /// Queue an action. Arguments remain separate argv entries and execute FIFO.
    pub fn dispatch<I, S>(&self, args: I) -> Result<(), mpsc::SendError<()>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args: Vec<String> = args.into_iter().map(Into::into).collect();
        if args.is_empty() {
            return Ok(());
        }
        self.send(SessionCommand::Action(args))
    }

    /// Request a debounced, coalesced output inventory refresh.
    pub fn refresh_outputs(&self) -> Result<(), mpsc::SendError<()>> {
        self.send(SessionCommand::RefreshOutputs)
    }

    /// Create a lightweight handle for GDK/GTK observers and other adapters.
    pub fn refresh_handle(&self) -> NiriRefreshHandle {
        NiriRefreshHandle {
            commands: self
                .commands
                .as_ref()
                .expect("Niri session is stopped")
                .clone(),
        }
    }

    /// Receive the next state, diagnostic, process or action event.
    pub fn recv(&self) -> Result<NiriSessionEvent, mpsc::RecvError> {
        match &self.events {
            Some(events) => events.recv(),
            None => Err(mpsc::RecvError),
        }
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<NiriSessionEvent, RecvTimeoutError> {
        match &self.events {
            Some(events) => events.recv_timeout(timeout),
            None => Err(RecvTimeoutError::Disconnected),
        }
    }

    pub fn try_recv(&self) -> Result<NiriSessionEvent, mpsc::TryRecvError> {
        match &self.events {
            Some(events) => events.try_recv(),
            None => Err(mpsc::TryRecvError::Disconnected),
        }
    }

    /// Return the latest committed Niri and output state in one consistent value.
    pub fn snapshot(&self) -> NiriSessionSnapshot {
        self.snapshot
            .read()
            .expect("Niri snapshot lock poisoned")
            .clone()
    }

    /// Transfer the single event receiver to an adapter. `recv*` is unavailable
    /// afterwards; the session remains the owner of the workers and commands.
    pub fn take_events(&mut self) -> Option<Receiver<NiriSessionEvent>> {
        self.events.take()
    }

    fn send(&self, command: SessionCommand) -> Result<(), mpsc::SendError<()>> {
        self.commands
            .as_ref()
            .expect("Niri session is stopped")
            .send(command)
            .map_err(|_| mpsc::SendError(()))
    }

    pub fn stop(&mut self) {
        if let Some(commands) = self.commands.take() {
            let _ = commands.send(SessionCommand::Stop);
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for NiriSession {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_session(
    program: String,
    commands: Receiver<SessionCommand>,
    events: Sender<NiriSessionEvent>,
    published_snapshot: Arc<RwLock<NiriSessionSnapshot>>,
) {
    let (stream, process_events) = NiriEventStream::with_program(program.clone());
    let (outputs_service, output_events) = NiriOutputService::with_program(program.clone());
    let (actions, action_results) = NiriActionQueue::with_program(program);
    let mut snapshot = NiriSessionSnapshot::default();
    let mut running = true;

    while running {
        match commands.recv_timeout(POLL_INTERVAL) {
            Ok(SessionCommand::Action(args)) => {
                if actions.enqueue(args).is_err() {
                    let _ = events.send(NiriSessionEvent::Diagnostic(
                        "Niri action queue is unavailable".into(),
                    ));
                }
            }
            Ok(SessionCommand::RefreshOutputs) => {
                if outputs_service.request_refresh().is_err() {
                    let _ = events.send(NiriSessionEvent::Diagnostic(
                        "Niri output service is unavailable".into(),
                    ));
                }
            }
            Ok(SessionCommand::Stop) | Err(RecvTimeoutError::Disconnected) => running = false,
            Err(RecvTimeoutError::Timeout) => {}
        }

        for process_event in process_events.try_iter() {
            match process_event {
                NiriProcessEvent::StdoutLine(line) => match snapshot.niri.apply_line(&line) {
                    Ok(NiriLineResult::Event(signals)) => {
                        let topology_changed = signals.iter().any(|signal| {
                            matches!(signal, NiriSignal::EventReceived { kind, .. }
                                    if kind == "WorkspacesChanged")
                        });
                        for signal in signals {
                            if events.send(NiriSessionEvent::Signal(signal)).is_err() {
                                running = false;
                                break;
                            }
                        }
                        if topology_changed {
                            let _ = outputs_service.request_refresh();
                        }
                        publish_snapshot(&events, &published_snapshot, &mut snapshot, &mut running);
                    }
                    Ok(
                        NiriLineResult::HandshakeAccepted | NiriLineResult::HandshakeRejected(_),
                    ) => {
                        publish_snapshot(&events, &published_snapshot, &mut snapshot, &mut running);
                    }
                    Ok(NiriLineResult::Ignored) => {}
                    Err(error) => {
                        let _ = events.send(NiriSessionEvent::ProtocolError(error.to_string()));
                        publish_snapshot(&events, &published_snapshot, &mut snapshot, &mut running);
                    }
                },
                NiriProcessEvent::Stderr(message) => {
                    let _ = events.send(NiriSessionEvent::Diagnostic(message));
                }
                NiriProcessEvent::Exited { code, success } => {
                    snapshot.niri.connected = false;
                    if !success {
                        if let Some(code) = code {
                            snapshot.niri.last_error = format!("Niri event stream exited ({code})");
                        }
                    }
                    publish_snapshot(&events, &published_snapshot, &mut snapshot, &mut running);
                    let _ = events.send(NiriSessionEvent::ProcessExited { code, success });
                }
                NiriProcessEvent::SpawnFailed(error) => {
                    snapshot.niri.connected = false;
                    publish_snapshot(&events, &published_snapshot, &mut snapshot, &mut running);
                    let _ = events.send(NiriSessionEvent::ProcessSpawnFailed(error));
                }
            }
        }

        for output_event in output_events.try_iter() {
            match output_event {
                NiriOutputEvent::Refreshed(outputs) => {
                    snapshot.outputs = Some(outputs);
                    publish_snapshot(&events, &published_snapshot, &mut snapshot, &mut running);
                }
                NiriOutputEvent::Failed(error) => {
                    let _ = events.send(NiriSessionEvent::OutputsRefreshFailed(error));
                }
            }
        }

        for result in action_results.try_iter() {
            if events
                .send(NiriSessionEvent::ActionFinished(result))
                .is_err()
            {
                running = false;
                break;
            }
        }
    }

    // Drop order is intentional: stop child/query workers before the action
    // queue is joined. Each worker owns and reaps its own subprocesses.
    drop(stream);
    drop(outputs_service);
    drop(actions);
}

fn publish_snapshot(
    events: &Sender<NiriSessionEvent>,
    published_snapshot: &RwLock<NiriSessionSnapshot>,
    snapshot: &mut NiriSessionSnapshot,
    running: &mut bool,
) {
    snapshot.revision = snapshot.revision.saturating_add(1);
    *published_snapshot
        .write()
        .expect("Niri snapshot lock poisoned") = snapshot.clone();
    if events
        .send(NiriSessionEvent::SnapshotChanged(snapshot.clone()))
        .is_err()
    {
        *running = false;
    }
}

/// Cloneable command handle for adapters that only need to request output refreshes.
#[derive(Clone)]
pub struct NiriRefreshHandle {
    commands: Sender<SessionCommand>,
}

impl NiriRefreshHandle {
    pub fn refresh_outputs(&self) -> Result<(), mpsc::SendError<()>> {
        self.commands
            .send(SessionCommand::RefreshOutputs)
            .map_err(|_| mpsc::SendError(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    struct FakeNiri(std::path::PathBuf);
    impl FakeNiri {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("velatura-session-{nonce}"));
            fs::create_dir_all(&dir).unwrap();
            let path = dir.join("niri");
            let script = r#"#!/bin/sh
if [ "$1" = "msg" ] && [ "$2" = "--json" ] && [ "$3" = "event-stream" ]; then
  echo '{"Ok":null}'
  echo '{"WorkspacesChanged":{"workspaces":[{"id":1,"name":"1","is_focused":true}]}}'
  exec sleep 30
fi
if [ "$1" = "msg" ] && [ "$2" = "--json" ] && [ "$3" = "outputs" ]; then
  echo '{"eDP-1":{"make":"Apple","model":"Color LCD","modes":[{"width":1440,"height":900,"refresh_rate":60000}],"current_mode":0,"logical":{"x":0,"y":0,"width":1440,"height":900,"scale":1.0,"transform":"Normal"}}}'
  exit 0
fi
if [ "$1" = "msg" ] && [ "$2" = "action" ]; then
  echo "$3"
  exit 0
fi
exit 2
"#;
            fs::write(&path, script).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o700);
            fs::set_permissions(&path, permissions).unwrap();
            Self(path)
        }
    }
    impl Drop for FakeNiri {
        fn drop(&mut self) {
            if let Some(parent) = self.0.parent() {
                let _ = fs::remove_dir_all(parent);
            }
        }
    }

    #[test]
    fn integrates_handshake_state_outputs_and_actions() {
        let fake = FakeNiri::new();
        let session = NiriSession::with_program(fake.0.to_string_lossy());
        let mut connected = false;
        let mut workspace = false;
        let mut output_snapshots = 0;
        let mut previous_revision = 0;
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline && !(connected && workspace && output_snapshots >= 2) {
            match session.recv_timeout(Duration::from_millis(250)) {
                Ok(NiriSessionEvent::SnapshotChanged(snapshot)) => {
                    assert!(snapshot.revision > previous_revision);
                    previous_revision = snapshot.revision;
                    connected |= snapshot.niri.connected;
                    workspace |= snapshot.niri.workspaces.len() == 1;
                    if snapshot
                        .outputs
                        .as_ref()
                        .is_some_and(|items| items.len() == 1)
                    {
                        output_snapshots += 1;
                    }
                }
                Ok(NiriSessionEvent::Signal(NiriSignal::EventReceived { kind, .. })) => {
                    workspace |= kind == "WorkspacesChanged";
                }
                Ok(other) => panic!("unexpected session event: {other:?}"),
                Err(RecvTimeoutError::Timeout) => {}
                Err(error) => panic!("session channel closed: {error}"),
            }
        }
        assert!(connected && workspace && output_snapshots >= 2);
        let current = session.snapshot();
        assert_eq!(current.revision, previous_revision);
        assert!(current.niri.connected);
        assert_eq!(current.niri.workspaces.len(), 1);
        assert_eq!(current.outputs.as_ref().map(Vec::len), Some(1));
        session
            .dispatch(["test-action", "arg with spaces"])
            .unwrap();
        loop {
            match session.recv_timeout(Duration::from_secs(2)).unwrap() {
                NiriSessionEvent::ActionFinished(result) => {
                    assert!(result.succeeded());
                    assert_eq!(result.args, vec!["test-action", "arg with spaces"]);
                    assert!(result.stdout.contains("test-action"));
                    break;
                }
                NiriSessionEvent::SnapshotChanged(_) | NiriSessionEvent::Signal(_) => {}
                other => panic!("unexpected event while waiting for action: {other:?}"),
            }
        }
    }

    #[test]
    fn explicit_stop_joins_the_session_worker() {
        let fake = FakeNiri::new();
        let mut session = NiriSession::with_program(fake.0.to_string_lossy());
        let started = Instant::now();
        session.stop();
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn initial_snapshot_is_empty_and_spawn_disconnection_is_explicit() {
        let session = NiriSession::with_program("/velatura-test/missing-niri-binary");
        let initial = session.snapshot();
        assert_eq!(initial.revision, 0);
        assert!(!initial.niri.connected);
        assert!(initial.niri.workspaces.is_empty());
        assert!(initial.outputs.is_none());

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut reported = false;
        while Instant::now() < deadline {
            match session.recv_timeout(Duration::from_millis(100)) {
                Ok(NiriSessionEvent::ProcessSpawnFailed(_)) => {
                    reported = true;
                    break;
                }
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => {}
                Err(error) => panic!("session channel closed unexpectedly: {error}"),
            }
        }
        assert!(reported);
        let disconnected = session.snapshot();
        assert!(!disconnected.niri.connected);
        assert!(disconnected.outputs.is_none());
    }
}
