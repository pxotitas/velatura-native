//! GTK port of Quickshell IslandRest: centred clock with activity marks on
//! the two sides. One running media activity is split across both sides.

use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use std::time::Duration;

use super::clock;
use super::settings::Settings;
use crate::services::media;

const SIDE: i32 = 64;

fn clock_core(settings: &Settings) -> i32 {
    if settings.clock_shows_date {
        150
    } else if settings.clock_shows_seconds {
        88
    } else {
        72
    }
}

fn file_for_art(url: &str) -> Option<gio::File> {
    if url.starts_with("file://") {
        Some(gio::File::for_uri(url))
    } else if url.starts_with('/') {
        Some(gio::File::for_path(url))
    } else {
        None
    }
}

pub fn width(settings: &Settings, media_active: bool) -> i32 {
    if media_active {
        clock_core(settings) + SIDE * 2
    } else {
        settings.rest_width()
    }
}

pub fn build(settings: &Settings, media_active: bool) -> gtk4::Box {
    let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    root.set_size_request(width(settings, media_active), settings.bar_height);
    root.set_valign(gtk4::Align::Center);

    let clock_view = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    clock_view.set_size_request(settings.rest_width(), settings.bar_height);
    clock_view.set_halign(gtk4::Align::Center);
    clock_view.set_valign(gtk4::Align::Center);
    clock_view.append(&clock::build(settings));
    root.append(&clock_view);

    let media_view = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    media_view.set_size_request(clock_core(settings) + SIDE * 2, settings.bar_height);
    media_view.set_valign(gtk4::Align::Center);
    root.append(&media_view);

    let mark = gtk4::Overlay::new();
    mark.set_size_request(SIDE, settings.bar_height);
    let picture = gtk4::Picture::new();
    picture.set_size_request(20, 20);
    picture.set_content_fit(gtk4::ContentFit::Cover);
    picture.set_can_shrink(true);
    picture.set_halign(gtk4::Align::Center);
    picture.set_valign(gtk4::Align::Center);
    mark.set_child(Some(&picture));
    let fallback = gtk4::Label::new(Some("󰎇"));
    fallback.add_css_class("chip-glyph");
    fallback.set_halign(gtk4::Align::Center);
    fallback.set_valign(gtk4::Align::Center);
    mark.add_overlay(&fallback);
    media_view.append(&mark);

    let core = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    core.set_size_request(clock_core(settings), settings.bar_height);
    core.set_halign(gtk4::Align::Center);
    core.set_valign(gtk4::Align::Center);
    core.append(&clock::build(settings));
    media_view.append(&core);

    // Cava's eight mirrored bands rest at the original Spectrum floor.
    let spectrum = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    spectrum.set_size_request(SIDE, settings.bar_height);
    spectrum.set_halign(gtk4::Align::Center);
    spectrum.set_valign(gtk4::Align::Center);
    let mut spectrum_bars = Vec::with_capacity(8);
    for _ in 0..8 {
        let bar = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        bar.set_size_request(2, 2);
        bar.add_css_class("island-spectrum-bar");
        spectrum.append(&bar);
        spectrum_bars.push(bar);
    }
    media_view.append(&spectrum);

    clock_view.set_visible(!media_active);
    media_view.set_visible(media_active);

    let mut current_art = String::new();
    let update_art = {
        let picture = picture.clone();
        let fallback = fallback.clone();
        move |url: &str, current: &mut String| {
            if *current == url {
                return;
            }
            *current = url.to_string();
            if let Some(file) = file_for_art(url) {
                picture.set_file(Some(&file));
                picture.set_visible(true);
                fallback.set_visible(false);
            } else {
                picture.set_file(None::<&gio::File>);
                picture.set_visible(false);
                fallback.set_visible(true);
            }
        }
    };

    let weak_root = root.downgrade();
    let clock_tick = clock_view.clone();
    let media_tick = media_view.clone();
    let bar_height = settings.bar_height;
    let rest_width = settings.rest_width();
    let active_width = clock_core(settings) + SIDE * 2;
    let mut cava_worker: Option<crate::services::cava::CavaWorker> = None;
    let mut cava_failed = false;
    glib::timeout_add_local(Duration::from_millis(750), move || {
        if weak_root.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        let snapshot = media::snapshot();
        let active = snapshot.as_ref().is_some_and(|item| item.playing());
        clock_tick.set_visible(!active);
        media_tick.set_visible(active);
        if let Some(root) = weak_root.upgrade() {
            root.set_size_request(if active { active_width } else { rest_width }, bar_height);
        }
        if let Some(snapshot) = snapshot.filter(|item| item.playing()) {
            update_art(&snapshot.art_url, &mut current_art);
            if cava_worker.is_none() && !cava_failed {
                match crate::services::cava::CavaWorker::start() {
                    Ok(worker) => cava_worker = Some(worker),
                    Err(_) => cava_failed = true,
                }
            }
            if cava_worker.as_mut().is_some_and(|worker| !worker.alive()) {
                cava_worker.take();
                cava_failed = true;
            }
            if let Some(worker) = cava_worker.as_ref() {
                for (bar, value) in spectrum_bars.iter().zip(worker.values()) {
                    bar.set_size_request(2, 2 + (value * 12.0).round() as i32);
                }
            }
        } else {
            update_art("", &mut current_art);
            cava_worker.take();
            cava_failed = false;
            for bar in &spectrum_bars {
                bar.set_size_request(2, 2);
            }
        }
        glib::ControlFlow::Continue
    });

    root
}
