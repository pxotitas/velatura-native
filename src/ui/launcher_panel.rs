//! `LauncherPanel.qml`, first slice: the field. The glyph of the mode the
//! query resolves to, the text with its placeholder, and the rule under it.
//! The results (apps, clipboard, emoji, …) are filled in by the next slices.

use gtk4::prelude::*;

use crate::services::launcher;

pub const WIDTH: i32 = 560;
pub const HEIGHT: i32 = 520;
const GAP: i32 = 12;
const FIELD_HEIGHT: i32 = 34;

pub struct LauncherPanel {
    pub widget: gtk4::Box,
    field: gtk4::Text,
}

impl LauncherPanel {
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, GAP);

        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, GAP);
        row.set_height_request(FIELD_HEIGHT);

        // The current mode's glyph rather than a magnifier.
        let glyph = gtk4::Label::new(Some(launcher::mode_for("").icon));
        glyph.add_css_class("launcher-glyph");
        row.append(&glyph);

        let field = gtk4::Text::new();
        field.add_css_class("launcher-field");
        field.set_hexpand(true);
        field.set_valign(gtk4::Align::Center);
        field.set_placeholder_text(Some("Search…"));
        row.append(&field);
        widget.append(&row);

        let rule = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        rule.set_height_request(1);
        rule.add_css_class("panel-rule");
        widget.append(&rule);

        field.connect_changed(move |field| {
            glyph.set_text(launcher::mode_for(&field.text()).icon);
        });
        Self { widget, field }
    }

    /// The window only receives keys once the field holds focus.
    pub fn focus(&self) {
        self.field.grab_focus();
    }

    pub fn query(&self) -> String {
        self.field.text().to_string()
    }
}

impl Default for LauncherPanel {
    fn default() -> Self {
        Self::new()
    }
}
