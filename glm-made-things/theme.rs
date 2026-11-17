//! Shared visual tokens and the GTK CSS provider.
//!
//! Every value here comes from `theme/Theme.qml` and `design.md` (classic
//! surface style, the default). Components must consume these tokens instead
//! of hardcoding colors; the provider is installed once on the display.

use gtk4::CssProvider;
use gtk4::gdk;

/// Capsule height (`SettingsService.barHeight`).
pub const CAPSULE_HEIGHT: i32 = 32;
/// Top margin (`SettingsService.barMargin`).
pub const BAR_TOP_MARGIN: i32 = 16;
/// Side margin (`SettingsService.barMargin`).
pub const BAR_SIDE_MARGIN: i32 = 18;
/// Spacing between capsules.
pub const CAPSULE_SPACING: i32 = 8;
/// Exclusive/reserve band: top margin + capsule height.
pub const BAR_RESERVE: i32 = BAR_TOP_MARGIN + CAPSULE_HEIGHT;
/// Panel padding.
pub const PANEL_PADDING: i32 = 20;
/// Large radius.
pub const RADIUS_LARGE: i32 = 18;

/// Morph duration (`Theme.durationMorph` at motion scale 100).
pub const DURATION_MORPH_MS: u64 = 380;
/// Medium duration.
pub const DURATION_MEDIUM_MS: u64 = 200;
/// Fast duration.
pub const DURATION_FAST_MS: u64 = 140;

/// `Theme.island` — the black is the island's structural identity.
pub const ISLAND: &str = "#000000";
/// `Theme.islandSurface`.
pub const ISLAND_SURFACE: &str = "#141414";
/// `Theme.islandSurfaceHover`.
pub const ISLAND_SURFACE_HOVER: &str = "#1f1f1f";
/// `Theme.islandBorder`.
pub const ISLAND_BORDER: &str = "#262626";
/// `Theme.text`.
pub const TEXT: &str = "#ffffff";
/// Muted text for metadata.
pub const TEXT_MUTED: &str = "#9a9a9a";
/// Accent.
pub const ACCENT: &str = "#0a84ff";

/// The launcher panel metrics, shared with the island so its height math
/// stays in sync (`LauncherService.qml`).
pub const LAUNCHER_FIELD_HEIGHT: i32 = 34;
pub const LAUNCHER_ROW_HEIGHT: i32 = 48;
pub const LAUNCHER_ROW_SPACING: i32 = 2;
pub const LAUNCHER_GAP: i32 = 12;
pub const LAUNCHER_PANEL_WIDTH: i32 = 560;
pub const LAUNCHER_FIXED_HEIGHT: i32 = 520;
/// `SettingsService.launcherResults`.
pub const LAUNCHER_MAX_RESULTS: usize = 9;

/// Install the shared CSS on the display. Returns the provider so callers
/// can keep it alive; GTK holds a reference once added, so dropping is safe.
pub fn install(display: &gdk::Display) -> CssProvider {
    let provider = CssProvider::new();
    provider.load_from_string(&stylesheet());
    gtk4::style_context_add_provider_for_display(
        display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    provider
}

/// The classic stylesheet. Capsules are solid dark surfaces with a hairline
/// rim; hover states are one shade off the surface, exactly as the tokens
/// declare.
pub fn stylesheet() -> String {
    format!(
        r#"
/* ── Bar and island surfaces ─────────────────────────────────────── */
.velatura-bar {{
    background: transparent;
}}

.velatura-capsule {{
    background-color: {ISLAND};
    border-radius: 999px;
    color: {TEXT};
    border: 1px solid {ISLAND_BORDER};
    padding: 0 14px;
}}

.velatura-island {{
    background-color: {ISLAND};
    border-radius: 18px;
    color: {TEXT};
    border: 1px solid {ISLAND_BORDER};
}}

.velatura-module {{
    background-color: {ISLAND_SURFACE};
    border-radius: 999px;
    color: {TEXT};
    padding: 0 10px;
}}

.velatura-module:hover {{
    background-color: {ISLAND_SURFACE_HOVER};
}}

.velatura-module-label {{
    color: {TEXT};
    font-size: 13px;
    font-weight: 600;
}}

.velatura-module-muted {{
    color: {TEXT_MUTED};
    font-size: 11px;
}}

/* ── Panel (expanded island) ─────────────────────────────────────── */
.velatura-panel {{
    background-color: {ISLAND};
    color: {TEXT};
    border-radius: {RADIUS_LARGE}px;
    border: 1px solid {ISLAND_BORDER};
}}

.velatura-panel-inner {{
    padding: {PANEL_PADDING}px;
}}

.velatura-heading {{
    color: {TEXT};
    font-size: 16px;
    font-weight: 700;
}}

.velatura-muted {{
    color: {TEXT_MUTED};
    font-size: 11px;
}}

/* ── Launcher ────────────────────────────────────────────────────── */
.velatura-launcher-field {{
    background-color: {ISLAND_SURFACE};
    color: {TEXT};
    border-radius: 8px;
    border: 1px solid {ISLAND_BORDER};
    padding: 4px 10px;
    min-height: {LAUNCHER_FIELD_HEIGHT}px;
}}

.velatura-launcher-field:focus {{
    border-color: {ACCENT};
}}

.velatura-launcher-row {{
    border-radius: 8px;
    padding: 4px 8px;
    min-height: {LAUNCHER_ROW_HEIGHT}px;
}}

.velatura-launcher-row:hover {{
    background-color: {ISLAND_SURFACE_HOVER};
}}

.velatura-launcher-row:selected {{
    background-color: {ISLAND_SURFACE_HOVER};
}}

.velatura-launcher-name {{
    color: {TEXT};
    font-size: 14px;
    font-weight: 600;
}}

.velatura-launcher-subtitle {{
    color: {TEXT_MUTED};
    font-size: 11px;
}}

/* ── Notifications ───────────────────────────────────────────────── */
.velatura-notification-title {{
    color: {TEXT};
    font-size: 13px;
    font-weight: 700;
}}

.velatura-notification-body {{
    color: {TEXT_MUTED};
    font-size: 11px;
}}

.velatura-notification-button {{
    background-color: {ISLAND_SURFACE};
    color: {TEXT};
    border-radius: 999px;
    padding: 4px 12px;
}}

.velatura-notification-button:hover {{
    background-color: {ISLAND_SURFACE_HOVER};
}}

/* ── OSD ─────────────────────────────────────────────────────────── */
.velatura-osd-level {{
    background-color: {ISLAND_SURFACE};
    border-radius: 999px;
    min-height: 6px;
}}

.velatura-osd-level-fill {{
    background-color: {TEXT};
    border-radius: 999px;
    min-height: 6px;
}}
"#
    )
}
