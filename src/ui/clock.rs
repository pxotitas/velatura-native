//! `ClockModule.qml`: the island's resting face. The time, optionally with
//! the date, centred in a capsule-height lane.

use gtk4::glib;
use gtk4::prelude::*;
use std::time::Duration;

use super::settings::Settings;

/// Converts a Qt date/time format (`HH:mm`, `ddd d MMM`) to `g_date_time_format`.
pub fn qt_to_glib(format: &str) -> String {
    const TOKENS: &[(&str, &str)] = &[
        ("yyyy", "%Y"),
        ("yy", "%y"),
        ("MMMM", "%B"),
        ("MMM", "%b"),
        ("MM", "%m"),
        ("M", "%-m"),
        ("dddd", "%A"),
        ("ddd", "%a"),
        ("dd", "%d"),
        ("d", "%-d"),
        ("HH", "%H"),
        ("H", "%-H"),
        ("hh", "%I"),
        ("h", "%-I"),
        ("mm", "%M"),
        ("m", "%-M"),
        ("ss", "%S"),
        ("s", "%-S"),
        ("AP", "%p"),
        ("ap", "%P"),
    ];
    let mut out = String::new();
    let mut rest = format;
    'scan: while let Some(first) = rest.chars().next() {
        if first == '\'' {
            let body = &rest[1..];
            let end = body.find('\'').unwrap_or(body.len());
            out.push_str(&body[..end].replace('%', "%%"));
            rest = body.get(end + 1..).unwrap_or("");
            continue;
        }
        for (token, replacement) in TOKENS {
            if let Some(after) = rest.strip_prefix(token) {
                out.push_str(replacement);
                rest = after;
                continue 'scan;
            }
        }
        if first == '%' {
            out.push_str("%%");
        } else {
            out.push(first);
        }
        rest = &rest[first.len_utf8()..];
    }
    out
}

fn time_format(settings: &Settings) -> String {
    // Seconds are appended to the chosen format so the two settings stay
    // independent.
    let qt = if settings.clock_shows_seconds {
        settings.clock_format.replacen("mm", "mm:ss", 1)
    } else {
        settings.clock_format.clone()
    };
    qt_to_glib(&qt)
}

fn now(format: &str) -> String {
    glib::DateTime::now_local()
        .ok()
        .and_then(|time| time.format(format).ok())
        .map(|text| text.to_string())
        .unwrap_or_default()
}

pub fn build(settings: &Settings) -> gtk4::Widget {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row.set_halign(gtk4::Align::Center);
    row.set_valign(gtk4::Align::Center);
    row.set_height_request(settings.bar_height);

    let time = gtk4::Label::new(None);
    time.add_css_class("island-clock");
    row.append(&time);

    let date = gtk4::Label::new(None);
    date.add_css_class("island-date");
    date.set_visible(settings.clock_shows_date);
    row.append(&date);

    let time_format = time_format(settings);
    let date_format = qt_to_glib("ddd d MMM");
    let refresh = {
        let (time, date) = (time.clone(), date.clone());
        move || {
            let text = now(&time_format);
            if time.text() != text {
                time.set_text(&text);
            }
            if date.is_visible() {
                let text = now(&date_format);
                if date.text() != text {
                    date.set_text(&text);
                }
            }
        }
    };
    refresh();
    let source = glib::timeout_add_local(Duration::from_millis(500), move || {
        refresh();
        glib::ControlFlow::Continue
    });
    // The timer dies with the widget so a closed bar never ticks.
    let source = std::cell::RefCell::new(Some(source));
    row.connect_destroy(move |_| {
        if let Some(source) = source.borrow_mut().take() {
            source.remove();
        }
    });
    row.upcast()
}
