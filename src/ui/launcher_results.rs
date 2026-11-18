//! The launcher's list of rows (`LauncherPanel.qml`'s `resultList`): an icon,
//! a name and a subtitle, one selection shared by pointer and keyboard, and
//! wrap-around at both ends.

use gtk4::pango;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::capsule::Capsule;
use super::launcher_store::Entry;
use super::motion::{Curve, Tween};
use super::palette;
use super::tokens;

const ROW_HEIGHT: i32 = 48;
const ROW_SPACING: i32 = 2;
const ROW_WIDTH: i32 = 520;
const ROW_RADIUS: f32 = 10.0;
const ICON: i32 = 26;

/// Icons some applications ask for under another name (`Theme.appIconAliases`).
fn icon_alias(name: &str) -> &str {
    match name {
        "vesktop" => "Vesktop",
        "org.gnome.Nautilus" | "thunar" | "org.xfce.thunar" => "impasto-files",
        other => other,
    }
}

struct Row {
    capsule: Capsule,
    highlight: Tween,
}

pub struct ResultList {
    pub widget: gtk4::ScrolledWindow,
    column: gtk4::Box,
    rows: RefCell<Vec<Row>>,
    entries: RefCell<Vec<Entry>>,
    selected: Cell<usize>,
    run: Rc<dyn Fn(&Entry)>,
}

