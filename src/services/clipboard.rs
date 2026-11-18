//! Clipboard history, faithful to scripts/clipboard.py and ClipboardService.qml.
//! The original Quickshell state directory and JSON schema are retained so
//! history remains available across the native-shell migration.
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex, mpsc::Sender},
    thread,
    time::Duration,
};

pub const KEEP: usize = 200;
pub const PREVIEW: usize = 240;
pub const LIMIT: u64 = 16 * 1024 * 1024;
pub const SENSITIVE_ENV: &str = "CLIPBOARD_STATE";
pub const SENSITIVE_VALUE: &str = "sensitive";
const SECRET_MIME: &str = "x-kde-passwordManagerHint";
const LOCK_EX: i32 = 2;
const LOCK_UN: i32 = 8;

unsafe extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub key: String,
    #[serde(default)]
    pub hash: String,
    pub kind: String,
    pub mime: String,
    pub file: String,
    #[serde(default)]
    pub preview: String,
    #[serde(default, alias = "size")]
    pub bytes: u64,
    #[serde(default, alias = "time")]
    pub copied: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct StoreFile {
    entries: Vec<Entry>,
}

#[derive(Debug)]
struct StoreLock(File);

impl StoreLock {
    fn acquire() -> std::io::Result<Self> {
        fs::create_dir_all(state_directory())?;
        fs::create_dir_all(payload_directory())?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(lock_path())?;
        loop {
            let result = unsafe { flock(file.as_raw_fd(), LOCK_EX) };
            if result == 0 {
                return Ok(Self(file));
            }
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = unsafe { flock(self.0.as_raw_fd(), LOCK_UN) };
    }
}

fn state_directory() -> PathBuf {
    let root = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| "/".into()))
                .join(".local/state")
        });
    root.join("quickshell")
}

fn store_path() -> PathBuf {
    state_directory().join("clipboard.json")
}
fn payload_directory() -> PathBuf {
    state_directory().join("clipboard")
}
fn lock_path() -> PathBuf {
    state_directory().join("clipboard.lock")
}

fn suffix(mime: &str) -> Option<&'static str> {
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

fn image_type(offered: &[String]) -> Option<String> {
    let images: Vec<_> = offered
        .iter()
        .filter(|mime| mime.starts_with("image/"))
        .collect();
    for preferred in [
        "image/png",
        "image/jpeg",
        "image/webp",
        "image/gif",
        "image/bmp",
        "image/tiff",
    ] {
        if let Some(found) = images.iter().find(|mime| mime.as_str() == preferred) {
            return Some((*found).clone());
        }
    }
    images.first().map(|mime| (*mime).clone())
}

fn offered_types() -> Vec<String> {
    Command::new("timeout")
        .args(["5", "wl-paste", "--list-types"])
        .stdin(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())
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

fn read_image(mime: &str) -> Option<Vec<u8>> {
    let output = Command::new("timeout")
        .args(["15", "wl-paste", "--no-newline", "--type", mime])
        .stdin(Stdio::null())
        .output()
        .ok()?;
    (output.status.success() && !output.stdout.is_empty()).then_some(output.stdout)
}

fn preview_of(body: &[u8]) -> String {
    std::str::from_utf8(body)
        .ok()
        .map(|text| {
            text.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(PREVIEW)
                .collect()
        })
        .unwrap_or_default()
}

fn digest(body: &[u8]) -> Option<String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(body).ok()?;
    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }
    let hash = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()?
        .to_string();
    (hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())).then_some(hash)
}

