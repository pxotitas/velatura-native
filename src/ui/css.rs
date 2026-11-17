//! Stylesheet for the parts of the shell that are real GTK widgets (text).
//! Shapes are painted by their own widgets; this only carries type and the
//! transparent layer window.

use gtk4::gdk;
use gtk4::prelude::*;

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
        let Some((file, names)) = line.split_once('\t') else {
            continue;
        };
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

pub fn install(settings: &Settings) -> gtk4::CssProvider {
    let family = font_of(&settings.font_family);
    let mono = font_of(&settings.font_mono);
    let glyph = (f64::from(settings.bar_height) * 0.44).round() as i32;
    let css = format!(
        "window.velatura-surface, window.velatura-surface > * {{\
            background: transparent; box-shadow: none; border: none; border-radius: 0; outline: none; }}\
         .island-clock {{ font-family: \"{family}\"; font-size: {regular}px; font-weight: 600; color: {text}; }}\
         .chip-glyph {{ font-family: \"{mono}\"; font-size: {glyph}px; color: {text}; }}\
         .island-date {{ font-family: \"{family}\"; font-size: {small}px; font-weight: 400; color: {muted}; }}\
         .toast-title {{ font-family: \"{family}\"; font-size: {regular}px; font-weight: 700; color: {text}; }}\
         .toast-body {{ font-family: \"{family}\"; font-size: {small}px; font-weight: 400; color: {muted}; }}\
         .toast-button {{ font-family: \"{family}\"; font-size: {small}px; font-weight: 600; \
            color: {text}; background: {surface}; border-radius: 999px; padding: 4px 12px; }}",
        regular = tokens::FONT_SIZE_REGULAR,
        small = tokens::FONT_SIZE_SMALL,
        text = tokens::TEXT.css(),
        muted = tokens::TEXT_MUTED.css(),
        surface = tokens::ISLAND_SURFACE.css(),
    );
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(&css);
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    provider
}
