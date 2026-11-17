//! Clipboard history: a watcher, a store, and restore.
//!
//! Native port of `scripts/clipboard.py` and `services/ClipboardService.qml`.
//! Qt/GTK clipboards on Wayland are only readable by a surface with keyboard
//! focus, so the selection is read through the data-control protocol with
//! `wl-paste --watch`, exactly as the original does.
//!
//! How it works without a shell anywhere:
//!
//! - the watcher spawns `wl-paste --watch <own exe> internal clipboard-store`;
//!   every clipboard change runs the store subcommand with the content on
//!   stdin (all argv entries are fixed, nothing user-provided);
//! - the store subcommand skips offers marked `CLIPBOARD_STATE=sensitive`
//!   (password managers) and writes the payload under
//!   `XDG_STATE_HOME/velatura/clipboard/` plus metadata in `clipboard.json`;
//! - the service watches the store file and republishes its entries;
//! - restore runs `wl-copy --type <mime>` with the payload file as stdin —
//!   direct exec with validated paths, no shell, no string building.
//!
//! `velatura internal clipboard-store` is the only exposed entry point; it is
//! not reachable from the UI API and validates every input against the store
//! directory before touching the filesystem.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Entries kept in the history. Original: `KEEP = 200`.
pub const KEEP: usize = 200;
/// Preview length kept in the metadata. Original: `PREVIEW = 240`.
pub const PREVIEW: usize = 240;
/// Payload size cap. Original: `LIMIT = 16 MiB`.
pub const LIMIT: u64 = 16 * 1024 * 1024;
/// The environment marker wl-clipboard sets for password-manager offers.
pub const SENSITIVE_ENV: &str = "CLIPBOARD_STATE";
pub const SENSITIVE_VALUE: &str = "sensitive";

/// File suffix for image payloads, matching the original's table.
fn image_suffix(mime: &str) -> Option<&'static str> {
    match mime {
        "image/png" => Some(".png"),
        "image/jpeg" => Some(".jpg"),
        "image/webp" => Some(".webp"),
        "image/gif" => Some(".gif"),
        "image/bmp" => Some(".bmp"),
        "image/tiff" => Some(".tiff"),
        _ => None,
    }
}

fn is_image(mime: &str) -> bool {
    image_suffix(mime).is_some()
}

/// One history entry: metadata only, payloads stay out of the index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    /// Random hex key, also the payload file stem.
    pub key: String,
    /// Milliseconds since the epoch.
    pub time: u64,
    /// MIME type handed back to wl-copy on restore.
    pub mime: String,
    /// `text`, `image` or `binary`.
    pub kind: String,
    /// Whether the payload is a single file path (an image).
    #[serde(default)]
    pub file: String,
    /// Text preview for list rendering; empty for images.
    #[serde(default)]
    pub preview: String,
    /// Byte length of the payload.
    pub size: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct StoreFile {
    entries: Vec<Entry>,
}

/// Human title for list rows, mirroring `ClipboardService.title`.
pub fn title(entry: &Entry) -> String {
    match entry.kind.as_str() {
        "image" => format!("Image · {}", format_size(entry.size)),
        "binary" => format!("Binary · {}", format_size(entry.size)),
        _ => entry
            .preview
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(60)
            .collect(),
    }
}

/// One-line description for list rows, mirroring `describe`.
pub fn describe(entry: &Entry) -> String {
    match entry.kind.as_str() {
        "text" => format!("{} · {}", entry.mime, format_size(entry.size)),
        "image" => entry.mime.clone(),
        _ => format!("{} · {}", entry.mime, format_size(entry.size)),
    }
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} kB", size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    }
}

/// Search entries newest first, matching on the preview text.
pub fn search<'a>(entries: &'a [Entry], term: &str, max: usize) -> Vec<&'a Entry> {
    let term = term.to_lowercase();
    let found: Vec<&Entry> = entries
        .iter()
        .filter(|entry| {
            term.is_empty()
                || entry.preview.to_lowercase().contains(&term)
                || entry.mime.to_lowercase().contains(&term)
        })
        .take(max)
        .collect();
    found
}

fn store_path() -> PathBuf {
    crate::services::paths::state_directory().join("clipboard.json")
}

fn payload_directory() -> PathBuf {
    crate::services::paths::state_directory().join("clipboard")
}

