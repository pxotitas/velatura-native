//! Layer-shell plumbing for the full-screen transparent bar surface
//! and the independent strip that reserves the bar's usable area.

use gtk4::prelude::*;
use gtk4::{cairo, gdk};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

pub const NAMESPACE: &str = "impasto-bar";
const RESERVE_NAMESPACE: &str = "velatura-bar-reserve";

pub fn configure(window: &gtk4::ApplicationWindow, monitor: &gdk::Monitor) {
    window.init_layer_shell();
    window.set_namespace(Some(NAMESPACE));
    window.set_layer(Layer::Top);
    window.set_keyboard_mode(KeyboardMode::None);
    window.set_monitor(Some(monitor));
    for edge in [Edge::Top, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
    }
    // The bar itself ignores exclusive zones. A separate background surface
    // owns the reservation, matching bar/BarReserve.qml.
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

/// One 1px-high, input-transparent layer-shell surface per output. Its
/// exclusive zone keeps tiled windows below the bar/island band even though
/// the visible bar surface itself reserves nothing.
pub struct Reservation {
    window: gtk4::ApplicationWindow,
    pub connector: String,
}

impl Reservation {
    pub fn new(
        app: &gtk4::Application,
        monitor: &gdk::Monitor,
        exclusive_zone: i32,
    ) -> Self {
        let window = gtk4::ApplicationWindow::new(app);
        window.init_layer_shell();
        window.set_namespace(Some(RESERVE_NAMESPACE));
        window.set_layer(Layer::Background);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_monitor(Some(monitor));
        for edge in [Edge::Top, Edge::Left, Edge::Right] {
            window.set_anchor(edge, true);
        }
        window.set_exclusive_zone(exclusive_zone.max(0));
        let geometry = monitor.geometry();
        window.set_default_size(geometry.width(), 1);
        window.set_decorated(false);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        content.set_size_request(-1, 1);
        window.set_child(Some(&content));
        window.connect_map(|mapped| set_input_rects(mapped, &[]));

        let connector = monitor.connector().map(|name| name.to_string()).unwrap_or_default();
        window.present();
        Self { window, connector }
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        self.window.destroy();
    }
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
