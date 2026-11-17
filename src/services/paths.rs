//! User paths. This shell owns its own state under `XDG_STATE_HOME/velatura`
//! and never touches the original Quickshell configuration or state.

use std::path::PathBuf;

fn xdg_home(env: &str, fallback: &str) -> PathBuf {
    std::env::var_os(env).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| "/".into())).join(fallback)
    })
}

/// `XDG_STATE_HOME/velatura` (fallback `~/.local/state/velatura`).
pub fn state_directory() -> PathBuf {
    xdg_home("XDG_STATE_HOME", ".local/state").join("velatura")
}

/// `XDG_CACHE_HOME/velatura` (fallback `~/.cache/velatura`).
pub fn cache_directory() -> PathBuf {
    xdg_home("XDG_CACHE_HOME", ".cache").join("velatura")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_directory_is_under_xdg_state_home() {
        let dir = state_directory();
        assert!(dir.ends_with("velatura"));
        assert!(
            dir.starts_with(
                std::env::var_os("XDG_STATE_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(
                        || PathBuf::from(std::env::var_os("HOME").unwrap()).join(".local/state")
                    )
            )
        );
    }
}