fn load_store() -> StoreFile {
    fs::read_to_string(store_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save_store(store: &StoreFile) -> std::io::Result<()> {
    let path = store_path();
    fs::create_dir_all(state_directory())?;
    let text = serde_json::to_vec(store)?;
    let temporary = state_directory().join(format!("clipboard.json.{}.part", std::process::id()));
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)?;
    file.write_all(&text)?;
    file.sync_all()?;
    fs::rename(&temporary, &path)?;
    if let Ok(directory) = File::open(state_directory()) {
        let _ = directory.sync_all();
    }
    Ok(())
}

fn valid_payload_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && !name.contains('/')
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn resolve_payload(file: &str) -> Option<PathBuf> {
    let candidate = PathBuf::from(file);
    let path = if candidate.is_absolute() {
        if candidate.parent()? != payload_directory() {
            return None;
        }
        if !valid_payload_name(candidate.file_name()?.to_str()?) {
            return None;
        }
        candidate
    } else {
        if !valid_payload_name(file) {
            return None;
        }
        payload_directory().join(file)
    };
    let canonical_dir = fs::canonicalize(payload_directory()).ok()?;
    let canonical_path = fs::canonicalize(&path).ok()?;
    (canonical_path.parent() == Some(canonical_dir.as_path())).then_some(canonical_path)
}

fn discard(entry: &Entry, remaining: &[Entry]) {
    if remaining.iter().any(|other| other.file == entry.file) {
        return;
    }
    if let Some(path) = resolve_payload(&entry.file) {
        let _ = fs::remove_file(path);
    }
}

fn load_entries() -> Vec<Entry> {
    load_store().entries
}
pub fn entries() -> Vec<Entry> {
    load_entries()
}

pub fn title(entry: &Entry) -> String {
    if entry.kind == "image" {
        "Image".into()
    } else if !entry.preview.is_empty() {
        entry.preview.clone()
    } else {
        "Not text".into()
    }
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{} kB", (bytes as f64 / 1024.0).round() as u64)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub fn since(copied: u64, now: u64) -> String {
    let seconds = now.saturating_sub(copied) / 1000;
    if seconds < 60 {
        "just now".into()
    } else if seconds < 3600 {
        format!("{} min ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{} h ago", seconds / 3600)
    } else {
        format!("{} d ago", seconds / 86400)
    }
}

pub fn describe(entry: &Entry) -> String {
    let kind = if entry.kind == "image" {
        "Image"
    } else {
        "Text"
    };
    format!(
        "{kind}  ·  {}  ·  {}",
        format_size(entry.bytes),
        since(entry.copied, crate::services::now_ms())
    )
}

pub fn search<'a>(entries: &'a [Entry], term: &str, max: usize) -> Vec<&'a Entry> {
    let wanted = term.to_lowercase();
    entries
        .iter()
        .filter(|entry| wanted.is_empty() || entry.preview.to_lowercase().contains(&wanted))
        .take(max)
        .collect()
}

fn store_payload(body: Vec<u8>, mime: String, kind: &str, keep: usize) -> std::io::Result<()> {
    if body.is_empty()
        || body.len() as u64 > LIMIT
        || (kind == "text" && body.iter().all(u8::is_ascii_whitespace))
    {
        return Ok(());
    }
    let hash =
        digest(&body).ok_or_else(|| std::io::Error::other("sha256sum unavailable or failed"))?;
    let _lock = StoreLock::acquire()?;
    let mut store = load_store();
    if let Some(index) = store.entries.iter().position(|entry| entry.hash == hash) {
        let mut entry = store.entries.remove(index);
        entry.copied = crate::services::now_ms();
        store.entries.insert(0, entry);
        return save_store(&store);
    }
    let key = format!("clip-{}", &hash[..12]);
    let file_name = format!(
        "{key}{}",
        suffix(&mime).unwrap_or(if kind == "text" { ".txt" } else { ".bin" })
    );
    let path = payload_directory().join(&file_name);
    fs::write(&path, &body)?;
    store.entries.insert(
        0,
        Entry {
            key,
            hash,
            kind: kind.into(),
            mime,
            file: path.to_string_lossy().into_owned(),
            preview: if kind == "text" {
                preview_of(&body)
            } else {
                String::new()
            },
            bytes: body.len() as u64,
            copied: crate::services::now_ms(),
        },
    );
    while store.entries.len() > keep.max(1) {
        let removed = store.entries.pop().expect("length checked");
        discard(&removed, &store.entries);
    }
    save_store(&store)
}

