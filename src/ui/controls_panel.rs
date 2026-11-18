//! Native control-centre grid, based on ControlsPanel.qml's default board.

use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::Cell;
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

use super::settings::Settings;
use crate::services::media;
use crate::services::notifications::NotificationService;
use crate::services::system_controls as controls;
use crate::services::weather;

pub const WIDTH: i32 = 940;
pub const HEIGHT: i32 = 678;
const CONTENT_WIDTH: i32 = 900;
const CONTENT_HEIGHT: i32 = 638;
const COLUMN_WIDTH: i32 = 292;

fn card(width: i32, height: i32) -> gtk4::Box {
    let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    box_.set_size_request(width, height);
    box_.add_css_class("control-card");
    box_
}

fn label(text: &str, class: &str) -> gtk4::Label {
    let label = gtk4::Label::new(Some(text));
    label.add_css_class(class);
    label.set_xalign(0.0);
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label
}

fn action_button(glyph: &str) -> gtk4::Button {
    let button = gtk4::Button::with_label(glyph);
    button.add_css_class("flat");
    button.add_css_class("control-icon-button");
    button
}

#[allow(clippy::too_many_arguments)]
fn toggle_tile(
    glyph: &str,
    title: &str,
    detail: &str,
    active: bool,
    on_label: &str,
    off_label: &str,
    action: impl Fn(bool) -> Result<(), String> + 'static,
    open: Option<Rc<dyn Fn()>>,
) -> gtk4::Widget {
    let tile = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    tile.add_css_class("control-tile");
    if active {
        tile.add_css_class("control-tile-active");
    }

    let main = gtk4::Button::new();
    main.set_hexpand(true);
    main.add_css_class("flat");
    main.add_css_class("control-tile-main");
    let contents = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    let icon = gtk4::Label::new(Some(glyph));
    icon.add_css_class("control-tile-glyph");
    let texts = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    texts.set_hexpand(true);
    texts.set_halign(gtk4::Align::Start);
    let name = label(title, "control-tile-title");
    let subtitle = label(detail, "control-tile-subtitle");
    texts.append(&name);
    texts.append(&subtitle);
    contents.append(&icon);
    contents.append(&texts);
    main.set_child(Some(&contents));

    let state = Rc::new(Cell::new(active));
    let state_tick = Rc::clone(&state);
    let subtitle_tick = subtitle.clone();
    let tile_tick = tile.clone();
    let on_label = on_label.to_string();
    let off_label = off_label.to_string();
    main.connect_clicked(move |_| {
        let next = !state_tick.get();
        if action(next).is_ok() {
            state_tick.set(next);
            subtitle_tick.set_text(if next { &on_label } else { &off_label });
            if next {
                tile_tick.add_css_class("control-tile-active");
            } else {
                tile_tick.remove_css_class("control-tile-active");
            }
        }
    });
    tile.append(&main);

    if let Some(open) = open {
        let arrow = action_button("›");
        arrow.connect_clicked(move |_| open());
        tile.append(&arrow);
    }
    tile.upcast()
}

