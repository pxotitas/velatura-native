//! `NotificationLayer.qml`: the island while a notification is shown, in one
//! row: the picture, the title with the app and "now" at its end, up to two
//! lines of body, and up to two short buttons at the end (the first in the
//! accent). More or longer buttons share a row underneath. A click is the
//! default action, or closes it when there is none; a right click closes it.

use gtk4::prelude::*;
use gtk4::pango;
use std::rc::Rc;

use super::capsule::Capsule;
use super::motion::{Curve, Tween};
use super::palette;
use super::tokens::{self, Rgba};
use crate::services::notifications::{NotificationService, Toast, Urgency};

const PICTURE: i32 = 48;
const TEXT_GAP: i32 = 12;
const BUTTON_GAP: i32 = 8;
const BUTTON_PAD: i32 = 26;
const BUTTON_HEIGHT: i32 = 42;
const ROW: i32 = 54;
const STACKED_ROW: i32 = 40;
const INDICATOR_BAD: Rgba = Rgba::hex(0xff, 0x45, 0x3a);

pub struct NotificationLayer {
    pub widget: gtk4::Box,
    service: Rc<NotificationService>,
}

impl NotificationLayer {
    pub fn new(service: Rc<NotificationService>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        widget.set_valign(gtk4::Align::Start);
        Self { widget, service }
    }

