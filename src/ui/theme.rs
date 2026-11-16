//! Shared visual tokens transcribed from the active Quickshell theme.
//!
//! This is deliberately a presentation layer, not a settings store. Runtime
//! selection and persistence belong to the backend Settings/Theme services,
//! which are not yet exposed by this checkout.

use gtk4::CssProvider;
use gtk4::gdk;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Palette {
    pub id: &'static str,
    pub name: &'static str,
    pub background: &'static str,
    pub surface: &'static str,
    pub surface_hover: &'static str,
    pub border: &'static str,
    pub text: &'static str,
    pub muted: &'static str,
    pub accent: &'static str,
    pub accent_hover: &'static str,
    pub accent_text: &'static str,
    pub red: &'static str,
    pub green: &'static str,
    pub yellow: &'static str,
    pub blue: &'static str,
}

/// Initial colors as declared by `Theme.qml`, before the adaptive wallpaper
/// palette is loaded. These are the original shell defaults, not a new theme.
pub const INITIAL_PALETTE: Palette = Palette {
    id: "adaptive",
    name: "Adaptive (Wallpaper)",
    background: "#0c0c0c",
    surface: "#141414",
    surface_hover: "#202020",
    border: "#282828",
    text: "#ffffff",
    muted: "#8e8e93",
    accent: "#0a84ff",
    accent_hover: "#409cff",
    accent_text: "#ffffff",
    red: "#ff453a",
    green: "#32d74b",
    yellow: "#ffd60a",
    blue: "#0a84ff",
};

