//! Settings read from the legacy `settings.json` (same keys and defaults as
//! `SettingsService.qml`). Read-only for now: persistence is owned by the
//! settings backend that is still pending.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub workspace_style: String,
    pub bar_height: i32,
    pub bar_margin: i32,
    pub bar_side_margin: i32,
    pub bar_style: String,
    pub bar_sides: String,
    pub bar_left: Vec<String>,
    pub bar_right: Vec<String>,
    pub bar_everywhere: bool,
    pub bar_hidden: bool,
    pub island_attached: bool,
    pub island_summary: bool,
    pub surface_style: String,
    pub game_mode: bool,
    pub launcher_results: usize,
    pub launcher_order: String,
    pub clipboard_history: bool,
    pub clipboard_keep: usize,
    pub clipboard_images: bool,
    pub clipboard_wipe_on_lock: bool,
    pub dock_pinned: Vec<String>,
    pub clock_format: String,
    pub clock_shows_seconds: bool,
    pub clock_shows_date: bool,
    pub motion_scale: i32,
    pub motion_curve: String,
    pub font_family: String,
    pub font_mono: String,
    pub bar_shadow: bool,
    pub screen_corners: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            workspace_style: "dots".into(),
            bar_height: 32,
            bar_margin: 16,
            bar_side_margin: 18,
            bar_style: "grouped".into(),
            bar_sides: "capsule".into(),
            bar_left: vec!["workspaces".into(), "launcher".into()],
            bar_right: vec![
                "tray".into(),
                "notifications".into(),
                "network".into(),
                "bluetooth".into(),
                "volume".into(),
                "battery".into(),
            ],
            bar_everywhere: true,
            bar_hidden: false,
            island_attached: false,
            island_summary: true,
            surface_style: "classic".into(),
            game_mode: false,
            launcher_results: 9,
            launcher_order: "recent".into(),
            clipboard_history: true,
            clipboard_keep: 200,
            clipboard_images: true,
            clipboard_wipe_on_lock: false,
            dock_pinned: Vec::new(),
            clock_format: "HH:mm".into(),
            clock_shows_seconds: false,
            clock_shows_date: false,
            motion_scale: 100,
            motion_curve: "OutCubic".into(),
            font_family: "Inter, Cantarell, SF Pro Text, sans-serif".into(),
            font_mono: "JetBrainsMono Nerd Font Mono, monospace".into(),
            bar_shadow: false,
            screen_corners: false,
        }
    }
}

impl Settings {
    /// `$XDG_STATE_HOME/quickshell/settings.json`, the original's file.
    pub fn path() -> PathBuf {
        let state = std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
            })
            .unwrap_or_default();
        state.join("quickshell/settings.json")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Attached island: reaches the screen edge and flares into fillets.
    pub fn notched(&self) -> bool {
        self.island_attached || self.bar_style == "bezel"
    }

    pub fn unified(&self) -> bool {
        self.bar_style == "island" || self.bar_style == "bezel"
    }

    pub fn surface(&self) -> &str {
        if self.game_mode {
            "classic"
        } else {
            &self.surface_style
        }
    }

    /// Reserved top band, matching Theme.barReserve / BarReserve.qml.
    pub fn bar_reserve(&self) -> i32 {
        if self.bar_hidden {
            0
        } else {
            self.bar_margin + self.bar_height
        }
    }

    /// Width of the island at rest, `ModuleService.entry("clock").width`.
    pub fn rest_width(&self) -> i32 {
        if self.clock_shows_date { 240 } else { 150 }
    }

    /// `Theme.motion`: 0 in game mode.
    pub fn motion(&self) -> f64 {
        if self.game_mode {
            0.0
        } else {
            f64::from(self.motion_scale) / 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn bar_reserve_matches_original_theme_metric() {
        let settings = Settings::default();
        assert_eq!(settings.bar_reserve(), 48);
    }

    #[test]
    fn hidden_bar_releases_the_reservation() {
        let settings = Settings {
            bar_hidden: true,
            ..Settings::default()
        };
        assert_eq!(settings.bar_reserve(), 0);
    }
}