impl ResultList {
    pub fn new(run: impl Fn(&Entry) + 'static) -> Rc<Self> {
        let column = gtk4::Box::new(gtk4::Orientation::Vertical, ROW_SPACING);
        let widget = gtk4::ScrolledWindow::new();
        widget.set_child(Some(&column));
        widget.set_vexpand(true);
        widget.set_hscrollbar_policy(gtk4::PolicyType::Never);
        // Scrolls, but draws no bar: the original's list has none.
        widget.set_vscrollbar_policy(gtk4::PolicyType::External);
        Rc::new(Self {
            widget,
            column,
            rows: RefCell::new(Vec::new()),
            entries: RefCell::new(Vec::new()),
            selected: Cell::new(0),
            run: Rc::new(run),
        })
    }

    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.borrow().is_empty()
    }

    /// Replaces the rows; the selection goes back to the top, so Enter always
    /// runs the first result.
    pub fn set_entries(self: &Rc<Self>, entries: Vec<Entry>) {
        while let Some(child) = self.column.first_child() {
            self.column.remove(&child);
        }
        let mut rows = Vec::with_capacity(entries.len());
        for (index, entry) in entries.iter().enumerate() {
            let row = self.build_row(entry, index);
            self.column.append(&row.capsule);
            rows.push(row);
        }
        *self.rows.borrow_mut() = rows;
        *self.entries.borrow_mut() = entries;
        self.selected.set(0);
        self.paint_selection(false);
        self.widget.vadjustment().set_value(0.0);
    }

    fn paint_selection(&self, animated: bool) {
        let duration = if animated { tokens::DURATION_FAST } else { 0.0 };
        for (index, row) in self.rows.borrow().iter().enumerate() {
            let target = f64::from(u8::from(index == self.selected.get()));
            row.highlight.animate(target, duration, Curve::OutCubic);
        }
    }

    fn select(&self, index: usize) {
        if self.selected.replace(index) != index {
            self.paint_selection(true);
        }
    }

    /// The list wraps around at both ends.
    pub fn step(&self, delta: i32) {
        let count = self.len() as i32;
        if count == 0 {
            return;
        }
        let next = (self.selected.get() as i32 + delta).rem_euclid(count) as usize;
        self.select(next);
        self.scroll_to(next);
    }

    pub fn selected_entry(&self) -> Option<Entry> {
        self.entries.borrow().get(self.selected.get()).cloned()
    }

    pub fn run_selected(&self) {
        if let Some(entry) = self.selected_entry() {
            (self.run)(&entry);
        }
    }

    fn scroll_to(&self, index: usize) {
        let adjustment = self.widget.vadjustment();
        let top = (index as i32 * (ROW_HEIGHT + ROW_SPACING)) as f64;
        let bottom = top + f64::from(ROW_HEIGHT);
        if top < adjustment.value() {
            adjustment.set_value(top);
        } else if bottom > adjustment.value() + adjustment.page_size() {
            adjustment.set_value(bottom - adjustment.page_size());
        }
    }

    fn build_row(self: &Rc<Self>, entry: &Entry, index: usize) -> Row {
        let capsule = Capsule::new();
        capsule.set_size(ROW_WIDTH, ROW_HEIGHT);
        capsule.set_radii([ROW_RADIUS; 4]);
        capsule.set_insets(0, 10, 12, 0);
        capsule.set_cursor_from_name(Some("pointer"));

        let paint = {
            let capsule = capsule.clone();
            move |mix: f64| {
                let mut color = palette::current().surface_hover;
                color.3 = mix as f32;
                capsule.set_color(color.gdk());
            }
        };
        paint(0.0);
        let highlight = Tween::new(&capsule, 0.0, paint);

        let line = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        line.append(&Self::badge(entry));

        let text = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
        text.set_hexpand(true);
        text.set_valign(gtk4::Align::Center);
        let name = gtk4::Label::new(Some(&entry.name));
        name.add_css_class("launcher-row-name");
        name.set_xalign(0.0);
        name.set_ellipsize(pango::EllipsizeMode::End);
        text.append(&name);
        if !entry.subtitle.is_empty() {
            let subtitle = gtk4::Label::new(Some(&entry.subtitle));
            subtitle.add_css_class("launcher-row-subtitle");
            subtitle.set_xalign(0.0);
            subtitle.set_ellipsize(pango::EllipsizeMode::End);
            text.append(&subtitle);
        }
        line.append(&text);
        if entry.favourite {
            let pin = gtk4::Label::new(Some("\u{f10a9}"));
            pin.add_css_class("launcher-row-pin");
            pin.set_valign(gtk4::Align::Center);
            line.append(&pin);
        }
        capsule.set_content(Some(&line));

        // On movement, not hover: a row appearing under a resting pointer
        // would otherwise steal the selection on open.
        let motion = gtk4::EventControllerMotion::new();
        let list = Rc::downgrade(self);
        motion.connect_motion(move |_, _, _| {
            if let Some(list) = list.upgrade() {
                list.select(index);
            }
        });
        capsule.add_controller(motion);

        let click = gtk4::GestureClick::new();
        let list = Rc::downgrade(self);
        click.connect_released(move |_, _, _, _| {
            if let Some(list) = list.upgrade() {
                list.select(index);
                list.run_selected();
            }
        });
        capsule.add_controller(click);
        Row { capsule, highlight }
    }

    /// The themed icon, or a glyph in a tinted disc when there is none.
    fn badge(entry: &Entry) -> gtk4::Widget {
        if entry.kind == "clip" {
            if let Some(file) = entry.file.as_deref() {
                let path = crate::services::clipboard::payload_path(file);
                if !path.as_os_str().is_empty() {
                    let image = gtk4::Image::from_file(path);
                    image.set_pixel_size(ICON);
                    image.set_valign(gtk4::Align::Center);
                    image.add_css_class("launcher-clip-thumb");
                    return image.upcast();
                }
            }
        }
        let theme = gtk4::gdk::Display::default().map(|d| gtk4::IconTheme::for_display(&d));
        let name = icon_alias(&entry.icon);
        let has = theme
            .as_ref()
            .is_some_and(|t| !name.is_empty() && t.has_icon(name));
        if has {
            let image = gtk4::Image::from_icon_name(name);
            image.set_pixel_size(ICON);
            image.set_valign(gtk4::Align::Center);
            return image.upcast();
        }
        if name.starts_with('/') && std::path::Path::new(name).exists() {
            let image = gtk4::Image::from_file(name);
            image.set_pixel_size(ICON);
            image.set_valign(gtk4::Align::Center);
            return image.upcast();
        }
        let disc = Capsule::new();
        disc.set_size(ICON, ICON);
        disc.set_radii([ICON as f32 / 2.0; 4]);
        disc.set_valign(gtk4::Align::Center);
        disc.set_color(palette::current().surface_hover.gdk());
        let glyph = gtk4::Label::new(Some("\u{f0039}"));
        glyph.add_css_class("launcher-row-glyph");
        disc.set_content(Some(&glyph));
        disc.upcast()
    }
}