/// Entry point called by wl-paste --watch. Images are fetched again using
/// their actual MIME type because watcher stdin may contain HTML.
pub fn run_store_subcommand() -> i32 {
    if std::env::var(SENSITIVE_ENV).unwrap_or_else(|_| "data".into()) != "data" {
        return 0;
    }
    let offered = offered_types();
    if offered.iter().any(|mime| mime.contains(SECRET_MIME)) {
        return 0;
    }
    let mut input = Vec::new();
    if std::io::stdin().read_to_end(&mut input).is_err() {
        return 0;
    }
    let (body, mime, kind) = if let Some(image) = image_type(&offered) {
        if std::env::var("VELATURA_CLIPBOARD_IMAGES").is_ok_and(|value| value == "0") {
            return 0;
        }
        let Some(body) = read_image(&image) else {
            return 0;
        };
        (body, image, "image")
    } else {
        (input, "text/plain;charset=utf-8".into(), "text")
    };
    let keep = std::env::var("VELATURA_CLIPBOARD_KEEP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(KEEP)
        .max(1);
    store_payload(body, mime, kind, keep).map_or(0, |_| 0)
}

pub fn payload_path(file: &str) -> PathBuf {
    resolve_payload(file).unwrap_or_default()
}

pub fn restore(entry: &Entry) -> Result<(), ClipboardError> {
    let path = resolve_payload(&entry.file).ok_or(ClipboardError::InvalidKey)?;
    let file = File::open(path).map_err(|_| ClipboardError::MissingPayload)?;
    let mut child = Command::new("timeout")
        .args(["10", "wl-copy", "--type"])
        .arg(&entry.mime)
        .stdin(Stdio::from(file))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| ClipboardError::Spawn(error.to_string()))?;
    let _ = child.wait();
    Ok(())
}

pub fn restore_key(key: &str) -> Result<(), ClipboardError> {
    let lock = StoreLock::acquire().map_err(|error| ClipboardError::Spawn(error.to_string()))?;
    let entry = load_store()
        .entries
        .into_iter()
        .find(|entry| entry.key == key)
        .ok_or(ClipboardError::InvalidKey)?;
    drop(lock);
    restore(&entry)
}

pub fn forget(key: &str) {
    let Ok(_lock) = StoreLock::acquire() else {
        return;
    };
    let mut store = load_store();
    if let Some(index) = store.entries.iter().position(|entry| entry.key == key) {
        let entry = store.entries.remove(index);
        discard(&entry, &store.entries);
        let _ = save_store(&store);
    }
}

