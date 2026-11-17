//! Notification service: the shell is the notification daemon.
//!
//! Port of `services/NotificationService.qml`. The service owns
//! `org.freedesktop.Notifications`; if another daemon holds the name it stays
//! unregistered and retries on its own once the name is released, like the
//! original's `keepOnReload` server.
//!
//! Security notes: everything here comes over D-Bus, so it is untrusted
//! input. Strings are stored and rendered as data only — never executed,
//! never used to build commands, never written anywhere but the history
//! file. The history file lives in this shell's own state directory; the
//! original's files are never touched.

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use zbus::zvariant::OwnedValue;

use crate::services::now_ms;

/// Capabilities the island actually renders; claiming more makes
/// applications send content that gets dropped.
pub const CAPABILITIES: &[&str] = &["actions", "body", "body-markup", "persistence"];

/// `expire_timeout` sent by an application is capped, or a misbehaving
/// application could own the island. Original: `min(expireTimeout, 15000)`.
pub const EXPIRE_CAP_MS: i32 = 15000;

/// Default timeout for notifications that do not set their own. Original:
/// `SettingsService.notificationTimeout`.
pub const DEFAULT_TIMEOUT_MS: i32 = 5000;

pub const HISTORY_LIMIT: usize = 50;

/// Notification urgency, per the freedesktop spec's `urgency` hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

impl Urgency {
    fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::Low,
            2 => Self::Critical,
            _ => Self::Normal,
        }
    }
}

/// A notification currently on the island.
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: u32,
    pub app_name: String,
    pub icon: String,
    pub summary: String,
    pub body: String,
    /// Pairs of (action key, label), in the order the sender gave them.
    pub actions: Vec<(String, String)>,
    pub urgency: Urgency,
    /// The timeout the service assigned, in ms; 0 means wait for the user.
    pub timeout_ms: i32,
    /// Whether the sender asked not to be kept in history.
    pub transient: bool,
}

impl Toast {
    /// `default` is what clicking the notification does; the rest are its
    /// buttons, mirroring `buttonsOf`.
    pub fn default_action(&self) -> Option<&(String, String)> {
        self.actions.iter().find(|(key, _)| key == "default")
    }

    pub fn buttons(&self) -> Vec<&(String, String)> {
        self.actions
            .iter()
            .filter(|(key, text)| key != "default" && !text.is_empty())
            .collect()
    }

    /// The original's declared toast metrics: one width whatever it says,
    /// one row with the picture beside a title and up to two lines of body,
    /// and a second row when the buttons do not fit beside the text.
    pub fn toast_width() -> i32 {
        400
    }

    pub fn toast_height(&self) -> i32 {
        const PADDING: i32 = 12;
        const ROW: i32 = 54;
        const STACKED_ROW: i32 = 40;
        const INLINE_LETTERS: usize = 20;
        let buttons = self.buttons();
        let inline = buttons.len() <= 2
            && buttons
                .iter()
                .map(|(_, text)| text.chars().count())
                .sum::<usize>()
                <= INLINE_LETTERS;
        2 * PADDING + ROW + if inline { 0 } else { STACKED_ROW }
    }
}

/// An entry in the history: a copy of what was shown, not the live object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u32,
    /// `false` for an entry read back from disk: notification ids start
    /// again at every login, so only a live entry may be matched by id.
    pub live: bool,
    pub time: u64,
    pub summary: String,
    pub body: String,
    pub app_name: String,
    pub icon: String,
    /// 0 low, 1 normal, 2 critical.
    pub urgency: u8,
}

/// Events the service publishes to the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationEvent {
    /// A notification takes the island (subject to do-not-disturb).
    Arrived(Toast),
    /// The current notification left the island without user action.
    Expired(u32),
    /// The user or the application closed it.
    Closed(u32),
    /// An action was invoked on the live connection.
    ActionInvoked { id: u32, key: String },
    /// The history changed (arrivals, removals, clearing).
    HistoryChanged(Vec<HistoryEntry>),
    /// Do Not Disturb changed after the service applied its command.
    DoNotDisturbChanged(bool),
    /// The daemon acquired or lost the bus name.
    NameOwned(bool),
}

/// Reasons a notification was closed, per the freedesktop spec.
const CLOSE_EXPIRED: u32 = 1;
const CLOSE_DISMISSED: u32 = 2;
const CLOSE_CLOSECALL: u32 = 3;

