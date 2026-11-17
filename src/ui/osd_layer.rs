//! `OsdLayer.qml`: one compact transient for volume, brightness and status.

use gtk4::prelude::*;

use super::motion::{Curve, Tween};
use super::settings::Settings;
use super::tokens;
use crate::services::island_state::TransientEvent;

pub struct OsdLayer {
    pub widget: gtk4::CenterBox,
    icon: gtk4::Label,
    level: gtk4::ProgressBar,
    progress: Tween,
    progress_duration: f64,
    curve: Curve,
    label: gtk4::Label,
}

impl OsdLayer {
    pub fn new(settings: &Settings) -> Self {
        let widget = gtk4::CenterBox::new();
        widget.set_halign(gtk4::Align::Center);
        widget.set_valign(gtk4::Align::Center);
        widget.set_size_request(240, -1);
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        row.set_halign(gtk4::Align::Center);
        row.set_valign(gtk4::Align::Center);
        widget.set_center_widget(Some(&row));

        let icon = gtk4::Label::new(None);
        icon.add_css_class("osd-glyph");
        row.append(&icon);

        let level = gtk4::ProgressBar::new();
        level.add_css_class("osd-level");
        level.set_size_request(130, 4);
        level.set_valign(gtk4::Align::Center);
        level.set_show_text(false);
        row.append(&level);
        let target = level.clone();
        let progress = Tween::new(&level, 0.0, move |fraction| {
            target.set_fraction(fraction.clamp(0.0, 1.0));
        });

        let label = gtk4::Label::new(None);
        label.add_css_class("osd-label");
        label.set_width_chars(3);
        label.set_size_request(34, -1);
        label.set_xalign(1.0);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        row.append(&label);

        Self {
            widget,
            icon,
            level,
            progress,
            progress_duration: tokens::DURATION_FAST * settings.motion(),
            curve: Curve::from_id(&settings.motion_curve),
            label,
        }
    }

    pub fn set_event(&self, event: &TransientEvent) {
        self.icon.set_text(&event.icon);
        self.label.set_text(&event.label);
        let has_level = event.progress >= 0.0;
        self.level.set_visible(has_level);
        if has_level {
            self.progress.animate(
                f64::from(event.progress.clamp(0.0, 1.0)),
                self.progress_duration,
                self.curve,
            );
        }
    }
}