/// Curated palettes from `theme/Palettes.qml`, kept in one frontend catalog.
pub const CURATED_PALETTES: [Palette; 9] = [
    Palette {
        id: "catppuccin_mocha",
        name: "Catppuccin Mocha",
        background: "#1e1e2e",
        surface: "#181825",
        surface_hover: "#313244",
        border: "#45475a",
        text: "#cdd6f4",
        muted: "#a6adc8",
        accent: "#89b4fa",
        accent_hover: "#b4befe",
        accent_text: "#11111b",
        red: "#f38ba8",
        green: "#a6e3a1",
        yellow: "#f9e2af",
        blue: "#89b4fa",
    },
    Palette {
        id: "catppuccin_latte",
        name: "Catppuccin Latte",
        background: "#eff1f5",
        surface: "#e6e9ef",
        surface_hover: "#ccd0da",
        border: "#bcc0cc",
        text: "#4c4f69",
        muted: "#6c6f85",
        accent: "#1e66f5",
        accent_hover: "#04a5e5",
        accent_text: "#eff1f5",
        red: "#d20f39",
        green: "#40a02b",
        yellow: "#df8e1d",
        blue: "#1e66f5",
    },
    Palette {
        id: "tokyo_night",
        name: "Tokyo Night",
        background: "#1a1b26",
        surface: "#16161e",
        surface_hover: "#24283b",
        border: "#292e42",
        text: "#c0caf5",
        muted: "#7982a9",
        accent: "#7aa2f7",
        accent_hover: "#bb9af7",
        accent_text: "#15161e",
        red: "#f7768e",
        green: "#9ece6a",
        yellow: "#e0af68",
        blue: "#7aa2f7",
    },
    Palette {
        id: "gruvbox_dark",
        name: "Gruvbox Dark",
        background: "#282828",
        surface: "#1d2021",
        surface_hover: "#3c3836",
        border: "#504945",
        text: "#ebdbb2",
        muted: "#a89984",
        accent: "#d79921",
        accent_hover: "#fabd2f",
        accent_text: "#282828",
        red: "#fb4934",
        green: "#b8bb26",
        yellow: "#fabd2f",
        blue: "#83a598",
    },
    Palette {
        id: "nord",
        name: "Nord",
        background: "#2e3440",
        surface: "#242933",
        surface_hover: "#3b4252",
        border: "#434c5e",
        text: "#eceff4",
        muted: "#d8dee9",
        accent: "#88c0d0",
        accent_hover: "#81a1c1",
        accent_text: "#2e3440",
        red: "#bf616a",
        green: "#a3be8c",
        yellow: "#ebcb8b",
        blue: "#81a1c1",
    },
    Palette {
        id: "rose_pine",
        name: "Rosé Pine",
        background: "#191724",
        surface: "#1f1d2e",
        surface_hover: "#26233a",
        border: "#403d52",
        text: "#e0def4",
        muted: "#908caa",
        accent: "#ebbcba",
        accent_hover: "#f6c177",
        accent_text: "#191724",
        red: "#eb6f92",
        green: "#31748f",
        yellow: "#f6c177",
        blue: "#9ccfd8",
    },
    Palette {
        id: "cyberpunk",
        name: "Cyberpunk",
        background: "#0d0f18",
        surface: "#141726",
        surface_hover: "#21253b",
        border: "#ff007f",
        text: "#e0f7fa",
        muted: "#00e5ff",
        accent: "#ff007f",
        accent_hover: "#00e5ff",
        accent_text: "#ffffff",
        red: "#ff1744",
        green: "#00e676",
        yellow: "#ffea00",
        blue: "#00e5ff",
    },
    Palette {
        id: "bauhaus",
        name: "Bauhaus",
        background: "#1a1a1a",
        surface: "#242424",
        surface_hover: "#333333",
        border: "#444444",
        text: "#f5f5f5",
        muted: "#999999",
        accent: "#e52521",
        accent_hover: "#f5a623",
        accent_text: "#ffffff",
        red: "#e52521",
        green: "#2b82d9",
        yellow: "#f5a623",
        blue: "#2b82d9",
    },
    Palette {
        id: "crimson",
        name: "Crimson Moon",
        background: "#120a0d",
        surface: "#1c1015",
        surface_hover: "#2d1822",
        border: "#4a2133",
        text: "#f7e1ea",
        muted: "#ad8394",
        accent: "#e63946",
        accent_hover: "#ff4d6d",
        accent_text: "#ffffff",
        red: "#e63946",
        green: "#52b788",
        yellow: "#ee9b00",
        blue: "#a2d2ff",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceStyle {
    Classic,
    Glass,
    Liquid,
}

/// Shared visual state. Defaults correspond to the source settings schema:
/// classic surfaces, original bar dimensions, font stacks, and motion values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisualTheme {
    pub palette: Palette,
    pub surface_style: SurfaceStyle,
    pub bar_height: u16,
    pub bar_top_margin: u16,
    pub bar_side_margin: u16,
    pub capsule_spacing: u16,
    pub font_family: &'static str,
    pub font_mono: &'static str,
    pub motion_scale_percent: u16,
    pub duration_fast_ms: u16,
    pub duration_medium_ms: u16,
    pub duration_morph_ms: u16,
    pub radius_small: u8,
    pub radius_medium: u8,
    pub radius_large: u8,
    pub radius_pill: u16,
}

impl Default for VisualTheme {
    fn default() -> Self {
        Self {
            palette: INITIAL_PALETTE,
            surface_style: SurfaceStyle::Classic,
            bar_height: 32,
            bar_top_margin: 16,
            bar_side_margin: 18,
            capsule_spacing: 8,
            font_family: "Inter, Cantarell, SF Pro Text, sans-serif",
            font_mono: "JetBrainsMono Nerd Font Mono, monospace",
            motion_scale_percent: 100,
            duration_fast_ms: 140,
            duration_medium_ms: 200,
            duration_morph_ms: 380,
            radius_small: 8,
            radius_medium: 12,
            radius_large: 18,
            radius_pill: 999,
        }
    }
}

impl VisualTheme {
    /// GTK CSS for widgets that opt into the Velatura root class.
    pub fn css(&self) -> String {
        // Classic is the only enabled surface at this stage. The other style
        // names remain represented by the audited enum but need compositor
        // blur support and persisted settings before they can be rendered.
        let surface = match self.surface_style {
            SurfaceStyle::Classic => "#000000",
            SurfaceStyle::Glass | SurfaceStyle::Liquid => "#000000",
        };
        format!(
            "@define-color velatura-background {};\n\
             @define-color velatura-surface {};\n\
             @define-color velatura-hover {};\n\
             @define-color velatura-border {};\n\
             @define-color velatura-text {};\n\
             @define-color velatura-muted {};\n\
             @define-color velatura-accent {};\n\
             @define-color velatura-accent-hover {};\n\
             @define-color velatura-accent-text {};\n\
             .velatura-root {{ background-color: @velatura-background; color: @velatura-text; font-family: {}; }}\n\
             .velatura-root .velatura-island {{ background-color: {}; border: 1px solid #262626; border-radius: 999px; }}\n\
             .velatura-root .velatura-surface {{ background-color: @velatura-surface; border: 1px solid @velatura-border; border-radius: 12px; }}\n\
             .velatura-root label {{ color: @velatura-text; }}\n\
             .velatura-root .dim-label {{ color: @velatura-muted; }}\n\
             .velatura-root button {{ color: @velatura-text; background-color: @velatura-surface; border-color: @velatura-border; }}\n\
             .velatura-root button:hover {{ background-color: @velatura-hover; }}\n\
             .velatura-root button:focus-visible {{ outline: 1px solid @velatura-accent; }}\n",
            self.palette.background,
            self.palette.surface,
            self.palette.surface_hover,
            self.palette.border,
            self.palette.text,
            self.palette.muted,
            self.palette.accent,
            self.palette.accent_hover,
            self.palette.accent_text,
            self.font_family,
            surface,
        )
    }
}

/// Install the same application-wide CSS provider used by all Velatura
/// surfaces. Call once from the existing GTK activate callback.
pub fn install(display: &gdk::Display, theme: &VisualTheme) -> CssProvider {
    let provider = CssProvider::new();
    provider.load_from_data(&theme.css());
    gtk4::style_context_add_provider_for_display(
        display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    provider
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_defaults_and_motion_values_are_preserved() {
        let theme = VisualTheme::default();
        assert_eq!(theme.palette, INITIAL_PALETTE);
        assert_eq!(theme.surface_style, SurfaceStyle::Classic);
        assert_eq!(
            (
                theme.bar_height,
                theme.bar_top_margin,
                theme.bar_side_margin
            ),
            (32, 16, 18)
        );
        assert_eq!(
            (
                theme.duration_fast_ms,
                theme.duration_medium_ms,
                theme.duration_morph_ms
            ),
            (140, 200, 380)
        );
        assert_eq!(
            theme.font_family,
            "Inter, Cantarell, SF Pro Text, sans-serif"
        );
        assert_eq!(theme.font_mono, "JetBrainsMono Nerd Font Mono, monospace");
    }

    #[test]
    fn curated_palette_catalog_matches_original_ids_and_contains_no_placeholder() {
        let ids: Vec<_> = CURATED_PALETTES.iter().map(|palette| palette.id).collect();
        assert_eq!(
            ids,
            [
                "catppuccin_mocha",
                "catppuccin_latte",
                "tokyo_night",
                "gruvbox_dark",
                "nord",
                "rose_pine",
                "cyberpunk",
                "bauhaus",
                "crimson",
            ]
        );
        assert!(
            CURATED_PALETTES
                .iter()
                .all(|palette| palette.accent.starts_with('#'))
        );
    }

    #[test]
    fn generated_css_uses_shared_semantic_tokens_and_classic_surface() {
        let css = VisualTheme::default().css();
        assert!(css.contains("@define-color velatura-accent #0a84ff;"));
        assert!(css.contains("background-color: #000000; border: 1px solid #262626"));
        assert!(css.contains(".velatura-root button:focus-visible"));
    }
}
