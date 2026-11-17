//! Design tokens from `theme/Theme.qml` that the native surfaces share.
//! Only values that are fixed in the original live here; anything driven by a
//! setting is read from [`super::settings::Settings`].

use gtk4::gdk;

pub const CAPSULE_SPACING: i32 = 8;
pub const PANEL_PADDING: i32 = 20;
pub const RADIUS_LARGE: f32 = 18.0;
pub const RADIUS_NOTCH: i32 = 6;

pub const DURATION_FAST: f64 = 140.0;
pub const DURATION_MEDIUM: f64 = 200.0;
pub const DURATION_MORPH: f64 = 380.0;

pub const FONT_SIZE_REGULAR: i32 = 13;
pub const FONT_SIZE_SMALL: i32 = 11;

pub const ISLAND: Rgba = Rgba(0.0, 0.0, 0.0, 1.0);
pub const ISLAND_SURFACE: Rgba = Rgba::hex(0x14, 0x14, 0x14);
pub const ISLAND_SURFACE_HOVER: Rgba = Rgba::hex(0x1f, 0x1f, 0x1f);
pub const TEXT: Rgba = Rgba(1.0, 1.0, 1.0, 1.0);
pub const TEXT_MUTED: Rgba = Rgba::hex(0x8e, 0x8e, 0x93);
pub const ACCENT: Rgba = Rgba::hex(0x0a, 0x84, 0xff);
/// `Theme.accentText` (classic).
pub const ACCENT_TEXT: Rgba = Rgba(1.0, 1.0, 1.0, 1.0);
/// `Theme.indicatorDim` — status indicators are fixed, not palette-driven.
pub const INDICATOR_DIM: Rgba = Rgba::hex(0x4d, 0x4d, 0x4d);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

impl Rgba {
    pub const fn hex(r: u8, g: u8, b: u8) -> Self {
        Self(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
    }

    pub fn gdk(self) -> gdk::RGBA {
        gdk::RGBA::new(self.0, self.1, self.2, self.3)
    }

    pub fn css(self) -> String {
        format!(
            "rgba({},{},{},{:.3})",
            (self.0 * 255.0).round() as u8,
            (self.1 * 255.0).round() as u8,
            (self.2 * 255.0).round() as u8,
            self.3
        )
    }

    pub fn mix(self, other: Rgba, t: f32) -> Self {
        Self(
            self.0 + (other.0 - self.0) * t,
            self.1 + (other.1 - self.1) * t,
            self.2 + (other.2 - self.2) * t,
            self.3 + (other.3 - self.3) * t,
        )
    }

    /// `Qt.tint(self, overlay)`: the overlay laid over this colour.
    pub fn tint(self, over: Rgba) -> Self {
        let a = over.3;
        Self(
            self.0 * (1.0 - a) + over.0 * a,
            self.1 * (1.0 - a) + over.1 * a,
            self.2 * (1.0 - a) + over.2 * a,
            self.3,
        )
    }
}

pub fn see_through(style: &str) -> bool {
    matches!(style, "glass" | "liquid")
}

/// `Theme.groundOf`. `matugen` needs the wallpaper palette, which the theme
/// backend does not provide yet, so it falls back to the dark default the
/// original starts from (`matugenBackground: #0c0c0c`).
pub fn ground_of(style: &str) -> Rgba {
    match style {
        "matugen" => Rgba(
            0x0c as f32 / 255.0,
            0x0c as f32 / 255.0,
            0x0c as f32 / 255.0,
            0.96,
        ),
        "liquid" => Rgba(0.06, 0.07, 0.09, 0.66),
        "glass" => Rgba(ACCENT.0 * 0.22, ACCENT.1 * 0.22, ACCENT.2 * 0.22, 0.90),
        _ => ISLAND,
    }
}

/// `Theme.rimOf`.
pub fn rim_of(style: &str) -> Rgba {
    match style {
        "liquid" => Rgba(1.0, 1.0, 1.0, 0.08),
        "glass" => Rgba(1.0, 1.0, 1.0, 0.20),
        _ => Rgba::hex(0x26, 0x26, 0x26),
    }
}

pub fn ground_lit(style: &str) -> Rgba {
    ground_of(style).tint(Rgba(1.0, 1.0, 1.0, 0.10))
}