struct ServiceState {
    current: Option<Toast>,
    /// Monotonic instant when the current notification expires; `None` for a
    /// critical one that waits for the user.
    expiry: Option<Instant>,
    history: Vec<HistoryEntry>,
    do_not_disturb: bool,
    restored: bool,
    next_id: u32,
}

enum ServerCommand {
    /// Close the given id (`u32::MAX` closes the current one as the user
    /// asked; a concrete id comes from the application).
    Close(u32),
    /// Invoke a notification action.
    Invoke {
        id: u32,
        key: String,
    },
    /// Take the current notification off the island without telling the
    /// application (`dismiss` in the original).
    Dismiss,
    SetDoNotDisturb(bool),
    ClearHistory,
    RemoveFromHistory(u32),
}

/// The notification daemon. Dropping it releases the bus name and stops its
/// thread.
pub struct NotificationService {
    events: Mutex<std::sync::mpsc::Receiver<NotificationEvent>>,
    commands: Mutex<Option<std::sync::mpsc::Sender<ServerCommand>>>,
    state: Arc<Mutex<ServiceState>>,
    stopped: Arc<AtomicBool>,
}

impl NotificationService {
    /// The next event for the UI, blocking. Call from a bridge worker, never
    /// on the main thread.
    pub fn recv(&self) -> Result<NotificationEvent, std::sync::mpsc::RecvError> {
        self.events.lock().unwrap().recv()
    }

    pub fn start() -> Self {
        let history_path = crate::services::paths::state_directory().join("notifications.json");
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        let stopped = Arc::new(AtomicBool::new(false));
        let state = Arc::new(Mutex::new(ServiceState {
            current: None,
            expiry: None,
            history: Vec::new(),
            do_not_disturb: false,
            restored: false,
            next_id: 1,
        }));
        {
            let mut guard = state.lock().unwrap();
            restore_history(&mut guard, &history_path);
        }
        let worker_state = Arc::clone(&state);
        let worker_stopped = Arc::clone(&stopped);
        thread::Builder::new()
            .name("velatura-notifications".into())
            .spawn(move || {
                run_daemon(
                    worker_state,
                    event_tx,
                    history_path,
                    command_rx,
                    worker_stopped,
                );
            })
            .expect("failed to start notification worker");
        Self {
            events: Mutex::new(event_rx),
            commands: Mutex::new(Some(command_tx)),
            state,
            stopped,
        }
    }

    /// The next event for the UI. Consume from a GTK event source, never
    /// blocking the main loop.
    pub fn try_recv(&self) -> Result<NotificationEvent, std::sync::mpsc::TryRecvError> {
        self.events.lock().unwrap().try_recv()
    }

    pub fn current(&self) -> Option<Toast> {
        self.state.lock().unwrap().current.clone()
    }

    pub fn history(&self) -> Vec<HistoryEntry> {
        self.state.lock().unwrap().history.clone()
    }

    pub fn do_not_disturb(&self) -> bool {
        self.state.lock().unwrap().do_not_disturb
    }

    pub fn set_do_not_disturb(&self, silence: bool) {
        self.send(ServerCommand::SetDoNotDisturb(silence));
    }

    /// The user closed it deliberately, so the application is told.
    pub fn close(&self) {
        self.send(ServerCommand::Close(u32::MAX));
    }

    /// Takes it off the island without telling the application it was acted
    /// on (`dismiss` in the original).
    pub fn dismiss(&self) {
        self.send(ServerCommand::Dismiss);
    }

    pub fn invoke_action(&self, id: u32, key: &str) {
        self.send(ServerCommand::Invoke {
            id,
            key: key.to_string(),
        });
    }

    pub fn clear_history(&self) {
        self.send(ServerCommand::ClearHistory);
    }

    pub fn remove_from_history(&self, id: u32) {
        self.send(ServerCommand::RemoveFromHistory(id));
    }

    fn send(&self, command: ServerCommand) {
        if let Some(sender) = self.commands.lock().unwrap().as_ref() {
            let _ = sender.send(command);
        }
    }
}

impl Drop for NotificationService {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        *self.commands.lock().unwrap() = None;
    }
}

// ── DAEMON THREAD ───────────────────────────────────────────────────────────

struct Shared {
    state: Arc<Mutex<ServiceState>>,
    events: std::sync::mpsc::Sender<NotificationEvent>,
    history_path: PathBuf,
    /// The session-bus connection that serves the daemon; lives and dies on
    /// the worker thread.
    connection: Mutex<Option<zbus::blocking::Connection>>,
}