fn weather_place() -> String {
    std::fs::read_to_string(Settings::path())
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|value| value.get("weatherPlace")?.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn art_file(url: &str) -> Option<gio::File> {
    if url.starts_with("file://") {
        Some(gio::File::for_uri(url))
    } else if url.starts_with('/') {
        Some(gio::File::for_path(url))
    } else {
        None
    }
}

fn set_art(picture: &gtk4::Picture, url: &str, current: &mut String) {
    if *current == url {
        return;
    }
    *current = url.to_string();
    picture.set_file(art_file(url).as_ref());
}

fn format_time(seconds: f64) -> String {
    let total = seconds.max(0.0).floor() as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

fn wallpaper_file() -> Option<String> {
    let output = Command::new("awww").arg("query").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let marker = "currently displaying: image: ";
    let path = text
        .lines()
        .find_map(|line| line.split_once(marker).map(|(_, path)| path.trim()))?;
    std::path::Path::new(path)
        .exists()
        .then(|| path.to_string())
}

pub struct ControlsPanel {
    pub widget: gtk4::Box,
}

impl ControlsPanel {
    pub fn new(
        notifications: Rc<NotificationService>,
        on_close: impl Fn() + 'static,
        on_open_panel: impl Fn(&str) + 'static,
    ) -> Self {
        let on_open_panel: Rc<dyn Fn(&str)> = Rc::new(on_open_panel);
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.set_size_request(CONTENT_WIDTH, CONTENT_HEIGHT);
        root.add_css_class("controls-panel");

        // ── TOP ROW ────────────────────────────────────────────────────────
        let top = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        top.set_size_request(CONTENT_WIDTH, 28);
        let lock = action_button("󰌾");
        lock.set_tooltip_text(Some("Lock session"));
        lock.connect_clicked(|_| {
            let _ = Command::new("loginctl").arg("lock-session").spawn();
        });
        let focus = action_button(if notifications.do_not_disturb() {
            "󰂛"
        } else {
            "󰂚"
        });
        focus.set_tooltip_text(Some("Toggle focus"));
        {
            let notifications = Rc::clone(&notifications);
            focus.connect_clicked(move |button| {
                let next = !notifications.do_not_disturb();
                notifications.set_do_not_disturb(next);
                button.set_label(if next { "󰂛" } else { "󰂚" });
            });
        }
        top.append(&lock);
        top.append(&focus);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        top.append(&spacer);
        let close = action_button("󰅖");
        close.set_tooltip_text(Some("Close control centre"));
        close.connect_clicked(move |_| on_close());
        top.append(&close);
        root.append(&top);

        // ── THREE-COLUMN DEFAULT BOARD ─────────────────────────────────────
        let body = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        body.set_size_request(CONTENT_WIDTH, 596);

        // LEFT: six toggles, output/brightness sliders and wallpaper.
        let left = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        left.set_size_request(COLUMN_WIDTH, 596);
        let tiles = gtk4::Grid::new();
        tiles.set_column_spacing(12);
        tiles.set_row_spacing(12);
        tiles.set_size_request(COLUMN_WIDTH, 216);

        let wifi = controls::wifi_state().ok();
        let wifi_on = wifi.as_ref().is_some_and(|state| state.enabled);
        let wifi_detail = wifi
            .as_ref()
            .map(|state| {
                if state.connected {
                    state.ssid.clone()
                } else if state.enabled {
                    "Not connected".into()
                } else {
                    "Off".into()
                }
            })
            .unwrap_or_else(|| "Unavailable".into());
        let wifi_open = {
            let open = Rc::clone(&on_open_panel);
            Rc::new(move || open("wifi")) as Rc<dyn Fn()>
        };
        tiles.attach(
            &toggle_tile(
                "󰤨",
                "Wi-Fi",
                &wifi_detail,
                wifi_on,
                "On",
                "Off",
                controls::set_wifi,
                Some(wifi_open),
            ),
            0,
            0,
            1,
            1,
        );

        let bluetooth = controls::bluetooth_state().ok();
        let bluetooth_on = bluetooth.as_ref().is_some_and(|state| state.enabled);
        let bluetooth_detail = bluetooth
            .as_ref()
            .map(|state| {
                state
                    .devices
                    .iter()
                    .find(|device| device.connected)
                    .map(|device| device.name.clone())
                    .unwrap_or_else(|| {
                        if state.enabled {
                            "No devices".into()
                        } else {
                            "Off".into()
                        }
                    })
            })
            .unwrap_or_else(|| "Unavailable".into());
        let bluetooth_open = {
            let open = Rc::clone(&on_open_panel);
            Rc::new(move || open("bluetooth")) as Rc<dyn Fn()>
        };
        tiles.attach(
            &toggle_tile(
                "󰂯",
                "Bluetooth",
                &bluetooth_detail,
                bluetooth_on,
                "On",
                "Off",
                controls::set_bluetooth,
                Some(bluetooth_open),
            ),
            1,
            0,
            1,
            1,
        );

        let profile = controls::power_profile().unwrap_or_else(|_| "Unknown".into());
        tiles.attach(
            &toggle_tile(
                "󰓅",
                "Power",
                &profile,
                profile == "performance",
                "Performance",
                "Balanced",
                |enabled| {
                    let target = if enabled { "performance" } else { "balanced" };
                    let status = Command::new("powerprofilesctl")
                        .args(["set", target])
                        .status()
                        .map_err(|error| error.to_string())?;
                    if status.success() {
                        Ok(())
                    } else {
                        Err("power profile failed".into())
                    }
                },
                None,
            ),
            0,
            1,
            1,
            1,
        );

        let dnd = notifications.do_not_disturb();
        {
            let service = Rc::clone(&notifications);
            tiles.attach(
                &toggle_tile(
                    "󰂛",
                    "Focus",
                    if dnd { "Silenced" } else { "Notifying" },
                    dnd,
                    "Silenced",
                    "Notifying",
                    move |enabled| {
                        service.set_do_not_disturb(enabled);
                        Ok(())
                    },
                    None,
                ),
                1,
                1,
                1,
                1,
            );
        }

        let mic_muted = controls::microphone_muted().unwrap_or(false);
        tiles.attach(
            &toggle_tile(
                "󰍬",
                "Microphone",
                if mic_muted { "Muted" } else { "Live" },
                !mic_muted,
                "Live",
                "Muted",
                |enabled| controls::set_microphone_muted(!enabled),
                None,
            ),
            0,
            2,
            1,
            1,
        );

        let airplane = !wifi_on && !bluetooth_on;
        tiles.attach(
            &toggle_tile(
                "󰀝",
                "Airplane",
                if airplane { "Radios off" } else { "Radios on" },
                airplane,
                "Radios off",
                "Radios on",
                move |enabled| {
                    controls::set_wifi(!enabled)?;
                    controls::set_bluetooth(!enabled)
                },
                None,
            ),
            1,
            2,
            1,
            1,
        );
        left.append(&tiles);

        let audio = controls::audio_state().ok();
        let volume_value = audio.as_ref().map_or(0, |state| state.volume);
        let volume_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        volume_row.set_size_request(COLUMN_WIDTH, 44);
        volume_row.add_css_class("control-slider-row");
        let mute = action_button(if audio.as_ref().is_some_and(|state| state.muted) {
            "󰖁"
        } else {
            "󰕾"
        });
        {
            let mute_label = mute.clone();
            mute.connect_clicked(move |_| {
                if controls::toggle_mute().is_ok() {
                    if let Ok(state) = controls::audio_state() {
                        mute_label.set_label(if state.muted { "󰖁" } else { "󰕾" });
                    }
                }
            });
        }
        let volume = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        volume.set_hexpand(true);
        volume.set_draw_value(false);
        volume.add_css_class("control-slider");
        volume.set_value(f64::from(volume_value));
        let volume_label = label(&format!("{volume_value}%"), "control-value");
        let volume_label_tick = volume_label.clone();
        volume.connect_value_changed(move |scale| {
            let value = scale.value().round().clamp(0.0, 100.0) as u8;
            volume_label_tick.set_text(&format!("{value}%"));
            let _ = controls::set_volume(value);
        });
        volume_row.append(&mute);
        volume_row.append(&volume);
        volume_row.append(&volume_label);
        left.append(&volume_row);

        let brightness_value = controls::brightness_state().unwrap_or(0);
        let brightness_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        brightness_row.set_size_request(COLUMN_WIDTH, 44);
        brightness_row.add_css_class("control-slider-row");
        let brightness_icon = action_button("󰃠");
        let brightness = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        brightness.set_hexpand(true);
        brightness.set_draw_value(false);
        brightness.add_css_class("control-slider");
        brightness.set_value(f64::from(brightness_value));
        let brightness_label = label(&format!("{brightness_value}%"), "control-value");
        let brightness_label_tick = brightness_label.clone();
        brightness.connect_value_changed(move |scale| {
            let value = scale.value().round().clamp(0.0, 100.0) as u8;
            brightness_label_tick.set_text(&format!("{value}%"));
            let _ = controls::set_brightness(value);
        });
        brightness_row.append(&brightness_icon);
        brightness_row.append(&brightness);
        brightness_row.append(&brightness_label);
        left.append(&brightness_row);

        let appearance = gtk4::Overlay::new();
        appearance.set_size_request(COLUMN_WIDTH, 216);
        appearance.add_css_class("control-card");
        let wallpaper = gtk4::Picture::new();
        wallpaper.set_size_request(COLUMN_WIDTH, 216);
        wallpaper.set_content_fit(gtk4::ContentFit::Cover);
        if let Some(path) = wallpaper_file() {
            wallpaper.set_file(Some(&gio::File::for_path(path)));
        }
        appearance.set_child(Some(&wallpaper));
        let appearance_footer = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        appearance_footer.add_css_class("control-appearance-footer");
        appearance_footer.set_margin_start(12);
        appearance_footer.set_margin_end(12);
        appearance_footer.set_margin_bottom(10);
        appearance_footer.set_halign(gtk4::Align::Fill);
        appearance_footer.set_valign(gtk4::Align::End);
        appearance_footer.append(&label("Appearance", "control-title"));
        appearance_footer.append(&label("Adaptive (Wallpaper)", "control-subtitle"));
        appearance.add_overlay(&appearance_footer);
        left.append(&appearance);
        body.append(&left);

        // CENTRE: media, weather, calendar.
        let centre = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        centre.set_size_request(COLUMN_WIDTH, 596);

        let media_card = card(COLUMN_WIDTH, 140);
        let media_head = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let media_art = gtk4::Picture::new();
        media_art.set_size_request(56, 56);
        media_art.set_content_fit(gtk4::ContentFit::Cover);
        let media_meta = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        media_meta.set_hexpand(true);
        let media_title = label("Nothing playing", "control-title");
        let media_artist = label("", "control-subtitle");
        media_meta.append(&media_title);
        media_meta.append(&media_artist);
        media_head.append(&media_art);
        media_head.append(&media_meta);
        media_card.append(&media_head);
        let media_progress = gtk4::ProgressBar::new();
        media_progress.set_height_request(3);
        media_progress.add_css_class("control-media-progress");
        media_card.append(&media_progress);
        let transport = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let previous = action_button("󰒮");
        let play = action_button("󰐊");
        let next = action_button("󰒭");
        previous.connect_clicked(|_| media::previous());
        play.connect_clicked(|_| media::toggle());
        next.connect_clicked(|_| media::next());
        transport.append(&previous);
        transport.append(&play);
        transport.append(&next);
        let position = label("0:00", "control-value");
        position.set_hexpand(true);
        position.set_xalign(1.0);
        transport.append(&position);
        media_card.append(&transport);
        centre.append(&media_card);

        let weather_card = card(COLUMN_WIDTH, 140);
        let weather_head = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let weather_icon = label("󰅤", "control-weather-glyph");
        let weather_temp = label("—°", "control-weather-temp");
        let weather_meta = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        weather_meta.set_hexpand(true);
        let weather_desc = label("Loading weather…", "control-title");
        let weather_location = label("", "control-subtitle");
        weather_meta.append(&weather_desc);
        weather_meta.append(&weather_location);
        weather_head.append(&weather_icon);
        weather_head.append(&weather_temp);
        weather_head.append(&weather_meta);
        weather_card.append(&weather_head);
        let weather_range = label("", "control-subtitle");
        weather_card.append(&weather_range);
        centre.append(&weather_card);

        let calendar = card(COLUMN_WIDTH, 292);
        let calendar_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let now = glib::DateTime::now_local().ok();
        let day_text = now
            .as_ref()
            .and_then(|now| now.format("%A %-d %B").ok())
            .unwrap_or_default();
        let date_label = label(&day_text, "control-title");
        date_label.set_hexpand(true);
        let calendar_icon = label("‹", "control-subtitle");
        calendar_header.append(&calendar_icon);
        calendar_header.append(&date_label);
        calendar_header.append(&label("＋", "control-subtitle"));
        calendar.append(&calendar_header);
        let calendar_rule = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        calendar.append(&calendar_rule);
        centre.append(&calendar);
        body.append(&centre);

        // RIGHT: notification history, not a fake notification preview.
        let notification_card = card(COLUMN_WIDTH, 596);
        let notification_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let notification_title = label(
            &format!("Notifications  {}", notifications.history().len()),
            "control-title",
        );
        notification_title.set_hexpand(true);
        let clear = gtk4::Button::with_label("Clear");
        clear.add_css_class("flat");
        clear.add_css_class("control-clear");
        notification_header.append(&notification_title);
        notification_header.append(&clear);
        notification_card.append(&notification_header);
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scroll.set_vexpand(true);
        let notification_list = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        for entry in notifications.history().iter().take(20) {
            let item = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
            item.add_css_class("control-notification");
            let glyph = label("󰂚", "control-notification-glyph");
            let text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
            let heading = label(&entry.summary, "control-notification-title");
            let body = label(&entry.body, "control-subtitle");
            let app = label(&entry.app_name, "control-notification-app");
            text.append(&heading);
            text.append(&body);
            text.append(&app);
            item.append(&glyph);
            item.append(&text);
            notification_list.append(&item);
        }
        if notification_list.first_child().is_none() {
            notification_list.append(&label("You're all caught up", "control-subtitle"));
        }
        scroll.set_child(Some(&notification_list));
        notification_card.append(&scroll);
        {
            let notifications = Rc::clone(&notifications);
            let list = notification_list.clone();
            let title = notification_title.clone();
            clear.connect_clicked(move |_| {
                notifications.clear_history();
                while let Some(child) = list.first_child() {
                    list.remove(&child);
                }
                list.append(&label("You're all caught up", "control-subtitle"));
                title.set_text("Notifications  0");
            });
        }
        body.append(&notification_card);

        root.append(&body);

        // Weather I/O happens off the GTK thread; no guessed fallback values.
        let (weather_tx, weather_rx) = std::sync::mpsc::channel();
        let place = weather_place();
        std::thread::spawn(move || {
            let _ = weather_tx.send(weather::fetch(&place));
        });
        let weak_root = root.downgrade();
        glib::timeout_add_local(Duration::from_millis(500), move || {
            if weak_root.upgrade().is_none() {
                return glib::ControlFlow::Break;
            }
            match weather_rx.try_recv() {
                Ok(Ok(report)) => {
                    weather_icon.set_text(&report.glyph);
                    weather_temp.set_text(&format!("{}°", report.temperature));
                    weather_desc.set_text(&format!(
                        "{} · feels {}°",
                        report.description, report.feels_like
                    ));
                    weather_location.set_text(&format!("{}, {}", report.place, report.region));
                    weather_range.set_text(&format!("{}° / {}°", report.high, report.low));
                    glib::ControlFlow::Break
                }
                Ok(Err(_)) => {
                    weather_desc.set_text("No reading");
                    weather_location.set_text("Weather unavailable");
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            }
        });

        // MPRIS updates while the control centre stays open.
        let mut current_art = String::new();
        let weak_root = root.downgrade();
        glib::timeout_add_local(Duration::from_millis(750), move || {
            if weak_root.upgrade().is_none() {
                return glib::ControlFlow::Break;
            }
            if let Some(snapshot) = media::snapshot() {
                media_title.set_text(snapshot.identity());
                media_artist.set_text(&snapshot.artist);
                play.set_label(if snapshot.playing() { "󰏤" } else { "󰐊" });
                set_art(&media_art, &snapshot.art_url, &mut current_art);
                position.set_text(&format_time(snapshot.position_s));
                let length = snapshot.length_us as f64 / 1_000_000.0;
                media_progress.set_fraction(if length > 0.0 && length.is_finite() {
                    (snapshot.position_s / length).clamp(0.0, 1.0)
                } else {
                    0.0
                });
            } else {
                media_title.set_text("Nothing playing");
                media_artist.set_text("");
                set_art(&media_art, "", &mut current_art);
                media_progress.set_fraction(0.0);
            }
            glib::ControlFlow::Continue
        });

        Self { widget: root }
    }
}
