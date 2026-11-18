//! `LauncherPanel.qml`: the field, the rule under it, and the results. The
//! glyph is the mode the query resolves to. Up and Down move the selection
//! (wrapping), Enter runs it; Escape is handled by the bar for every panel.
//! Results exist for applications so far; the other modes say so.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::launcher_results::ResultList;
use super::launcher_store::{Outcome, Store};
use crate::services::launcher;

pub const WIDTH: i32 = 560;
pub const HEIGHT: i32 = 520;
const GAP: i32 = 12;
const FIELD_HEIGHT: i32 = 34;

pub struct LauncherPanel {
    pub widget: gtk4::Box,
    field: gtk4::Text,
    refresh: Rc<dyn Fn()>,
}

impl LauncherPanel {
    /// `close` is called after a result has run.
    pub fn new(store: Rc<RefCell<Store>>, close: Rc<dyn Fn()>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, GAP);
        widget.set_vexpand(true);

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

        // An explicit empty state; in a sigil mode usually only the sigil has
        // been typed so far.
        let empty = gtk4::Label::new(None);
        empty.add_css_class("launcher-empty");
        empty.set_xalign(0.0);
        empty.set_visible(false);
        widget.append(&empty);

        let results = ResultList::new({
            let (store, close) = (Rc::clone(&store), Rc::clone(&close));
            move |entry| {
                store.borrow_mut().launch(entry);
                close();
            }
        });
        widget.append(&results.widget);

        let refresh = {
            let (field, glyph, empty, results, store_for_refresh) = (
                field.clone(),
                glyph.clone(),
                empty.clone(),
                Rc::clone(&results),
                Rc::clone(&store),
            );
            move || {
                let query = field.text().to_string();
                let mode = launcher::mode_for(&query);
                glyph.set_text(mode.icon);
                match store_for_refresh.borrow().search(&query) {
                    Outcome::Entries(entries) => {
                        let none = entries.is_empty();
                        results.set_entries(entries);
                        empty.set_text(mode.empty);
                        empty.set_visible(none && !query.is_empty());
                    }
                    Outcome::Unported(label) => {
                        results.set_entries(Vec::new());
                        empty.set_text(&format!("{label} is not ported yet"));
                        empty.set_visible(true);
                    }
                }
            }
        };
        let refresh: Rc<dyn Fn()> = Rc::new(refresh);
        refresh();
        field.connect_changed({
            let refresh = Rc::clone(&refresh);
            move |_| refresh()
        });

        // Keys the field would otherwise keep for itself.
        let keys = gtk4::EventControllerKey::new();
        keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
        keys.connect_key_pressed({
            let (results, store, refresh, close) = (
                Rc::clone(&results),
                Rc::clone(&store),
                Rc::clone(&refresh),
                Rc::clone(&close),
            );
            move |_, key, _, state| {
                use gtk4::gdk::ModifierType;
                match key {
                    gtk4::gdk::Key::Down => {
                        results.step(1);
                        gtk4::glib::Propagation::Stop
                    }
                    gtk4::gdk::Key::Up => {
                        results.step(-1);
                        gtk4::glib::Propagation::Stop
                    }
                    gtk4::gdk::Key::Delete if state.contains(ModifierType::SHIFT_MASK) => {
                        if let Some(entry) = results
                            .selected_entry()
                            .filter(|entry| entry.kind == "clip")
                        {
                            store.borrow().forget_clip(&entry.id);
                            refresh();
                        }
                        gtk4::glib::Propagation::Stop
                    }
                    gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter
                        if state
                            .intersects(ModifierType::SHIFT_MASK | ModifierType::CONTROL_MASK) =>
                    {
                        if let Some(entry) = results
                            .selected_entry()
                            .filter(|entry| entry.kind == "clip" && entry.file.is_some())
                        {
                            store.borrow().open_clip_image(&entry);
                            close();
                            gtk4::glib::Propagation::Stop
                        } else {
                            gtk4::glib::Propagation::Proceed
                        }
                    }
                    _ => gtk4::glib::Propagation::Proceed,
                }
            }
        });
        field.add_controller(keys);
        field.connect_activate(move |_| results.run_selected());

        Self {
            widget,
            field,
            refresh,
        }
    }

    pub fn refresh(&self) {
        (self.refresh)();
    }

    /// The window only receives keys once the field holds focus.
    pub fn focus(&self) {
        self.field.grab_focus();
    }

    pub fn query(&self) -> String {
        self.field.text().to_string()
    }
}
