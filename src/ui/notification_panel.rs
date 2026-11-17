//! Notifications module chip and its history detail (`NotificationsModule.qml`).

use gtk4::prelude::*;
use gtk4::{gdk, pango};
use std::rc::Rc;

use super::capsule::Capsule;
use super::settings::Settings;
use super::tokens;
use crate::services::notifications::{HistoryEntry, NotificationService};

const ROW_HEIGHT: i32 = 22;
const ROW_GAP: i32 = 8;
const CARD_PADDING: i32 = 16;
const CARD_INSET: i32 = 4;
const CARD_MARK: i32 = 44;
const CARD_GAP: i32 = 14;
const CARD_WIDTH: i32 = 380;

pub struct NotificationChip {
    pub widget: Capsule,
    service: Rc<NotificationService>,
    icon: gtk4::Label,
    label: gtk4::Label,
    height: i32,
}

impl NotificationChip {
    pub fn new(
        settings: &Settings,
        service: Rc<NotificationService>,
        open: impl Fn() + 'static,
    ) -> Self {
        let widget = Capsule::new();
        let height = settings.bar_height;
        widget.set_radii([height as f32 / 2.0; 4]);
        widget.set_color(tokens::ground_of(&settings.surface_style).gdk());
        widget.set_border(1.0, tokens::rim_of(&settings.surface_style).gdk());

        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 5);
        let icon = gtk4::Label::new(None);
        icon.add_css_class("chip-glyph");
        icon.set_text("󰂚");
        let label = gtk4::Label::new(None);
        label.add_css_class("notification-chip-value");
        row.append(&icon);
        row.append(&label);
        widget.set_content(Some(&row));
        widget.set_insets(0, 9, 9, 0);
        widget.set_cursor_from_name(Some("pointer"));

        let click = gtk4::GestureClick::new();
        click.connect_released(move |_, _, _, _| open());
        widget.add_controller(click);

        let chip = Self {
            widget,
            service,
            icon,
            label,
            height,
        };
        chip.refresh();
        chip
    }

    pub fn refresh(&self) {
        let count = self.service.history().len();
        self.icon.set_text(if self.service.do_not_disturb() {
            "󰂛"
        } else {
            "󰂚"
        });
        self.label.set_text(&count.to_string());
        let (_, natural, _, _) = self.label.measure(gtk4::Orientation::Horizontal, -1);
        let icon = (f64::from(self.height) * 0.44).round() as i32;
        let width = 2 * 9 + icon + 5 + natural + 2;
        self.widget.set_size(width.max(self.height), self.height);
    }
}

pub struct NotificationHistoryPanel {
    pub widget: gtk4::Box,
    service: Rc<NotificationService>,
    title_subtitle: gtk4::Label,
    mark: gtk4::Label,
    list: gtk4::Box,
    list_scroll: gtk4::ScrolledWindow,
    list_slot: gtk4::Box,
    dnd: gtk4::Button,
    clear: gtk4::Button,
    on_open: Rc<dyn Fn()>,
}

impl NotificationHistoryPanel {
    pub fn new(service: Rc<NotificationService>, on_open: impl Fn() + 'static) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, CARD_GAP);
        widget.add_css_class("notification-history-card");
        widget.set_size_request(CARD_WIDTH, Self::height(0));

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        header.set_height_request(CARD_MARK);
        let mark = gtk4::Label::new(None);
        mark.add_css_class("notification-history-mark");
        mark.set_size_request(CARD_MARK, CARD_MARK);
        header.append(&mark);

        let title_subtitle = gtk4::Label::new(None);
        title_subtitle.add_css_class("notification-history-heading");
        title_subtitle.set_xalign(0.0);
        title_subtitle.set_hexpand(true);
        title_subtitle.set_ellipsize(pango::EllipsizeMode::End);
        header.append(&title_subtitle);
        widget.append(&header);

        let list_slot = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let list_scroll = gtk4::ScrolledWindow::new();
        list_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        list_scroll.set_propagate_natural_height(false);
        list_scroll.add_css_class("notification-history-list");
        let list = gtk4::Box::new(gtk4::Orientation::Vertical, ROW_GAP);
        list_scroll.set_child(Some(&list));
        list_slot.append(&list_scroll);
        widget.append(&list_slot);

        let footer = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        footer.set_height_request(ROW_HEIGHT);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        footer.append(&spacer);
        let dnd = gtk4::Button::with_label("Do Not Disturb");
        dnd.add_css_class("notification-history-button");
        footer.append(&dnd);
        let clear = gtk4::Button::with_label("Clear");
        clear.add_css_class("notification-history-button");
        footer.append(&clear);
        widget.append(&footer);

        let panel = Self {
            widget,
            service,
            title_subtitle,
            mark,
            list,
            list_scroll,
            list_slot,
            dnd,
            clear,
            on_open: Rc::new(on_open),
        };
        panel.connect_actions();
        panel.refresh();
        panel
    }

    fn connect_actions(&self) {
        let service = Rc::clone(&self.service);
        self.dnd.connect_clicked(move |_| {
            service.set_do_not_disturb(!service.do_not_disturb());
        });
        let service = Rc::clone(&self.service);
        self.clear.connect_clicked(move |_| service.clear_history());
    }

