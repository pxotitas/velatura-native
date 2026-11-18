//! Stylesheet for GTK text and transparent shell surfaces.

use gtk4::gdk;
use gtk4::prelude::*;
use std::rc::Rc;

use super::palette::{self, Palette};
use super::settings::Settings;
use super::tokens;

fn installed_families() -> Vec<String> {
    let listed = std::process::Command::new("fc-list")
        .args(["--format", "%{file}\t%{family}\n"])
        .output()
        .ok()
        .filter(|output| output.status.success());
    let Some(output) = listed else {
        return gtk4::Label::new(None)
            .pango_context()
            .list_families()
            .iter()
            .map(|family| family.name().to_string())
            .collect();
    };
    let mut families = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some((file, names)) = line.split_once('\t') else {
            continue;
        };
        if std::path::Path::new(file).exists() {
            families.extend(names.split(',').map(|name| name.trim().to_string()));
        }
    }
    families
}

/// Qt matches one installed family from its fallback stack, never a stack.
pub fn font_of(stack: &str) -> String {
    let installed = installed_families();
    let names: Vec<&str> = stack.split(',').map(str::trim).collect();
    names
        .iter()
        .find(|name| !name.is_empty() && installed.iter().any(|family| family == **name))
        .or(names.last())
        .map(|name| (*name).to_string())
        .unwrap_or_else(|| "sans-serif".into())
}

struct Fonts {
    text: String,
    mono: String,
}

fn render(settings: &Settings, fonts: &Fonts, p: &Palette) -> String {
    let (text, muted, accent_text) = (p.text.css(), p.text_muted.css(), p.accent_text.css());
    let (family, mono) = (&fonts.text, &fonts.mono);
    let glyph = (f64::from(settings.bar_height) * 0.44).round() as i32;
    let bad = "rgba(255,69,58,1.000)";
    let mut css = format!(
        r#"
window.velatura-surface, window.velatura-surface > * {{
    background: transparent; box-shadow: none; border: none; border-radius: 0; outline: none;
}}
.island-clock {{ font-family: "{family}"; font-size: {regular}px; font-weight: 600; color: {text}; }}
.island-date {{ font-family: "{family}"; font-size: {small}px; font-weight: 400; color: {muted}; }}
.chip-glyph {{ font-family: "{mono}"; font-size: {glyph}px; color: {text}; }}
.toast-title {{ font-family: "{family}"; font-size: 14px; font-weight: 600; color: {text}; }}
.toast-meta {{ font-family: "{family}"; font-size: {small}px; color: {muted}; }}
.toast-body {{ font-family: "{family}"; font-size: {regular}px; color: {muted}; }}
.toast-button {{ font-family: "{family}"; font-size: 12px; font-weight: 500; color: {text}; }}
.toast-button-first {{ font-family: "{family}"; font-size: 12px; font-weight: 500; color: {accent_text}; }}
.toast-cell {{ font-family: "{family}"; font-size: {regular}px; font-weight: 500; color: {text}; }}
.toast-cell-first {{ font-family: "{family}"; font-size: {regular}px; font-weight: 600; color: {accent}; }}
.toast-glyph {{ font-family: "{mono}"; font-size: 17px; color: {accent}; }}
.toast-glyph-bad {{ font-family: "{mono}"; font-size: 17px; color: {bad}; }}
.toast-hairline {{ background: {rim}; }}
.panel-rule {{ background: {rim}; }}
.launcher-empty {{ font-family: "{family}"; font-size: {small}px; color: {muted}; }}
.launcher-clip-thumb {{ border-radius: 4px; }}
.launcher-row-name {{ font-family: "{family}"; font-size: {regular}px; color: {text}; }}
.launcher-row-subtitle {{ font-family: "{family}"; font-size: {small}px; color: {muted}; }}
.launcher-row-pin {{ font-family: "{mono}"; font-size: 12px; color: {muted}; }}
.launcher-row-glyph {{ font-family: "{mono}"; font-size: 14px; color: {accent}; }}
.launcher-glyph {{ font-family: "{mono}"; font-size: 17px; color: {accent}; }}
.launcher-field {{ font-family: "{family}"; font-size: 16px; color: {text}; background: none; border: none; box-shadow: none; outline: none; caret-color: {accent}; min-height: 0; padding: 0; }}
.launcher-field selection {{ background: {accent}; color: {accent_text}; }}
.launcher-field placeholder {{ color: {muted}; }}
"#,
        regular = tokens::FONT_SIZE_REGULAR,
        small = tokens::FONT_SIZE_SMALL,
        accent = p.accent.css(),
        rim = p.border.css(),
    );
    css.push_str(&format!(
        r#"
.summary-title {{ font-family: "{family}"; font-size: 15px; font-weight: 700; color: {text}; }}
.summary-artist {{ font-family: "{family}"; font-size: 13px; color: {muted}; }}
.summary-clock {{ font-family: "{family}"; font-size: 62px; font-weight: 900; color: {text}; }}
.summary-weekday {{ font-family: "{family}"; font-size: 20px; font-weight: 900; color: {text}; }}
.summary-date {{ font-family: "{family}"; font-size: 15px; font-weight: 900; color: {muted}; }}
.summary-art {{ border-radius: 18px; background: transparent; }}
.summary-art-fallback {{ font-family: "{mono}"; font-size: 76px; color: {muted}; }}
.summary-control {{ min-width: 34px; min-height: 34px; padding: 0; font-family: "{mono}"; font-size: 17px; color: {muted}; }}
.summary-control-primary {{ font-size: 22px; color: {text}; }}
.summary-day-name {{ font-family: "{family}"; font-size: 11px; font-weight: 700; color: {muted}; opacity: 0.7; }}
.summary-day-name-today {{ font-family: "{family}"; font-size: 11px; font-weight: 700; color: {accent}; }}
.summary-day-number {{ font-family: "{family}"; font-size: 22px; font-weight: 600; color: {muted}; opacity: 0.55; }}
.summary-day-number-today {{ font-family: "{family}"; font-size: 34px; font-weight: 900; color: {text}; }}
.island-spectrum-bar {{ background: {text}; border-radius: 1px; }}
.control-card {{ background: rgba(20,20,20,0.98); border: 1px solid {rim}; border-radius: 12px; padding: 12px; }}
.control-tile {{ background: rgba(21,21,21,1); border: 1px solid {rim}; border-radius: 12px; min-height: 64px; }}
.control-tile-active {{ border-color: {accent}; }}
.control-tile-main {{ padding: 7px 5px 7px 9px; }}
.control-tile-glyph {{ font-family: "{mono}"; font-size: 20px; color: {accent}; }}
.control-tile-title {{ font-family: "{family}"; font-size: 12px; font-weight: 600; color: {text}; }}
.control-tile-subtitle {{ font-family: "{family}"; font-size: 10px; color: {muted}; }}
.control-icon-button {{ min-width: 28px; min-height: 28px; padding: 3px; font-family: "{mono}"; font-size: 15px; color: {text}; }}
.control-slider-row {{ background: rgba(20,20,20,1); border: 1px solid {rim}; border-radius: 24px; padding: 5px 10px; }}
.control-slider trough {{ background: rgba(255,255,255,0.12); min-height: 4px; border-radius: 4px; }}
.control-slider highlight {{ background: {accent}; min-height: 4px; border-radius: 4px; }}
.control-slider slider {{ background: {text}; min-width: 10px; min-height: 10px; border-radius: 50%; }}
.control-value {{ font-family: "{mono}"; font-size: 10px; color: {text}; }}
.control-appearance-footer {{ background: linear-gradient(to top, rgba(0,0,0,0.9), rgba(0,0,0,0)); padding-top: 26px; }}
.control-title {{ font-family: "{family}"; font-size: 13px; font-weight: 600; color: {text}; }}
.control-subtitle {{ font-family: "{family}"; font-size: 10px; color: {muted}; }}
.control-weather-glyph {{ font-family: "{mono}"; font-size: 36px; color: {accent}; }}
.control-weather-temp {{ font-family: "{family}"; font-size: 28px; font-weight: 700; color: {text}; }}
.control-notification {{ padding: 6px 0; }}
.control-notification-glyph {{ font-family: "{mono}"; font-size: 22px; color: {accent}; }}
.control-notification-title {{ font-family: "{family}"; font-size: 11px; font-weight: 600; color: {text}; }}
.control-notification-app {{ font-family: "{family}"; font-size: 9px; color: {muted}; }}
.control-clear {{ font-family: "{family}"; font-size: 10px; color: {muted}; }}
"#,
        family = family,
        mono = mono,
        text = text,
        muted = muted,
        accent = p.accent.css(),
        rim = p.border.css(),
    ));
    css
}