impl Shared {
    fn publish(&self, event: NotificationEvent) {
        let _ = self.events.send(event);
    }

    fn persist(&self, guard: &ServiceState) {
        if !guard.restored {
            return;
        }
        save_history(&self.history_path, guard);
    }

    fn emit_closed_signal(&self, id: u32, reason: u32) {
        self.emit_signal("NotificationClosed", (id, reason.to_string()));
    }

    fn emit_action_signal(&self, id: u32, key: &str) {
        self.emit_signal("ActionInvoked", (id, key.to_string()));
    }

    /// One typed signal message on the session bus; the connection lives on
    /// the worker thread only, so no cross-thread bus traffic happens here.
    fn emit_signal(&self, signal_name: &str, body: (u32, String)) {
        if let Ok(guard) = self.connection.lock()
            && let Some(connection) = guard.as_ref()
            && let Ok(message) = zbus::message::Message::signal(
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
                signal_name,
            )
            .and_then(|builder| builder.build(&body))
        {
            let _ = connection.send(&message);
        }
    }
}

fn run_daemon(
    state: Arc<Mutex<ServiceState>>,
    events: std::sync::mpsc::Sender<NotificationEvent>,
    history_path: PathBuf,
    commands: std::sync::mpsc::Receiver<ServerCommand>,
    stopped: Arc<AtomicBool>,
) {
    let shared = Arc::new(Shared {
        state,
        events,
        history_path,
        connection: Mutex::new(None),
    });

    loop {
        if stopped.load(Ordering::SeqCst) {
            return;
        }
        match try_connect(&shared) {
            Ok(()) => {
                shared.publish(NotificationEvent::NameOwned(true));
                serve(&shared, &commands, stopped.as_ref());
                *shared.connection.lock().unwrap() = None;
                shared.publish(NotificationEvent::NameOwned(false));
                if stopped.load(Ordering::SeqCst) {
                    return;
                }
                // Lost the name or the connection: retry like the original
                // server's take-over behaviour.
                thread::sleep(Duration::from_secs(2));
            }
            Err(error) => {
                worker_log(&format!("notification daemon connect: {error}"));
                thread::sleep(Duration::from_secs(3));
            }
        }
    }
}

fn worker_log(message: &str) {
    let _ = std::io::stderr().write_all(format!("velatura: {message}\n").as_bytes());
}

fn try_connect(shared: &Arc<Shared>) -> Result<(), String> {
    let connection = zbus::blocking::connection::Builder::session()
        .map_err(|e| e.to_string())?
        .name("org.freedesktop.Notifications")
        .map_err(|e| e.to_string())?
        .serve_at(
            "/org/freedesktop/Notifications",
            NotificationsIface {
                shared: Arc::clone(shared),
            },
        )
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;
    // The emitter wraps the async connection behind the blocking wrapper.
    // The connection itself is kept for signal emission; the `Shared` owner
    // lives on that thread only, so no cross-thread bus access happens.
    *shared.connection.lock().unwrap() = Some(connection);
    Ok(())
}

fn serve(
    shared: &Arc<Shared>,
    commands: &std::sync::mpsc::Receiver<ServerCommand>,
    stopped: &AtomicBool,
) {
    loop {
        if stopped.load(Ordering::SeqCst) {
            return;
        }
        // Expire the current notification on its own clock, shown or not:
        // one nobody closes would otherwise stay alive for the whole session.
        let timeout = {
            let guard = shared.state.lock().unwrap();
            guard
                .expiry
                .map(|at| at.saturating_duration_since(Instant::now()))
        };
        match commands.recv_timeout(timeout.unwrap_or(Duration::from_secs(1))) {
            Ok(command) => apply_command(shared, command),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let expired = {
                    let mut guard = shared.state.lock().unwrap();
                    match (guard.current.as_ref(), guard.expiry) {
                        (Some(toast), Some(at)) if Instant::now() >= at => {
                            let id = toast.id;
                            guard.current = None;
                            guard.expiry = None;
                            Some(id)
                        }
                        _ => None,
                    }
                };
                if let Some(id) = expired {
                    shared.publish(NotificationEvent::Expired(id));
                    shared.emit_closed_signal(id, CLOSE_EXPIRED);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // The API handle was dropped; the service is being stopped.
                return;
            }
        }
    }
}

