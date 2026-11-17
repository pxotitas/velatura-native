//! The live colours the original's `Theme` takes from the wallpaper
//! (`dynamic-colors.json`, the adaptive theme). Island and capsule grounds are
//! not here: they belong to the surface style and stay in `tokens`.
//! The static palettes are pending; with no file the dark defaults apply.

use gtk4::{gio, glib, prelude::*};
use serde::Deserialize;
use std::cell::RefCell;
use std::path::PathBuf;

use super::tokens::Rgba;

#[derive(Debug, Clone)]
pub struct Palette {
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub accent_text: Rgba,
    pub border: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub surface_hover: Rgba,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            accent: Rgba::hex(0x0a, 0x84, 0xff),
            accent_hover: Rgba::hex(0x40, 0x9c, 0xff),
            accent_text: Rgba::hex(0xff, 0xff, 0xff),
            border: Rgba::hex(0x26, 0x26, 0x26),
            text: Rgba::hex(0xff, 0xff, 0xff),
            text_muted: Rgba::hex(0x8e, 0x8e, 0x93),
            surface_hover: Rgba::hex(0x20, 0x20, 0x20),
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Colors {
    accent: Option<String>,
    accent_hover: Option<String>,
    accent_text: Option<String>,
    border: Option<String>,
    text: Option<String>,
    text_muted: Option<String>,
    surface_hover: Option<String>,
}

fn parse(hex: &Option<String>) -> Option<Rgba> {
    let hex = hex.as_deref()?.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    Some(Rgba::hex((value >> 16) as u8, (value >> 8) as u8, value as u8))
}

thread_local! {
    static CURRENT: RefCell<Palette> = RefCell::new(Palette::default());
    static LISTENERS: RefCell<Vec<Box<dyn Fn()>>> = const { RefCell::new(Vec::new()) };
}

pub fn path() -> PathBuf {
    super::settings::Settings::path().with_file_name("dynamic-colors.json")
}

pub fn current() -> Palette {
    CURRENT.with(|palette| palette.borrow().clone())
}

/// Runs `callback` whenever the palette changes (widgets redraw from it).
pub fn on_change(callback: impl Fn() + 'static) {
    LISTENERS.with(|listeners| listeners.borrow_mut().push(Box::new(callback)));
}

fn reload() {
    let Some(colors) = std::fs::read_to_string(path())
        .ok()
        .and_then(|text| serde_json::from_str::<Colors>(&text).ok())
    else {
        return;
    };
    let base = Palette::default();
    let next = Palette {
        accent: parse(&colors.accent).unwrap_or(base.accent),
        accent_hover: parse(&colors.accent_hover).unwrap_or(base.accent_hover),
        accent_text: parse(&colors.accent_text).unwrap_or(base.accent_text),
        border: parse(&colors.border).unwrap_or(base.border),
        text: parse(&colors.text).unwrap_or(base.text),
        text_muted: parse(&colors.text_muted).unwrap_or(base.text_muted),
        surface_hover: parse(&colors.surface_hover).unwrap_or(base.surface_hover),
    };
    CURRENT.with(|palette| *palette.borrow_mut() = next);
    LISTENERS.with(|listeners| listeners.borrow().iter().for_each(|callback| callback()));
}

/// Loads the palette and follows the file. The returned monitor must be kept.
pub fn watch() -> Option<gio::FileMonitor> {
    reload();
    let monitor = gio::File::for_path(path())
        .monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
        .ok()?;
    monitor.connect_changed(|_, _, _, _| {
        // The writer replaces the file; wait for it to settle.
        glib::timeout_add_local_once(std::time::Duration::from_millis(120), reload);
    });
    Some(monitor)
}
