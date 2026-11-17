//! Layer-shell surface helpers for Wayland, via gtk4-layer-shell.
//!
//! Every shell surface (bar, island, panels) is a layer surface anchored
//! like the original; ordinary `GtkWindow`s are only used where the original
//! uses xdg-toplevel windows (Settings).

use gtk4::gdk;
use gtk4_layer_shell::LayerShell;

use super::theme;

/// Configure `window` as the top bar layer surface of `monitor`: anchored to
/// the top edge, exclusive zone for the reserve band, no keyboard focus.
pub fn setup_bar_surface(window: &gtk4::ApplicationWindow, monitor: Option<&gdk::Monitor>) {
    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Top);
    window.set_namespace(Some("velatura-bar"));
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_margin(gtk4_layer_shell::Edge::Top, theme::BAR_TOP_MARGIN);
    window.set_exclusive_zone(theme::BAR_RESERVE);
    if let Some(monitor) = monitor {
        window.set_monitor(Some(monitor));
    }
}