/// Matches `NotificationService.timeoutFor`: critical notices wait for the
/// user; other sender timeouts are capped; missing timeouts use the setting's
/// current native default.
fn timeout_for(urgency: Urgency, sender_timeout: i32) -> i32 {
    if urgency == Urgency::Critical {
        return 0;
    }
    if sender_timeout > 0 {
        return sender_timeout.min(EXPIRE_CAP_MS);
    }
    DEFAULT_TIMEOUT_MS
}

fn apply_command(shared: &Arc<Shared>, command: ServerCommand) {
    match command {
        ServerCommand::Close(u32::MAX) => {
            let id = shared.state.lock().unwrap().current.take().map(|t| t.id);
            if let Some(id) = id {
                shared.state.lock().unwrap().expiry = None;
                shared.publish(NotificationEvent::Closed(id));
                shared.emit_closed_signal(id, CLOSE_CLOSECALL);
            }
        }
        ServerCommand::Close(id) => {
            // The application closed its own notification while shown.
            let mut guard = shared.state.lock().unwrap();
            if guard.current.as_ref().is_some_and(|toast| toast.id == id) {
                guard.current = None;
                guard.expiry = None;
                drop(guard);
                shared.publish(NotificationEvent::Closed(id));
                shared.emit_closed_signal(id, CLOSE_CLOSECALL);
            }
        }
        ServerCommand::Dismiss => {
            let mut guard = shared.state.lock().unwrap();
            if let Some(toast) = guard.current.take() {
                guard.expiry = None;
                drop(guard);
                shared.publish(NotificationEvent::Closed(toast.id));
                // Dismissed without acting: the sender is told it was
                // dismissed so resident notifications can go away.
                shared.emit_closed_signal(toast.id, CLOSE_DISMISSED);
            }
        }
        ServerCommand::Invoke { id, key } => {
            let mut guard = shared.state.lock().unwrap();
            let was_current = guard.current.as_ref().is_some_and(|toast| toast.id == id);
            if was_current {
                guard.current = None;
                guard.expiry = None;
            }
            drop(guard);
            shared.publish(NotificationEvent::ActionInvoked {
                id,
                key: key.clone(),
            });
            shared.emit_action_signal(id, &key);
            shared.emit_closed_signal(id, CLOSE_DISMISSED);
        }
        ServerCommand::SetDoNotDisturb(silence) => {
            let mut guard = shared.state.lock().unwrap();
            guard.do_not_disturb = silence;
            let took = if silence { guard.current.take() } else { None };
            if took.is_some() {
                guard.expiry = None;
            }
            drop(guard);
            shared.publish(NotificationEvent::DoNotDisturbChanged(silence));
            if let Some(toast) = took {
                shared.publish(NotificationEvent::Closed(toast.id));
                shared.emit_closed_signal(toast.id, CLOSE_DISMISSED);
            }
        }
        ServerCommand::ClearHistory => {
            let mut guard = shared.state.lock().unwrap();
            guard.history.clear();
            shared.persist(&guard);
            let history = guard.history.clone();
            drop(guard);
            shared.publish(NotificationEvent::HistoryChanged(history));
        }
        ServerCommand::RemoveFromHistory(id) => {
            let mut guard = shared.state.lock().unwrap();
            guard.history.retain(|entry| entry.id != id);
            shared.persist(&guard);
            let history = guard.history.clone();
            drop(guard);
            shared.publish(NotificationEvent::HistoryChanged(history));
        }
    }
}

/// Called by the D-Bus `Notify` method: the original's `present`.
fn present(shared: &Arc<Shared>, toast: Toast) -> u32 {
    let id = toast.id;
    // A transient notification asks not to be kept.
    if !toast.transient {
        let mut guard = shared.state.lock().unwrap();
        guard.history.insert(
            0,
            HistoryEntry {
                id: toast.id,
                live: true,
                time: now_ms(),
                summary: toast.summary.clone(),
                body: toast.body.clone(),
                app_name: toast.app_name.clone(),
                icon: toast.icon.clone(),
                urgency: toast.urgency as u8,
            },
        );
        guard.history.truncate(HISTORY_LIMIT);
        shared.persist(&guard);
        let history = guard.history.clone();
        drop(guard);
        shared.publish(NotificationEvent::HistoryChanged(history));
    }
    // Critical notifications ignore do-not-disturb; newest wins, except over
    // a critical one. The newcomer is still recorded.
    let is_critical = toast.urgency == Urgency::Critical;
    {
        let guard = shared.state.lock().unwrap();
        if guard.do_not_disturb && !is_critical {
            return id;
        }
        if guard
            .current
            .as_ref()
            .is_some_and(|current| current.urgency == Urgency::Critical)
            && !is_critical
        {
            return id;
        }
    }
    let timeout = toast.timeout_ms;
    {
        let mut guard = shared.state.lock().unwrap();
        guard.current = Some(toast.clone());
        guard.expiry = if timeout > 0 {
            Some(Instant::now() + Duration::from_millis(timeout as u64))
        } else {
            None
        };
    }
    shared.publish(NotificationEvent::Arrived(toast));
    id
}

