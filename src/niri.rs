//! Niri event-stream state reducer.
//!
//! Semantics in this module are ported from the active
//! ~/.config/quickshell/services/NiriService.qml. Keep this reducer pure:
//! it does not spawn processes, reconnect sockets, or perform compositor I/O.

use serde_json::{Map, Value};

const INVALID_JSON_ERROR: &str = "Invalid JSON from Niri event stream";

/// The state currently projected by the original NiriService singleton.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NiriState {
    pub workspaces: Vec<Value>,
    pub windows: Vec<Value>,
    /// Initially absent; after the first workspace update the original uses 0
    /// when no focused workspace can be found.
    pub focused_workspace_id: Option<Value>,
    pub active_workspace_id: Option<Value>,
    pub focused_window: Option<Value>,
    pub connected: bool,
    /// Mirrors the original string property, including its intentionally
    /// persistent value after ordinary events and disconnects.
    pub last_error: String,
}

/// Signals emitted by the original service while reducing one input line.
#[derive(Debug, Clone, PartialEq)]
pub enum NiriSignal {
    /// Forward every valid compositor event, including event kinds this
    /// reducer does not yet project into state.
    EventReceived { kind: String, data: Value },
    /// Emitted only when WindowOpenedOrChanged introduces a previously unseen ID.
    WindowOpened(Value),
    /// Emitted for every WindowClosed event, even if the ID was not present.
    WindowClosed(Value),
}

/// Result of consuming one newline-delimited event-stream message.
#[derive(Debug, Clone, PartialEq)]
pub enum NiriLineResult {
    Ignored,
    HandshakeAccepted,
    HandshakeRejected(String),
    Event(Vec<NiriSignal>),
}