    pub fn refresh(&self) {
        let history = self.service.history();
        let visible = history.len().min(6);
        let subtitle = if history.is_empty() {
            "Nothing new".to_string()
        } else {
            format!("{} kept", history.len())
        };
        let subtitle = if self.service.do_not_disturb() {
            format!("{subtitle} · Do Not Disturb")
        } else {
            subtitle
        };
        self.title_subtitle.set_markup(&format!(
            "<b>Notifications</b>\n<span size=\"small\">{}</span>",
            glib_escape(&subtitle)
        ));
        self.mark.set_text(if self.service.do_not_disturb() {
            "󰂛"
        } else {
            "󰂚"
        });
        self.dnd.set_css_classes(if self.service.do_not_disturb() {
            &["notification-history-button", "active"]
        } else {
            &["notification-history-button"]
        });
        self.clear.set_sensitive(!history.is_empty());

        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        for entry in history {
            self.list.append(&self.row(entry));
        }

        let list_height = if visible == 0 {
            0
        } else {
            visible as i32 * ROW_HEIGHT + (visible as i32 - 1) * ROW_GAP
        };
        self.list_slot.set_visible(visible > 0);
        self.list_scroll.set_min_content_height(list_height);
        self.list_scroll.set_max_content_height(list_height);
        self.widget
            .set_size_request(CARD_WIDTH, Self::height(visible));
    }

    pub fn count(&self) -> usize {
        self.service.history().len()
    }

    fn row(&self, entry: HistoryEntry) -> gtk4::Widget {
        let current = self.service.current();
        let action = current
            .as_ref()
            .filter(|toast| entry.live && toast.id == entry.id)
            .and_then(|toast| toast.default_action().map(|(key, _)| key.clone()));

        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        row.add_css_class("notification-history-row");
        row.set_size_request(-1, ROW_HEIGHT);

        let icon = self.history_icon(&entry);
        icon.set_size_request(16, 16);
        row.append(&icon);

        let summary = gtk4::Label::new(Some(&entry.summary));
        summary.add_css_class("notification-history-summary");
        summary.set_width_chars(1);
        summary.set_max_width_chars(18);
        summary.set_ellipsize(pango::EllipsizeMode::End);
        row.append(&summary);

        let body = gtk4::Label::new(None);
        body.add_css_class("notification-history-body");
        body.set_width_chars(1);
        body.set_hexpand(true);
        body.set_ellipsize(pango::EllipsizeMode::End);
        if pango::parse_markup(&entry.body, '\0').is_ok() {
            body.set_markup(&entry.body);
        } else {
            body.set_text(&entry.body);
        }
        row.append(&body);

        let app = gtk4::Label::new(Some(&entry.app_name));
        app.add_css_class("notification-history-app");
        app.set_max_width_chars(10);
        app.set_ellipsize(pango::EllipsizeMode::End);
        row.append(&app);

        let remove = gtk4::Button::with_label("󰅖");
        remove.add_css_class("notification-history-remove");
        remove.set_tooltip_text(Some("Remove from history"));
        remove.set_visible(false);
        let service = Rc::clone(&self.service);
        let id = entry.id;
        remove.connect_clicked(move |_| service.remove_from_history(id));
        row.append(&remove);

        let app_on = app.clone();
        let remove_on = remove.clone();
        let motion = gtk4::EventControllerMotion::new();
        motion.connect_enter(move |_, _, _| {
            app_on.set_visible(false);
            remove_on.set_visible(true);
        });
        let app_off = app.clone();
        let remove_off = remove.clone();
        motion.connect_leave(move |_| {
            app_off.set_visible(true);
            remove_off.set_visible(false);
        });
        row.add_controller(motion);

        if let Some(key) = action {
            row.add_css_class("notification-history-open");
            row.set_cursor_from_name(Some("pointer"));
            let service = Rc::clone(&self.service);
            let on_open = Rc::clone(&self.on_open);
            let id = entry.id;
            let click = gtk4::GestureClick::new();
            click.connect_released(move |gesture, _, _, _| {
                if gesture.current_button() != 1 {
                    return;
                }
                service.invoke_action(id, &key);
                on_open();
            });
            row.add_controller(click);
        }
        row.upcast()
    }

    fn history_icon(&self, entry: &HistoryEntry) -> gtk4::Widget {
        let critical = entry.urgency == 2;
        let display = gdk::Display::default();
        let theme = display.as_ref().map(gtk4::IconTheme::for_display);
        let has_icon = |name: &str| {
            theme
                .as_ref()
                .is_some_and(|theme| !name.is_empty() && theme.has_icon(name))
        };
        if entry.icon.starts_with('/') && std::path::Path::new(&entry.icon).exists() {
            let picture = gtk4::Picture::for_filename(&entry.icon);
            picture.set_content_fit(gtk4::ContentFit::Cover);
            picture.upcast()
        } else if has_icon(&entry.icon) {
            let image = gtk4::Image::from_icon_name(&entry.icon);
            image.set_pixel_size(16);
            image.upcast()
        } else {
            let glyph = gtk4::Label::new(Some(if critical { "󰀪" } else { "󰂚" }));
            glyph.add_css_class(if critical {
                "notification-history-critical"
            } else {
                "notification-history-glyph"
            });
            glyph.upcast()
        }
    }

    pub fn height(shown: usize) -> i32 {
        let rows = shown as i32 + 1;
        2 * (CARD_INSET + CARD_PADDING)
            + CARD_MARK
            + CARD_GAP
            + rows * ROW_HEIGHT
            + (rows - 1) * ROW_GAP
    }
}

fn glib_escape(value: &str) -> String {
    gtk4::glib::markup_escape_text(value).to_string()
}