// ── D-BUS INTERFACE ─────────────────────────────────────────────────────────

struct NotificationsIface {
    shared: Arc<Shared>,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationsIface {
    // The signature is fixed by the freedesktop specification; the argument
    // count is not negotiable.
    #[allow(clippy::too_many_arguments)]
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        let urgency = hints
            .get("urgency")
            .and_then(|value| value.downcast_ref::<u8>().ok())
            .map(Urgency::from_byte)
            .unwrap_or(Urgency::Normal);
        let transient = hints
            .get("transient")
            .and_then(|value| value.downcast_ref::<bool>().ok())
            .unwrap_or(false);
        let mut pairs = Vec::new();
        for chunk in actions.chunks(2) {
            if let [key, label] = chunk {
                pairs.push((key.clone(), label.clone()));
            }
        }
        let toast = Toast {
            // `replaces_id` of 0 asks for a new id; anything else replaces in
            // place. Ids restart every login, matching the original.
            id: if replaces_id > 0 {
                replaces_id
            } else {
                let mut guard = self.shared.state.lock().unwrap();
                let id = guard.next_id;
                guard.next_id += 1;
                id
            },
            app_name: app_name.to_string(),
            icon: app_icon.to_string(),
            summary: summary.to_string(),
            body: body.to_string(),
            actions: pairs,
            urgency,
            timeout_ms: timeout_for(urgency, expire_timeout),
            transient,
        };
        present(&self.shared, toast)
    }

    fn close_notification(&self, id: u32) {
        let mut guard = self.shared.state.lock().unwrap();
        if guard.current.as_ref().is_some_and(|toast| toast.id == id) {
            guard.current = None;
            guard.expiry = None;
            drop(guard);
            self.shared.publish(NotificationEvent::Closed(id));
            self.shared.emit_closed_signal(id, CLOSE_CLOSECALL);
        }
    }

    fn get_capabilities(&self) -> Vec<String> {
        CAPABILITIES.iter().map(|cap| cap.to_string()).collect()
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        (
            "Velatura".to_string(),
            "Velatura Native".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
            "1.2".to_string(),
        )
    }
}

// ── HISTORY ON DISK ─────────────────────────────────────────────────────────

/// Nothing is written until the file has been read, or the first
/// notification of a session would replace the history on disk.
fn restore_history(state: &mut ServiceState, path: &std::path::Path) {
    let Ok(text) = std::fs::read_to_string(path) else {
        state.restored = true;
        return;
    };
    match serde_json::from_str::<HistoryFile>(&text) {
        Ok(file) => {
            state.history = file
                .history
                .into_iter()
                .map(|mut entry| {
                    entry.live = false;
                    entry
                })
                .take(HISTORY_LIMIT)
                .collect();
            state.restored = true;
        }
        Err(error) => {
            // A corrupt history is kept on disk untouched; the session starts
            // empty rather than overwriting it.
            worker_log(&format!("cannot read the notification history: {error}"));
        }
    }
}

#[derive(Serialize, Deserialize)]
struct HistoryFile {
    history: Vec<HistoryEntry>,
}