pub struct Theme {
    provider: gtk4::CssProvider,
}

/// Installs the stylesheet and keeps it in step with the palette.
pub fn install(settings: &Rc<Settings>) -> Rc<Theme> {
    let fonts = Rc::new(Fonts {
        text: font_of(&settings.font_family),
        mono: font_of(&settings.font_mono),
    });
    // Use grayscale antialiasing and disable pixel-grid hinting. The shell
    // is rendered on a scaled Wayland surface; forcing glyph stems onto the
    // physical pixel grid makes small launcher text look jagged at fractional
    // logical sizes. Fontconfig still supplies the selected family/weight.
    if let Some(gtk_settings) = gtk4::Settings::default() {
        gtk_settings.set_gtk_xft_antialias(1);
        gtk_settings.set_gtk_xft_hinting(0);
        gtk_settings.set_gtk_xft_hintstyle(Some("hintnone"));
        gtk_settings.set_gtk_xft_rgba(Some("none"));
    }
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(&render(settings, &fonts, &palette::current()));
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    let theme = Rc::new(Theme { provider });
    palette::on_change({
        let (settings, fonts, weak) = (
            Rc::clone(settings),
            Rc::clone(&fonts),
            Rc::downgrade(&theme),
        );
        move || {
            if let Some(theme) = weak.upgrade() {
                theme
                    .provider
                    .load_from_string(&render(&settings, &fonts, &palette::current()));
            }
        }
    });
    theme
}
