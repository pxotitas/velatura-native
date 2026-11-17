//! `BarChip.qml` for the buttons: a square capsule holding one glyph that
//! opens its panel. Alone, the chip is the capsule, so the hover pill fills
//! it less its one-pixel outline.

use gtk4::prelude::*;
use std::rc::Rc;

use super::capsule::Capsule;
use super::motion::{Curve, Tween};
use super::palette;
use super::settings::Settings;
use super::tokens;

pub struct ButtonChip {
    pub widget: Capsule,
}

impl ButtonChip {
    pub fn new(settings: &Rc<Settings>, glyph: &str, on_click: impl Fn() + 'static) -> Self {
        let side = settings.bar_height;
        let widget = Capsule::new();
        widget.set_size(side, side);
        let style = settings.surface();
        widget.set_radii([side as f32 / 2.0; 4]);
        widget.set_color(tokens::ground_of(style).gdk());
        widget.set_border(1.0, tokens::rim_of(style).gdk());

        let pill = Capsule::new();
        pill.set_size(side - 2, side - 2);
        pill.set_radii([(side - 2) as f32 / 2.0; 4]);
        pill.set_halign(gtk4::Align::Center);
        pill.set_valign(gtk4::Align::Center);

        let paint = {
            let pill = pill.clone();
            move |alpha: f64| {
                let mut color = palette::current().surface_hover;
                color.3 = alpha as f32;
                pill.set_color(color.gdk());
            }
        };
        paint(0.0);
        let hover = Tween::new(&widget, 0.0, paint);

        let label = gtk4::Label::new(Some(glyph));
        label.add_css_class("chip-glyph");
        label.set_halign(gtk4::Align::Center);
        label.set_valign(gtk4::Align::Center);

        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&pill));
        overlay.add_overlay(&label);
        widget.set_content(Some(&overlay));
        widget.set_cursor_from_name(Some("pointer"));

        let duration = tokens::DURATION_FAST * settings.motion();
        let motion = gtk4::EventControllerMotion::new();
        let enter = hover.clone();
        motion.connect_enter(move |_, _, _| enter.animate(1.0, duration, Curve::OutCubic));
        let leave = hover.clone();
        motion.connect_leave(move |_| leave.animate(0.0, duration, Curve::OutCubic));
        widget.add_controller(motion);

        let click = gtk4::GestureClick::new();
        click.connect_released(move |_, _, _, _| on_click());
        widget.add_controller(click);
        Self { widget }
    }
}