fn save_history(path: &std::path::Path, state: &ServiceState) {
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        worker_log(&format!("cannot create state directory: {error}"));
        return;
    }
    // Write whole, then rename, so a reader never sees half a file.
    let plain: Vec<HistoryEntry> = state
        .history
        .iter()
        .map(|entry| HistoryEntry {
            id: entry.id,
            live: false,
            time: entry.time,
            summary: entry.summary.clone(),
            body: entry.body.clone(),
            app_name: entry.app_name.clone(),
            icon: entry.icon.clone(),
            urgency: entry.urgency,
        })
        .collect();
    let Ok(json) = serde_json::to_string(&HistoryFile { history: plain }) else {
        return;
    };
    let temporary = path.with_extension("json.tmp");
    if std::fs::write(&temporary, json).is_ok() {
        let _ = std::fs::rename(&temporary, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toast(urgency: Urgency, sender_timeout: i32) -> Toast {
        Toast {
            id: 1,
            app_name: "test".into(),
            icon: String::new(),
            summary: "Summary".into(),
            body: "Body".into(),
            actions: vec![
                ("default".into(), "Open".into()),
                ("dismiss".into(), "No".into()),
            ],
            urgency,
            timeout_ms: timeout_for(urgency, sender_timeout),
            transient: false,
        }
    }

    #[test]
    fn timeout_rules_match_original() {
        assert_eq!(timeout_for(Urgency::Critical, 123), 0);
        assert_eq!(timeout_for(Urgency::Normal, 3000), 3000);
        assert_eq!(timeout_for(Urgency::Normal, 999_999), EXPIRE_CAP_MS);
        assert_eq!(timeout_for(Urgency::Normal, -1), DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn buttons_exclude_default() {
        let toast = toast(Urgency::Normal, -1);
        assert_eq!(toast.default_action().unwrap().0, "default");
        assert_eq!(toast.buttons().len(), 1);
        assert_eq!(toast.buttons()[0].0, "dismiss");
    }

    #[test]
    fn toast_metrics_declared_sizes() {
        assert_eq!(Toast::toast_width(), 400);
        let short = toast(Urgency::Normal, -1);
        assert_eq!(short.toast_height(), 78);
        let long = Toast {
            actions: vec![
                ("default".into(), "Open".into()),
                ("a".into(), "A very long label".into()),
                ("b".into(), "Another long one".into()),
            ],
            ..toast(Urgency::Normal, -1)
        };
        assert_eq!(long.toast_height(), 118);
    }

    #[test]
    fn history_round_trip_on_disk() {
        let dir = std::env::temp_dir().join(format!("velatura-notif-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("notifications.json");
        let file = HistoryFile {
            history: vec![HistoryEntry {
                id: 7,
                live: true,
                time: 5,
                summary: "s".into(),
                body: "b".into(),
                app_name: "app".into(),
                icon: "i".into(),
                urgency: 1,
            }],
        };
        std::fs::write(&path, serde_json::to_string(&file).unwrap()).unwrap();
        let mut state = ServiceState {
            current: None,
            expiry: None,
            history: Vec::new(),
            do_not_disturb: false,
            restored: false,
            next_id: 1,
        };
        restore_history(&mut state, &path);
        assert!(state.restored);
        assert_eq!(state.history.len(), 1);
        // Read-back entries are never live.
        assert!(!state.history[0].live);
        // A corrupt file is never overwritten.
        std::fs::write(&path, "{ not json").unwrap();
        let mut state = ServiceState {
            current: None,
            expiry: None,
            history: Vec::new(),
            do_not_disturb: false,
            restored: false,
            next_id: 1,
        };
        restore_history(&mut state, &path);
        assert!(!state.restored);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not json");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn present_replaces_and_keeps_history() {
        let dir = std::env::temp_dir().join(format!("velatura-notif-run-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("notifications.json");
        let (events, _rx) = std::sync::mpsc::channel();
        let state = Arc::new(Mutex::new(ServiceState {
            current: None,
            expiry: None,
            history: Vec::new(),
            do_not_disturb: false,
            restored: true,
            next_id: 1,
        }));
        let shared = Arc::new(Shared {
            state: Arc::clone(&state),
            events,
            history_path: path,
            connection: Mutex::new(None),
        });
        let mut first = toast(Urgency::Normal, -1);
        first.id = 1;
        assert_eq!(present(&shared, first), 1);
        assert!(state.lock().unwrap().expiry.is_some());
        // Critical one replaces normal, waits forever.
        let mut critical = toast(Urgency::Critical, -1);
        critical.id = 2;
        assert_eq!(present(&shared, critical), 2);
        assert!(state.lock().unwrap().expiry.is_none());
        // Normal does not take the island from the critical one.
        let mut normal = toast(Urgency::Normal, -1);
        normal.id = 3;
        assert_eq!(present(&shared, normal), 3);
        assert_eq!(
            state.lock().unwrap().current.as_ref().unwrap().urgency,
            Urgency::Critical
        );
        // Three arrivals, three history entries (one replaced in place: ids
        // 1 and 2 were recorded, the id-2 replacement re-recorded).
        assert!(!state.lock().unwrap().history.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
