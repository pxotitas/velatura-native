//! Stylesheet for the parts of the shell that are real GTK widgets (text).
//! Shapes are painted by their own widgets; this carries type, colours that
//! follow the palette, and the transparent layer window.

use gtk4::gdk;
use gtk4::prelude::*;
use std::rc::Rc;

use super::palette::{self, Palette};
use super::settings::Settings;
use super::tokens;

/// Families that are really installed. Fontconfig can list a family whose
/// file is gone (a dangling symlink), which Qt never offered and which pango
/// would draw as boxes, so each family needs at least one existing file.
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
        let Some((file, names)) = line.split_once('\t') else { continue };
        if std::path::Path::new(file).exists() {
            families.extend(names.split(',').map(|name| name.trim().to_string()));
        }
    }
    families
}

/// `Theme.fontOf`: Qt matches one family, never a stack, so the first family
/// of the stack that is installed is used, else the last entry.
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
    format!(
        "window.velatura-surface, window.velatura-surface > * {{\
            background: transparent; box-shadow: none; border: none; border-radius: 0; outline: none; }}\
         .island-clock {{ font-family: \"{family}\"; font-size: {regular}px; font-weight: 600; color: {text}; }}\
         .island-date {{ font-family: \"{family}\"; font-size: {small}px; font-weight: 400; color: {muted}; }}\
         .chip-glyph {{ font-family: \"{mono}\"; font-size: {glyph}px; color: {text}; }}\
         .toast-title {{ font-family: \"{family}\"; font-size: 14px; font-weight: 600; color: {text}; }}\
         .toast-meta {{ font-family: \"{family}\"; font-size: {small}px; color: {muted}; }}\
         .toast-body {{ font-family: \"{family}\"; font-size: {regular}px; color: {muted}; }}\
         .toast-button {{ font-family: \"{family}\"; font-size: 12px; font-weight: 500; color: {text}; }}\
         .toast-button-first {{ font-family: \"{family}\"; font-size: 12px; font-weight: 500; color: {accent_text}; }}\
         .toast-cell {{ font-family: \"{family}\"; font-size: {regular}px; font-weight: 500; color: {text}; }}\
         .toast-cell-first {{ font-family: \"{family}\"; font-size: {regular}px; font-weight: 600; color: {accent}; }}\
         .toast-glyph {{ font-family: \"{mono}\"; font-size: 17px; color: {accent}; }}\
         .toast-glyph-bad {{ font-family: \"{mono}\"; font-size: 17px; color: {bad}; }}\
         .toast-hairline {{ background: {rim}; }}\
         .panel-rule {{ background: {rim}; }}\
         .launcher-empty {{ font-family: \"{family}\"; font-size: {small}px; color: {muted}; }}\
         .launcher-row-name {{ font-family: \"{family}\"; font-size: {regular}px; color: {text}; }}\
         .launcher-row-subtitle {{ font-family: \"{family}\"; font-size: {small}px; color: {muted}; }}\
         .launcher-row-pin {{ font-family: \"{mono}\"; font-size: 12px; color: {muted}; }}\
         .launcher-row-glyph {{ font-family: \"{mono}\"; font-size: 14px; color: {accent}; }}\
         .launcher-glyph {{ font-family: \"{mono}\"; font-size: 17px; color: {accent}; }}\
         .launcher-field {{ font-family: \"{family}\"; font-size: 16px; color: {text}; background: none; \
            border: none; box-shadow: none; outline: none; caret-color: {accent}; min-height: 0; padding: 0; }}\
         .launcher-field selection {{ background: {accent}; color: {accent_text}; }}\
         .launcher-field placeholder {{ color: {muted}; }}",
        regular = tokens::FONT_SIZE_REGULAR,
        small = tokens::FONT_SIZE_SMALL,
        accent = p.accent.css(),
        rim = p.border.css(),
    )
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
        let (settings, fonts, weak) = (Rc::clone(settings), Rc::clone(&fonts), Rc::downgrade(&theme));
        move || {
            if let Some(theme) = weak.upgrade() {
                theme.provider.load_from_string(&render(&settings, &fonts, &palette::current()));
            }
        }
    });
    theme
}
