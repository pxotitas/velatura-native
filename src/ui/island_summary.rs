//! Faithful GTK rendering of the Quickshell IslandSummary media/clock face.

use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use std::time::Duration;

use crate::services::media::{self, MediaSnapshot};

const WIDTH: i32 = 620;
const HEIGHT: i32 = 150;
const SQUARE: i32 = 112;

fn label(text: &str, class: &str) -> gtk4::Label {
    let label = gtk4::Label::new(Some(text));
    label.add_css_class(class);
    label.set_xalign(0.0);
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label
}

fn art_file(url: &str) -> Option<gio::File> {
    if url.is_empty() {
        return None;
    }
    if url.starts_with("file://") {
        Some(gio::File::for_uri(url))
    } else if url.starts_with('/') {
        Some(gio::File::for_path(url))
    } else {
        None
    }
}

fn update_art(picture: &gtk4::Picture, fallback: &gtk4::Label, url: &str, current: &mut String) {
    if *current == url {
        return;
    }
    *current = url.to_string();
    if let Some(file) = art_file(url) {
        picture.set_file(Some(&file));
        picture.set_visible(true);
        fallback.set_visible(false);
    } else {
        picture.set_file(None::<&gio::File>);
        picture.set_visible(false);
        fallback.set_visible(true);
    }
}

fn refresh_clock(time: &gtk4::Label, weekday: &gtk4::Label, date: &gtk4::Label) {
    if let Ok(now) = glib::DateTime::now_local() {
        time.set_text(&now.format("%H:%M").unwrap_or_default());
        weekday.set_text(&now.format("%A").unwrap_or_default().to_uppercase());
        date.set_text(&now.format("%-d %B").unwrap_or_default().to_uppercase());
    }
}

fn build_week() -> gtk4::Box {
    let week = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    week.set_hexpand(true);
    week.set_valign(gtk4::Align::Center);
    if let Ok(today) = glib::DateTime::now_local() {
        for offset in -2..=2 {
            let day = today.add_days(offset).unwrap_or_else(|_| today.clone());
            let column = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            column.set_hexpand(true);
            column.set_halign(gtk4::Align::Center);
            column.set_valign(gtk4::Align::Center);
            let name = gtk4::Label::new(Some(
                &day.format("%a")
                    .unwrap_or_default()
                    .replace('.', "")
                    .to_uppercase(),
            ));
            name.add_css_class(if offset == 0 {
                "summary-day-name-today"
            } else {
                "summary-day-name"
            });
            let number = gtk4::Label::new(Some(&day.format("%-d").unwrap_or_default()));
            number.add_css_class(if offset == 0 {
                "summary-day-number-today"
            } else {
                "summary-day-number"
            });
            column.append(&name);
            column.append(&number);
            week.append(&column);
        }
    }
    week
}

#[derive(Clone)]
struct SummaryWidgets {
    artwork: gtk4::Overlay,
    details: gtk4::Box,
    week: gtk4::Box,
    title: gtk4::Label,
    artist: gtk4::Label,
    play: gtk4::Button,
    picture: gtk4::Picture,
    fallback: gtk4::Label,
    weekday: gtk4::Label,
    date: gtk4::Label,
}

fn apply_snapshot(snapshot: Option<MediaSnapshot>, ui: &SummaryWidgets, art_url: &mut String) {
    if let Some(snapshot) = snapshot {
        ui.artwork.set_visible(true);
        ui.details.set_visible(true);
        ui.week.set_visible(false);
        ui.weekday.set_visible(true);
        ui.date.set_visible(true);
        ui.title.set_text(snapshot.identity());
        ui.artist.set_text(&snapshot.artist);
        ui.play
            .set_label(if snapshot.playing() { "󰏤" } else { "󰐊" });
        update_art(&ui.picture, &ui.fallback, &snapshot.art_url, art_url);
    } else {
        ui.artwork.set_visible(false);
        ui.details.set_visible(false);
        ui.week.set_visible(true);
        ui.weekday.set_visible(false);
        ui.date.set_visible(false);
        update_art(&ui.picture, &ui.fallback, "", art_url);
    }
}