pub fn wipe() {
    if let Ok(_lock) = StoreLock::acquire() {
        let store = load_store();
        for entry in &store.entries {
            discard(entry, &[]);
        }
        let _ = save_store(&StoreFile::default());
    }
    let _ = Command::new("timeout")
        .args(["5", "wl-copy", "--clear"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardError {
    InvalidKey,
    MissingPayload,
    Spawn(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardEvent {
    EntriesChanged(Vec<Entry>),
}

pub struct ClipboardService {
    events: Mutex<std::sync::mpsc::Receiver<ClipboardEvent>>,
    sender: Sender<ClipboardEvent>,
    stopped: Arc<std::sync::atomic::AtomicBool>,
    watchers: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl ClipboardService {
    pub fn recv(&self) -> Result<ClipboardEvent, std::sync::mpsc::RecvError> {
        self.events.lock().unwrap().recv()
    }

    pub fn start(enabled: bool, keep: usize, images: bool) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let stopped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let watcher_sender = sender.clone();
        let watcher_stopped = Arc::clone(&stopped);
        let watcher = thread::Builder::new()
            .name("velatura-clipboard-watch".into())
            .spawn(move || run_watcher(watcher_sender, watcher_stopped, enabled, keep, images))
            .expect("failed to start clipboard watcher");
        Self {
            events: Mutex::new(receiver),
            sender,
            stopped,
            watchers: Mutex::new(vec![watcher]),
        }
    }

    pub fn try_recv(&self) -> Result<ClipboardEvent, std::sync::mpsc::TryRecvError> {
        self.events.lock().unwrap().try_recv()
    }
    pub fn entries(&self) -> Vec<Entry> {
        entries()
    }
    pub fn restore(&self, key: &str) -> Result<(), ClipboardError> {
        restore_key(key)
    }

    pub fn forget(&self, key: &str) {
        forget(key);
        let _ = self.sender.send(ClipboardEvent::EntriesChanged(entries()));
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

fn spawn_watcher(keep: usize, images: bool) -> Option<std::process::Child> {
    let executable = std::env::current_exe().ok()?;
    Command::new("wl-paste")
        .arg("--watch")
        .arg(executable)
        .args(["internal", "clipboard-store"])
        .env("VELATURA_CLIPBOARD_KEEP", keep.max(1).to_string())
        .env("VELATURA_CLIPBOARD_IMAGES", if images { "1" } else { "0" })
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

fn run_watcher(
    sender: Sender<ClipboardEvent>,
    stopped: Arc<std::sync::atomic::AtomicBool>,
    enabled: bool,
    keep: usize,
    images: bool,
) {
    if !enabled {
        let _ = sender.send(ClipboardEvent::EntriesChanged(entries()));
        return;
    }
    let path = store_path();
    let mut last_modified = metadata_of(&path);
    let mut child = spawn_watcher(keep, images);
    let mut started = child.as_ref().map(|_| std::time::Instant::now());
    let _ = sender.send(ClipboardEvent::EntriesChanged(entries()));
    while !stopped.load(std::sync::atomic::Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(250));
        if let Some(process) = child.as_mut() {
            match process.try_wait() {
                Ok(Some(_)) | Err(_) => {
                    // Match ClipboardService.qml: a watcher that dies in
                    // under five seconds is a broken command, not a crash to
                    // respawn. A previously healthy watcher may be restarted.
                    let was_stable =
                        started.is_some_and(|at| at.elapsed() >= Duration::from_secs(5));
                    child = None;
                    started = None;
                    if was_stable && !stopped.load(std::sync::atomic::Ordering::SeqCst) {
                        child = spawn_watcher(keep, images);
                        started = child.as_ref().map(|_| std::time::Instant::now());
                    }
                }
                Ok(None) => {}
            }
        }
        let modified = metadata_of(&path);
        if modified != last_modified {
            last_modified = modified;
            let _ = sender.send(ClipboardEvent::EntriesChanged(entries()));
        }
    }
    if let Some(mut process) = child {
        let _ = process.kill();
        let _ = process.wait();
    }
}

fn metadata_of(path: &Path) -> Option<u128> {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|since| since.as_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_selection_prefers_known_image_types_and_skips_secret_offers() {
        let offered = vec![
            "text/html".into(),
            "image/jpeg".into(),
            "image/png".into(),
            "application/x-kde-passwordManagerHint".into(),
        ];
        assert_eq!(image_type(&offered).as_deref(), Some("image/png"));
        assert!(offered.iter().any(|mime| mime.contains(SECRET_MIME)));
        assert_eq!(image_type(&["text/plain".into()]), None);
    }

    #[test]
    fn previews_collapse_whitespace_and_reject_non_utf8() {
        assert_eq!(preview_of(" first\n\t second  ".as_bytes()), "first second");
        assert!(preview_of(&[0xff, 0xfe]).is_empty());
    }

    #[test]
    fn formatting_matches_clipboard_service() {
        assert_eq!(format_size(2048), "2 kB");
        assert_eq!(since(1_000, 61_000), "1 min ago");
        let entry = Entry {
            key: "clip-x".into(),
            hash: "x".into(),
            kind: "image".into(),
            mime: "image/png".into(),
            file: "/tmp/x".into(),
            preview: String::new(),
            bytes: 2048,
            copied: 1,
        };
        assert_eq!(title(&entry), "Image");
    }

    #[test]
    fn payload_names_reject_traversal_and_accept_original_absolute_file_paths() {
        assert!(valid_payload_name("clip-deadbeef.png"));
        assert!(!valid_payload_name("../secret"));
        assert!(!valid_payload_name("/etc/passwd"));
        assert!(!valid_payload_name("clip/secret"));
    }

    #[test]
    fn original_quickshell_clipboard_json_schema_deserializes() {
        let json = r#"{"entries":[{"key":"clip-abcdef012345","hash":"abcdef0123456789","kind":"image","mime":"image/png","file":"/home/user/.local/state/quickshell/clipboard/clip-abcdef012345.png","preview":"","bytes":1234,"copied":1700000000000}]}"#;
        let store: StoreFile = serde_json::from_str(json).unwrap();
        assert_eq!(store.entries.len(), 1);
        assert_eq!(store.entries[0].bytes, 1234);
        assert_eq!(store.entries[0].copied, 1_700_000_000_000);
        assert!(
            store.entries[0]
                .file
                .starts_with("/home/user/.local/state/quickshell/clipboard/")
        );
    }

    #[test]
    fn sha256_digest_matches_known_vector() {
        assert_eq!(
            digest(b"abc").as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }
}