    pub fn set_toast(&self, toast: Option<&Toast>) {
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child);
        }
        let Some(toast) = toast else { return };
        let critical = toast.urgency == Urgency::Critical;
        let buttons: Vec<(String, String)> = toast.buttons().into_iter().cloned().collect();
        let inline = buttons.len() <= 2
            && buttons.iter().map(|(_, text)| text.chars().count()).sum::<usize>() <= 20;

        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, TEXT_GAP);
        row.set_height_request(ROW);
        row.append(&self.face(toast, critical));

        let text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        text.set_hexpand(true);
        text.set_valign(gtk4::Align::Center);
        text.append(&Self::title_row(toast, !(inline && !buttons.is_empty())));
        if !toast.body.is_empty() {
            text.append(&Self::body(&toast.body));
        }
        row.append(&text);

        if inline && !buttons.is_empty() {
            let actions = gtk4::Box::new(gtk4::Orientation::Horizontal, BUTTON_GAP);
            actions.set_valign(gtk4::Align::Center);
            for (index, (key, label)) in buttons.iter().enumerate() {
                let (service, id, key) = (Rc::clone(&self.service), toast.id, key.clone());
                actions.append(&Self::pill(label, index == 0, move || service.invoke_action(id, &key)));
            }
            row.append(&actions);
        }

        let service = Rc::clone(&self.service);
        let default = toast.default_action().map(|(key, _)| key.clone());
        let id = toast.id;
        let click = gtk4::GestureClick::new();
        click.set_button(0);
        click.connect_released(move |gesture, _, _, _| match (gesture.current_button(), &default) {
            (1, Some(key)) => service.invoke_action(id, key),
            _ => service.close(),
        });
        row.add_controller(click);
        row.set_cursor_from_name(Some("pointer"));
        self.widget.append(&row);

        if !inline {
            self.widget.append(&self.stacked(toast.id, &buttons));
        }
    }

    fn face(&self, toast: &Toast, critical: bool) -> Capsule {
        let face = Capsule::new();
        face.set_size(PICTURE, PICTURE);
        face.set_radii([f32::from(PICTURE as i16) / 2.0; 4]);
        face.set_valign(gtk4::Align::Center);
        face.set_color(palette::current().surface_hover.gdk());
        if critical {
            face.set_border(2.0, INDICATOR_BAD.gdk());
        }

        let icon = toast.icon.as_str();
        let theme = gtk4::gdk::Display::default().map(|d| gtk4::IconTheme::for_display(&d));
        let has = |name: &str| theme.as_ref().is_some_and(|t| !name.is_empty() && t.has_icon(name));
        let stand_in = if critical { "com.github.timecraft.notifier" } else { "cs-notifications" };

        let content: gtk4::Widget = if icon.starts_with('/') && std::path::Path::new(icon).exists() {
            let picture = gtk4::Picture::for_filename(icon);
            picture.set_content_fit(gtk4::ContentFit::Cover);
            picture.upcast()
        } else if has(icon) || has(stand_in) {
            let image = gtk4::Image::from_icon_name(if has(icon) { icon } else { stand_in });
            image.set_pixel_size((f64::from(PICTURE) * 0.62).round() as i32);
            image.set_halign(gtk4::Align::Center);
            image.set_valign(gtk4::Align::Center);
            image.upcast()
        } else {
            let glyph = gtk4::Label::new(Some(if critical { "\u{f002a}" } else { "\u{f009a}" }));
            glyph.add_css_class(if critical { "toast-glyph-bad" } else { "toast-glyph" });
            glyph.upcast()
        };
        face.set_content(Some(&content));
        face
    }

    fn title_row(toast: &Toast, show_meta: bool) -> gtk4::Overlay {
        let overlay = gtk4::Overlay::new();
        let title = gtk4::Label::new(Some(&toast.summary));
        title.add_css_class("toast-title");
        title.set_xalign(0.0);
        title.set_ellipsize(pango::EllipsizeMode::End);
        overlay.set_child(Some(&title));
        if show_meta {
            let parts: Vec<&str> = [toast.app_name.as_str(), "now"].into_iter().filter(|p| !p.is_empty()).collect();
            let meta = gtk4::Label::new(Some(&parts.join("  ·  ")));
            meta.add_css_class("toast-meta");
            meta.set_halign(gtk4::Align::End);
            meta.set_valign(gtk4::Align::Baseline);
            let (_, natural, _, _) = meta.measure(gtk4::Orientation::Horizontal, -1);
            title.set_margin_end(natural + 10);
            overlay.add_overlay(&meta);
        }
        overlay
    }

    /// Applications send Pango markup; invalid markup is shown as plain text.
    fn body(text: &str) -> gtk4::Label {
        let label = gtk4::Label::new(None);
        if pango::parse_markup(text, '\0').is_ok() {
            label.set_markup(text);
        } else {
            label.set_text(text);
        }
        label.add_css_class("toast-body");
        label.set_xalign(0.0);
        label.set_wrap(true);
        label.set_lines(2);
        label.set_ellipsize(pango::EllipsizeMode::End);
        label
    }

    fn pill(text: &str, first: bool, on_click: impl Fn() + 'static) -> Capsule {
        let label = gtk4::Label::new(Some(text));
        label.add_css_class(if first { "toast-button-first" } else { "toast-button" });
        let (_, natural, _, _) = label.measure(gtk4::Orientation::Horizontal, -1);
        let pill = Capsule::new();
        pill.set_size(natural + BUTTON_PAD, BUTTON_HEIGHT);
        pill.set_radii([BUTTON_HEIGHT as f32 / 2.0; 4]);
        pill.set_content(Some(&label));
        pill.set_cursor_from_name(Some("pointer"));

        let paint = {
            let pill = pill.clone();
            move |mix: f64| {
                let p = palette::current();
                let (from, to) = if first { (p.accent, p.accent_hover) } else { (p.surface_hover, p.border) };
                pill.set_color(from.mix(to, mix as f32).gdk());
            }
        };
        paint(0.0);
        let hover = Tween::new(&pill, 0.0, paint);
        let motion = gtk4::EventControllerMotion::new();
        let enter = hover.clone();
        motion.connect_enter(move |_, _, _| enter.animate(1.0, tokens::DURATION_FAST, Curve::OutCubic));
        let leave = hover.clone();
        motion.connect_leave(move |_| leave.animate(0.0, tokens::DURATION_FAST, Curve::OutCubic));
        pill.add_controller(motion);

        // Under the row's own gesture: the button keeps its click.
        let click = gtk4::GestureClick::new();
        click.connect_pressed(|gesture, _, _, _| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);
        });
        click.connect_released(move |_, _, _, _| on_click());
        pill.add_controller(click);
        pill
    }

    /// More or longer buttons, sharing the width under the row, split by hairlines.
    fn stacked(&self, id: u32, buttons: &[(String, String)]) -> gtk4::Box {
        let block = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        block.set_height_request(STACKED_ROW);
        let line = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        line.set_height_request(1);
        line.set_margin_top(6);
        line.add_css_class("toast-hairline");
        block.append(&line);

        let cells = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        cells.set_homogeneous(true);
        cells.set_vexpand(true);
        for (index, (key, label)) in buttons.iter().enumerate() {
            let cell = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            if index > 0 {
                let divider = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                divider.set_width_request(1);
                divider.set_margin_top(6);
                divider.set_margin_bottom(6);
                divider.add_css_class("toast-hairline");
                cell.append(&divider);
            }
            let text = gtk4::Label::new(Some(label));
            text.add_css_class(if index == 0 { "toast-cell-first" } else { "toast-cell" });
            text.set_ellipsize(pango::EllipsizeMode::End);
            text.set_hexpand(true);
            cell.append(&text);
            let (service, key) = (Rc::clone(&self.service), key.clone());
            let click = gtk4::GestureClick::new();
            click.connect_pressed(|gesture, _, _, _| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);
        });
            click.connect_released(move |_, _, _, _| service.invoke_action(id, &key));
            cell.add_controller(click);
            cell.set_cursor_from_name(Some("pointer"));
            cells.append(&cell);
        }
        block.append(&cells);
        block
    }
}