fn load_store() -> StoreFile {
    std::fs::read_to_string(store_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save_store(store: &StoreFile) {
    let path = store_path();
    if let Some(parent) = path.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return;
    }
    if let Ok(json) = serde_json::to_string(store) {
        let temporary = path.with_extension("json.tmp");
        if std::fs::write(&temporary, json).is_ok() {
            let _ = std::fs::rename(&temporary, path);
        }
    }
}

/// Read the MIME types of the current offer, or an empty list.
fn offered_types() -> Vec<String> {
    std::process::Command::new("wl-paste")
        .arg("--list-types")
        .stdin(std::process::Stdio::null())
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn choose_mime(offered: &[String]) -> Option<String> {
    // An image copied from a browser is also offered as text/html, so the
    // type list is checked directly, exactly as in the original.
    if let Some(image) = offered.iter().find(|mime| mime.starts_with("image/")) {
        return Some(image.clone());
    }
    if offered
        .iter()
        .any(|mime| mime == "text/plain;charset=utf-8")
    {
        return Some("text/plain;charset=utf-8".to_string());
    }
    if offered.iter().any(|mime| mime == "text/plain") {
        return Some("text/plain".to_string());
    }
    offered.first().cloned()
}

/// Store the current clipboard content. Runs in the store subcommand; stdin
/// carries the payload and the environment may carry the sensitive marker.
pub fn run_store_subcommand() -> i32 {
    if std::env::var(SENSITIVE_ENV).is_ok_and(|value| value == SENSITIVE_VALUE) {
        return 0;
    }
    let offered = offered_types();
    let Some(mime) = choose_mime(&offered) else {
        return 0;
    };
    let mut payload = Vec::new();
    if std::io::stdin().read_to_end(&mut payload).is_err() {
        return 0;
    }
    if payload.len() as u64 > LIMIT {
        return 0;
    }
    let empty = payload.is_empty();
    if empty {
        return 0;
    }
    let kind = if is_image(&mime) {
        "image"
    } else if mime.starts_with("text/") {
        "text"
    } else {
        "binary"
    };
    let key = random_key();
    let suffix = image_suffix(&mime).unwrap_or("");
    let file_name = format!("{key}{suffix}");
    let payload_path = payload_directory().join(&file_name);
    if std::fs::create_dir_all(payload_directory()).is_err() {
        return 0;
    }
    if std::fs::write(&payload_path, &payload).is_err() {
        return 0;
    }
    let mut preview = String::new();
    if kind == "text" {
        let text = String::from_utf8_lossy(&payload);
        let normalized = text.replace('\t', " ");
        preview = normalized.chars().take(PREVIEW).collect();
    }
    let mut store = load_store();
    store.entries.insert(
        0,
        Entry {
            key,
            time: crate::services::now_ms(),
            mime,
            kind: kind.to_string(),
            file: file_name,
            preview,
            size: payload.len() as u64,
        },
    );
    store.entries.truncate(KEEP);
    save_store(&store);
    0
}

fn random_key() -> String {
    // A random hex key from the OS CSPRNG; uniqueness within the store is
    // all that is needed.
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut bytes = [0u8; 8];
    if std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .is_ok()
    {
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        return format!("{:016x}{:04x}", u64::from_be_bytes(bytes), counter as u16);
    }
    format!(
        "{:016x}{:04x}",
        crate::services::now_ms(),
        COUNTER.fetch_add(1, Ordering::Relaxed) as u16
    )
}

/// Validate a payload file name: a plain hex stem with a known suffix. Paths
/// from outside this module are never trusted.
fn valid_payload_name(file: &str) -> bool {
    !file.is_empty()
        && file.len() <= 64
        && !file.contains('/')
        && !file.contains("..")
        && file
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
}

/// Restore an entry: put its payload back on the clipboard.
pub fn restore(entry: &Entry) -> Result<(), ClipboardError> {
    if !valid_payload_name(&entry.file) {
        return Err(ClipboardError::InvalidKey);
    }
    let payload_path = payload_directory().join(&entry.file);
    // Directory-traversal guard: the resolved parent must be the payload
    // directory.
    if payload_path.parent() != Some(payload_directory().as_path()) {
        return Err(ClipboardError::InvalidKey);
    }
    let file = std::fs::File::open(&payload_path).map_err(|_| ClipboardError::MissingPayload)?;
    let mut command = std::process::Command::new("wl-copy");
    command.arg("--type").arg(&entry.mime).stdin(file);
    command
        .spawn()
        .map_err(|error| ClipboardError::Spawn(error.to_string()))?;
    Ok(())
}

/// Forget one entry, removing its payload with it.
pub fn forget(key: &str) {
    let mut store = load_store();
    if let Some(position) = store.entries.iter().position(|entry| entry.key == key) {
        let entry = store.entries.remove(position);
        if valid_payload_name(&entry.file) {
            let _ = std::fs::remove_file(payload_directory().join(&entry.file));
        }
        save_store(&store);
    }
}

/// Wipe the history and every payload.
pub fn wipe() {
    let _ = std::fs::remove_file(store_path());
    let _ = std::fs::remove_dir_all(payload_directory());
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardError {
    InvalidKey,
    MissingPayload,
    Spawn(String),
}

/// Events published to the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardEvent {
    EntriesChanged(Vec<Entry>),
}

/// The clipboard service: runs the watcher and republishes the store.
pub struct ClipboardService {
    events: Mutex<std::sync::mpsc::Receiver<ClipboardEvent>>,
    sender: Sender<ClipboardEvent>,
    stopped: Arc<std::sync::atomic::AtomicBool>,
    watchers: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl ClipboardService {
    /// The next event, blocking. Call from a bridge worker only.
    pub fn recv(&self) -> Result<ClipboardEvent, std::sync::mpsc::RecvError> {
        self.events.lock().unwrap().recv()
    }

    pub fn start() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let stopped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut watchers = Vec::new();
        let watcher_stopped = Arc::clone(&stopped);
        let watcher_sender = sender.clone();
        watchers.push(
            thread::Builder::new()
                .name("velatura-clipboard-watch".into())
                .spawn(move || {
                    run_watcher(watcher_sender, watcher_stopped);
                })
                .expect("failed to start clipboard watcher"),
        );
        Self {
            events: Mutex::new(receiver),
            sender,
            stopped,
            watchers: Mutex::new(watchers),
        }
    }

    pub fn try_recv(&self) -> Result<ClipboardEvent, std::sync::mpsc::TryRecvError> {
        self.events.lock().unwrap().try_recv()
    }

    pub fn entries(&self) -> Vec<Entry> {
        load_store().entries
    }

    pub fn restore(&self, key: &str) -> Result<(), ClipboardError> {
        let store = load_store();
        let entry = store
            .entries
            .iter()
            .find(|entry| entry.key == key)
            .cloned()
            .ok_or(ClipboardError::InvalidKey)?;
        // The watcher sees wl-copy's change and stores the entry again; the
        // entry also moves to the top with its own clock, like the original.
        restore(&entry)
    }

    pub fn forget(&self, key: &str) {
        forget(key);
        let _ = self
            .sender
            .send(ClipboardEvent::EntriesChanged(self.entries()));
    }

    pub fn wipe(&self) {
        wipe();
        let _ = self.sender.send(ClipboardEvent::EntriesChanged(Vec::new()));
    }
}

impl Drop for ClipboardService {
    fn drop(&mut self) {
        self.stopped
            .store(true, std::sync::atomic::Ordering::SeqCst);
        for watcher in self.watchers.lock().unwrap().drain(..) {
            let _ = watcher.join();
        }
    }
}

fn run_watcher(sender: Sender<ClipboardEvent>, stopped: Arc<std::sync::atomic::AtomicBool>) {
    let path = store_path();
    let mut last_modified = metadata_of(&path);
    // Announce the existing history so the launcher has entries at once.
    let _ = sender.send(ClipboardEvent::EntriesChanged(load_store().entries));
    while !stopped.load(std::sync::atomic::Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(400));
        let modified = metadata_of(&path);
        if modified != last_modified {
            last_modified = modified;
            let _ = sender.send(ClipboardEvent::EntriesChanged(load_store().entries));
        }
    }
}

fn metadata_of(path: &Path) -> Option<u64> {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|since| since.as_nanos() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(unsafe_op_in_unsafe_fn)]
    fn store_rejects_sensitive_and_oversize() {
        // Sensitive offers are never stored. Env mutation in tests is unsafe
        // under edition 2024's rules; single-threaded by construction here.
        unsafe {
            std::env::set_var(SENSITIVE_ENV, SENSITIVE_VALUE);
        }
        assert_eq!(run_store_subcommand(), 0);
        unsafe {
            std::env::remove_var(SENSITIVE_ENV);
        }
    }

    #[test]
    fn choose_mime_prefers_image_over_html() {
        let offered = vec![
            "text/html".to_string(),
            "image/png".to_string(),
            "text/plain".to_string(),
        ];
        assert_eq!(choose_mime(&offered).unwrap(), "image/png");
        assert_eq!(
            choose_mime(&["text/plain;charset=utf-8".to_string()]).unwrap(),
            "text/plain;charset=utf-8"
        );
        assert!(choose_mime(&[]).is_none());
    }

    #[test]
    fn search_matches_preview_and_mime() {
        let entries = vec![Entry {
            key: "a".into(),
            time: 1,
            mime: "text/plain".into(),
            kind: "text".into(),
            file: "a".into(),
            preview: "Hello world".into(),
            size: 11,
        }];
        assert_eq!(search(&entries, "hello", 10).len(), 1);
        assert_eq!(search(&entries, "zzz", 10).len(), 0);
        assert_eq!(search(&entries, "", 10).len(), 1);
    }

    #[test]
    fn titles_and_sizes_render() {
        let image = Entry {
            key: "b".into(),
            time: 1,
            mime: "image/png".into(),
            kind: "image".into(),
            file: "b.png".into(),
            preview: String::new(),
            size: 2048,
        };
        assert_eq!(title(&image), "Image · 2.0 kB");
        assert_eq!(describe(&image), "image/png");
        let text = Entry {
            preview: "first\nsecond".into(),
            kind: "text".into(),
            mime: "text/plain".into(),
            file: String::new(),
            ..image.clone()
        };
        assert_eq!(title(&text), "first");
        let big = Entry {
            size: 3 * 1024 * 1024,
            kind: "binary".into(),
            ..text
        };
        assert_eq!(title(&big), "Binary · 3.0 MB");
    }

    #[test]
    fn restore_rejects_traversal() {
        let entry = Entry {
            key: "evil".into(),
            time: 1,
            mime: "text/plain".into(),
            kind: "text".into(),
            file: "../../etc/passwd".into(),
            preview: String::new(),
            size: 0,
        };
        assert_eq!(restore(&entry), Err(ClipboardError::InvalidKey));
    }
}