impl NiriState {
    /// Consume one JSONL line, preserving the active QML adapter's observable
    /// state transitions. Invalid JSON updates last_error but leaves all
    /// other state untouched.
    pub fn apply_line(&mut self, line: &str) -> Result<NiriLineResult, serde_json::Error> {
        if line.trim().is_empty() {
            return Ok(NiriLineResult::Ignored);
        }

        let message: Value = match serde_json::from_str(line) {
            Ok(message) => message,
            Err(error) => {
                self.last_error = INVALID_JSON_ERROR.to_owned();
                return Err(error);
            }
        };

        let Some(object) = message.as_object() else {
            return Ok(NiriLineResult::Ignored);
        };

        let ok = object.get("Ok").is_some_and(js_truthy);
        let err = object.get("Err").filter(|value| js_truthy(value));
        if ok || err.is_some() {
            if let Some(error) = err {
                let serialized = error.to_string();
                self.connected = false;
                self.last_error = serialized.clone();
                return Ok(NiriLineResult::HandshakeRejected(serialized));
            }
            self.connected = true;
            self.last_error.clear();
            return Ok(NiriLineResult::HandshakeAccepted);
        }

        let Some((kind, data)) = object.iter().next() else {
            return Ok(NiriLineResult::Ignored);
        };
        let kind = kind.clone();
        let data = if data.is_null() {
            Value::Object(Map::new())
        } else {
            data.clone()
        };
        let mut signals = Vec::with_capacity(2);

        match kind.as_str() {
            "WorkspacesChanged" => {
                self.workspaces = data
                    .get("workspaces")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                self.update_workspace_focus();
            }
            "WindowsChanged" => {
                self.windows = data
                    .get("windows")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                self.update_focused_window();
            }
            "WindowOpenedOrChanged" => {
                if let Some(window) = data.get("window").filter(|value| !value.is_null()) {
                    let incoming_focused = window.get("is_focused").is_some_and(js_truthy);
                    if incoming_focused {
                        for existing in &mut self.windows {
                            if let Some(object) = existing.as_object_mut() {
                                object.insert("is_focused".into(), Value::Bool(false));
                            }
                        }
                    }
                    let incoming_id = window.get("id");
                    let index = self
                        .windows
                        .iter()
                        .position(|existing| existing.get("id") == incoming_id);
                    if let Some(index) = index {
                        self.windows[index] = window.clone();
                    } else {
                        self.windows.push(window.clone());
                        signals.push(NiriSignal::WindowOpened(window.clone()));
                    }
                    self.update_focused_window();
                }
            }
            "WindowClosed" => {
                let id = data.get("id").cloned().unwrap_or(Value::Null);
                self.windows.retain(|window| window.get("id") != Some(&id));
                self.update_focused_window();
                signals.push(NiriSignal::WindowClosed(id));
            }
            "WindowFocusChanged" => {
                let id = data.get("id");
                for window in &mut self.windows {
                    let focused = window.get("id") == id;
                    if let Some(object) = window.as_object_mut() {
                        object.insert("is_focused".into(), Value::Bool(focused));
                    }
                }
                self.update_focused_window();
            }
            "WorkspaceActivated" => {
                let id = data.get("id");
                if let Some(activated) = self
                    .workspaces
                    .iter()
                    .find(|workspace| workspace.get("id") == id)
                    .cloned()
                {
                    let output = activated.get("output");
                    let focused = data.get("focused").is_some_and(js_truthy);
                    for workspace in &mut self.workspaces {
                        let same_output = workspace.get("output") == output;
                        let workspace_id_matches = workspace.get("id") == id;
                        if let Some(object) = workspace.as_object_mut() {
                            if same_output {
                                object
                                    .insert("is_active".into(), Value::Bool(workspace_id_matches));
                            }
                            if focused {
                                object
                                    .insert("is_focused".into(), Value::Bool(workspace_id_matches));
                            }
                        }
                    }
                    self.update_workspace_focus();
                }
            }
            "WindowUrgencyChanged" => {
                let id = data.get("id");
                let urgent = data.get("urgent").is_some_and(js_truthy);
                for window in &mut self.windows {
                    if window.get("id") == id
                        && let Some(object) = window.as_object_mut()
                    {
                        object.insert("is_urgent".into(), Value::Bool(urgent));
                    }
                }
            }
            "WorkspaceUrgencyChanged" => {
                let id = data.get("id");
                let urgent = data.get("urgent").is_some_and(js_truthy);
                for workspace in &mut self.workspaces {
                    if workspace.get("id") == id
                        && let Some(object) = workspace.as_object_mut()
                    {
                        object.insert("is_urgent".into(), Value::Bool(urgent));
                    }
                }
            }
            "WorkspaceActiveWindowChanged" => {
                let workspace_id = data.get("workspace_id");
                let active_window_id = data.get("active_window_id").cloned().unwrap_or(Value::Null);
                for workspace in &mut self.workspaces {
                    if workspace.get("id") == workspace_id
                        && let Some(object) = workspace.as_object_mut()
                    {
                        object.insert("active_window_id".into(), active_window_id.clone());
                    }
                }
            }
            "WindowLayoutsChanged" => {
                if let Some(changes) = data.get("changes").and_then(Value::as_array) {
                    for pair in changes {
                        let Some(pair) = pair.as_array() else {
                            continue;
                        };
                        if pair.len() < 2 {
                            continue;
                        }
                        let id = &pair[0];
                        let layout = pair[1].clone();
                        for window in &mut self.windows {
                            if window.get("id") == Some(id)
                                && let Some(object) = window.as_object_mut()
                            {
                                object.insert("layout".into(), layout.clone());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        self.connected = true;
        signals.push(NiriSignal::EventReceived { kind, data });
        Ok(NiriLineResult::Event(signals))
    }

    fn update_workspace_focus(&mut self) {
        let focused = self
            .workspaces
            .iter()
            .find(|workspace| workspace.get("is_focused").is_some_and(js_truthy));
        let active = self.workspaces.iter().find(|workspace| {
            workspace.get("is_active").is_some_and(js_truthy)
                && workspace.get("output") == focused.and_then(|item| item.get("output"))
        });
        let focused_id = focused.and_then(|workspace| workspace.get("id"));
        let active_id = active.and_then(|workspace| workspace.get("id"));
        self.focused_workspace_id = Some(focused_id.cloned().unwrap_or(Value::from(0)));
        self.active_workspace_id =
            Some(active_id.or(focused_id).cloned().unwrap_or(Value::from(0)));
    }

    fn update_focused_window(&mut self) {
        self.focused_window = self
            .windows
            .iter()
            .find(|window| window.get("is_focused").is_some_and(js_truthy))
            .cloned();
    }
}

fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64().is_some_and(|number| number != 0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn apply(state: &mut NiriState, line: &str) -> NiriLineResult {
        state.apply_line(line).expect("valid event line")
    }

    #[test]
    fn consumes_shared_fixture_in_order() {
        let fixture = fs::read_to_string("docs/fixtures/niri.events.mock.jsonl")
            .expect("shared Niri fixture exists");
        let mut state = NiriState::default();
        let results: Vec<_> = fixture
            .lines()
            .map(|line| apply(&mut state, line))
            .collect();
        assert_eq!(results[0], NiriLineResult::HandshakeAccepted);
        assert_eq!(state.workspaces.len(), 1);
        assert_eq!(
            state.windows.len(),
            0,
            "the final fixture event closes its window"
        );
        assert_eq!(state.focused_workspace_id, Some(Value::from(1)));
        assert_eq!(state.active_workspace_id, Some(Value::from(1)));
        assert!(state.connected);
        assert!(state.last_error.is_empty());
    }

    #[test]
    fn opening_and_changing_windows_preserves_original_signal_rules() {
        let mut state = NiriState::default();
        apply(
            &mut state,
            r#"{"WindowsChanged":{"windows":[{"id":1,"is_focused":true},{"id":2,"is_focused":false}]}}"#,
        );
        let opened = apply(
            &mut state,
            r#"{"WindowOpenedOrChanged":{"window":{"id":3,"is_focused":true}}}"#,
        );
        let NiriLineResult::Event(signals) = opened else {
            panic!("expected event")
        };
        assert!(matches!(signals.first(), Some(NiriSignal::WindowOpened(_))));
        assert_eq!(
            state.focused_window.as_ref().and_then(|w| w.get("id")),
            Some(&Value::from(3))
        );
        assert_eq!(state.windows[0]["is_focused"], Value::Bool(false));
        assert_eq!(state.windows[2]["is_focused"], Value::Bool(true));
        let changed = apply(
            &mut state,
            r#"{"WindowOpenedOrChanged":{"window":{"id":3,"is_focused":true,"title":"updated"}}}"#,
        );
        let NiriLineResult::Event(signals) = changed else {
            panic!("expected event")
        };
        assert_eq!(signals.len(), 1, "updates do not emit windowOpened");
    }

    #[test]
    fn workspace_activation_only_changes_focus_when_requested() {
        let mut state = NiriState::default();
        apply(
            &mut state,
            r#"{"WorkspacesChanged":{"workspaces":[{"id":1,"output":"A","is_focused":true,"is_active":true},{"id":2,"output":"A","is_focused":false,"is_active":false},{"id":3,"output":"B","is_focused":false,"is_active":true}]}}"#,
        );
        apply(
            &mut state,
            r#"{"WorkspaceActivated":{"id":2,"focused":false}}"#,
        );
        assert_eq!(state.workspaces[0]["is_active"], Value::Bool(false));
        assert_eq!(state.workspaces[1]["is_active"], Value::Bool(true));
        assert_eq!(state.workspaces[0]["is_focused"], Value::Bool(true));
        assert_eq!(state.workspaces[2]["is_active"], Value::Bool(true));
        assert_eq!(state.focused_workspace_id, Some(Value::from(1)));
        assert_eq!(state.active_workspace_id, Some(Value::from(2)));
    }

    #[test]
    fn invalid_json_sets_error_without_disconnecting_or_erasing_state() {
        let mut state = NiriState::default();
        apply(&mut state, r#"{"Ok":{"Handled":null}}"#);
        apply(
            &mut state,
            r#"{"WorkspacesChanged":{"workspaces":[{"id":7,"is_focused":true,"is_active":true}]}}"#,
        );
        let previous = state.clone();
        assert!(state.apply_line("{not-json").is_err());
        assert_eq!(state.workspaces, previous.workspaces);
        assert!(state.connected);
        assert_eq!(state.last_error, INVALID_JSON_ERROR);
    }

    #[test]
    fn handshake_rejection_disconnects_but_regular_events_keep_last_error() {
        let mut state = NiriState::default();
        assert_eq!(
            apply(&mut state, r#"{"Err":{"PermissionDenied":null}}"#),
            NiriLineResult::HandshakeRejected(r#"{"PermissionDenied":null}"#.into())
        );
        assert!(!state.connected);
        assert_eq!(state.last_error, r#"{"PermissionDenied":null}"#);
        apply(&mut state, r#"{"UnknownEvent":{"anything":true}}"#);
        assert!(state.connected);
        assert_eq!(state.last_error, r#"{"PermissionDenied":null}"#);
    }

    #[test]
    fn focus_urgency_and_active_window_events_update_only_their_target() {
        let mut state = NiriState::default();
        apply(
            &mut state,
            r#"{"WorkspacesChanged":{"workspaces":[{"id":1,"output":"A","is_focused":true,"is_active":true,"is_urgent":false},{"id":2,"output":"A","is_focused":false,"is_active":false,"is_urgent":false}]}}"#,
        );
        apply(
            &mut state,
            r#"{"WindowsChanged":{"windows":[{"id":10,"is_focused":true,"is_urgent":false},{"id":11,"is_focused":false,"is_urgent":false}]}}"#,
        );

        apply(&mut state, r#"{"WindowFocusChanged":{"id":11}}"#);
        assert_eq!(state.windows[0]["is_focused"], Value::Bool(false));
        assert_eq!(state.windows[1]["is_focused"], Value::Bool(true));
        assert_eq!(
            state.focused_window.as_ref().and_then(|w| w.get("id")),
            Some(&Value::from(11))
        );

        apply(
            &mut state,
            r#"{"WindowUrgencyChanged":{"id":11,"urgent":true}}"#,
        );
        apply(
            &mut state,
            r#"{"WorkspaceUrgencyChanged":{"id":2,"urgent":true}}"#,
        );
        apply(
            &mut state,
            r#"{"WorkspaceActiveWindowChanged":{"workspace_id":2,"active_window_id":11}}"#,
        );
        assert_eq!(state.windows[1]["is_urgent"], Value::Bool(true));
        assert_eq!(state.windows[0]["is_urgent"], Value::Bool(false));
        assert_eq!(state.workspaces[1]["is_urgent"], Value::Bool(true));
        assert_eq!(state.workspaces[0]["is_urgent"], Value::Bool(false));
        assert_eq!(state.workspaces[1]["active_window_id"], Value::from(11));
        assert!(state.connected);
    }

    #[test]
    fn window_layout_changes_are_projected_without_losing_other_fields() {
        let mut state = NiriState::default();
        apply(
            &mut state,
            r#"{"WindowsChanged":{"windows":[{"id":9,"title":"keep","layout":{"old":true}}]}}"#,
        );
        apply(
            &mut state,
            r#"{"WindowLayoutsChanged":{"changes":[[9,{"new":true}]]}}"#,
        );
        assert_eq!(state.windows[0]["title"], "keep");
        assert_eq!(state.windows[0]["layout"]["new"], true);
    }
}
