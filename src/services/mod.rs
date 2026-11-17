//! Shared service modules: domain logic and system adapters with no GTK
//! dependency. Each service is owned by the application runtime and talks to
//! the UI through typed events.

pub mod clipboard;
pub mod island_state;
pub mod launcher;
#[cfg(feature = "gtk-app")]
pub mod notifications;
pub mod paths;

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0)
}