pub fn build() -> gtk4::Box {
    let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    root.set_size_request(WIDTH, HEIGHT);
    root.set_hexpand(true);
    root.set_valign(gtk4::Align::Center);
    root.set_margin_start(19);
    root.set_margin_end(23);
    root.set_margin_top(19);
    root.set_margin_bottom(19);

    let artwork = gtk4::Overlay::new();
    artwork.set_size_request(SQUARE, SQUARE);
    artwork.set_overflow(gtk4::Overflow::Hidden);
    artwork.add_css_class("summary-art");
    let picture = gtk4::Picture::new();
    picture.set_size_request(SQUARE, SQUARE);
    picture.set_can_shrink(true);
    picture.set_content_fit(gtk4::ContentFit::Cover);
    picture.set_visible(false);
    artwork.set_child(Some(&picture));
    let fallback = gtk4::Label::new(Some("󰎇"));
    fallback.add_css_class("summary-art-fallback");
    artwork.add_overlay(&fallback);
    root.append(&artwork);

    let details = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    details.set_hexpand(true);
    details.set_valign(gtk4::Align::Center);
    details.set_halign(gtk4::Align::Fill);
    let title = label("", "summary-title");
    title.set_max_width_chars(28);
    let artist = label("", "summary-artist");
    artist.set_max_width_chars(28);
    details.append(&title);
    details.append(&artist);
    let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    details.append(&spacer);

    let controls = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    controls.set_valign(gtk4::Align::Center);
    let previous = gtk4::Button::with_label("󰒮");
    let play = gtk4::Button::with_label("󰏤");
    let next = gtk4::Button::with_label("󰒭");
    for (button, class) in [
        (&previous, "summary-control"),
        (&play, "summary-control summary-control-primary"),
        (&next, "summary-control"),
    ] {
        button.add_css_class("flat");
        for class in class.split_whitespace() {
            button.add_css_class(class);
        }
    }
    previous.connect_clicked(|_| media::previous());
    play.connect_clicked(|_| media::toggle());
    next.connect_clicked(|_| media::next());
    controls.append(&previous);
    controls.append(&play);
    controls.append(&next);
    details.append(&controls);
    root.append(&details);

    let week = build_week();
    root.append(&week);

    let clock = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    clock.set_valign(gtk4::Align::Center);
    clock.set_halign(gtk4::Align::End);
    let time = gtk4::Label::new(None);
    time.add_css_class("summary-clock");
    time.set_xalign(1.0);
    let weekday = gtk4::Label::new(None);
    weekday.add_css_class("summary-weekday");
    weekday.set_xalign(1.0);
    let date = gtk4::Label::new(None);
    date.add_css_class("summary-date");
    date.set_xalign(1.0);
    clock.append(&time);
    clock.append(&weekday);
    clock.append(&date);
    root.append(&clock);

    refresh_clock(&time, &weekday, &date);
    let ui = SummaryWidgets {
        artwork,
        details,
        week,
        title,
        artist,
        play,
        picture,
        fallback,
        weekday: weekday.clone(),
        date: date.clone(),
    };
    let mut current_art = String::new();
    apply_snapshot(media::snapshot(), &ui, &mut current_art);

    let weak_root = root.downgrade();
    let ui_tick = ui.clone();
    let (time_tick, weekday_tick, date_tick) = (time.clone(), weekday.clone(), date.clone());
    glib::timeout_add_local(Duration::from_millis(500), move || {
        if weak_root.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        apply_snapshot(media::snapshot(), &ui_tick, &mut current_art);
        refresh_clock(&time_tick, &weekday_tick, &date_tick);
        glib::ControlFlow::Continue
    });
    root
}
