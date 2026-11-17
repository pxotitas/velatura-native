//! Layer-shell plumbing for the full-screen transparent bar surface
//! (`Bar.qml`'s PanelWindow): anchored top/left/right, reserving nothing,
//! never resized; input is limited by a region instead.

use gtk4::prelude::*;
use gtk4::{cairo, gdk};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

pub const NAMESPACE: &str = "impasto-bar";

pub fn configure(window: &gtk4::ApplicationWindow, monitor: &gdk::Monitor) {
    window.init_layer_shell();
    window.set_namespace(Some(NAMESPACE));
    window.set_layer(Layer::Top);
    window.set_keyboard_mode(KeyboardMode::None);
    window.set_monitor(Some(monitor));
    for edge in [Edge::Top, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
    }
    // ExclusionMode.Ignore: stays against the edge whatever else reserved.
    window.set_exclusive_zone(-1);
    let geometry = monitor.geometry();
    window.set_default_size(geometry.width(), geometry.height());
    // Development aid: lets a second instance sit below the running original
    // so the two can be compared on one screen.
    if let Some(margin) = std::env::var("VELATURA_DEBUG_TOP_MARGIN").ok().and_then(|v| v.parse().ok()) {
        window.set_margin(Edge::Top, margin);
    }
    window.set_decorated(false);
    window.add_css_class("velatura-surface");
}

/// Limits the surface's input to `rects` (x, y, width, height). Applied now
/// when the window is mapped and again on realize.
pub fn set_input_rects(window: &gtk4::ApplicationWindow, rects: &[(i32, i32, i32, i32)]) {
    let Some(surface) = window.surface() else {
        return;
    };
    let region = cairo::Region::create();
    for &(x, y, width, height) in rects {
        if width > 0 && height > 0 {
            let _ = region.union_rectangle(&cairo::RectangleInt::new(x, y, width, height));
        }
    }
    surface.set_input_region(Some(&region));
}
