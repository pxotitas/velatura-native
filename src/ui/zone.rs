//! `BarZone.qml`: one side of the bar, its capsule groups in layout order.
//! The workspaces get a capsule of their own; chips share one until a
//! `split`. A group with nothing present is not drawn, as in the original.
//!
//! Ported modules so far: `workspaces`, `launcher`, and the first system
//! detail panels (`network`, `bluetooth`, `volume`, `battery`). Tray remains
//! pending until its DBus model and context menus are ported.

use gtk4::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

use super::chip::ButtonChip;
use super::settings::Settings;
use super::tokens;
use super::workspaces::Strip;
use crate::niri_session::{NiriSession, NiriSessionSnapshot};

pub struct Zone {
    widget: gtk4::Box,
    strip: Option<Strip>,
}

/// Glyphs of the button modules (`ModuleService.buttons`).
fn button_glyph(id: &str) -> Option<&'static str> {
    match id {
        "launcher" => Some("\u{f0349}"),
        // Material Design Icons used by the original Quickshell modules.
        "network" => Some("\u{f01ee}"),
        "bluetooth" => Some("\u{f00af}"),
        "volume" => Some("\u{f057e}"),
        "battery" => Some("\u{f0079}"),
        _ => None,
    }
}

impl Zone {
    pub fn new(
        settings: &Rc<Settings>,
        side: &str,
        output: &str,
        session: Arc<NiriSession>,
        open_panel: Rc<dyn Fn(&str)>,
    ) -> Option<Self> {
        let entries = if side == "left" {
            &settings.bar_left
        } else {
            &settings.bar_right
        };
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, tokens::CAPSULE_SPACING);
        row.set_height_request(settings.bar_height);
        let mut strip = None;
        let mut chips: Vec<&str> = Vec::new();
        let mut empty = true;

        let flush = |chips: &mut Vec<&str>, row: &gtk4::Box, empty: &mut bool| {
            // One button alone is its own capsule; shared capsules arrive
            // with the first non-button chip.
            let present: Vec<&str> = chips
                .drain(..)
                .filter(|id| button_glyph(id).is_some())
                .collect();
            for id in present {
                let panel = match id {
                    "network" => "wifi",
                    "volume" => "sound",
                    other => other,
                }
                .to_string();
                let open = Rc::clone(&open_panel);
                let chip = ButtonChip::new(settings, button_glyph(id).unwrap_or(""), move || {
                    open(&panel)
                });
                row.append(&chip.widget);
                *empty = false;
            }
        };

        for item in entries {
            match item.as_str() {
                "split" => flush(&mut chips, &row, &mut empty),
                "workspaces" => {
                    flush(&mut chips, &row, &mut empty);
                    let built = Strip::new(Rc::clone(settings), output, Arc::clone(&session));
                    built.set_style(&settings.workspace_style);
                    row.append(&built);
                    strip = Some(built);
                    empty = false;
                }
                "tray" => flush(&mut chips, &row, &mut empty),
                id => chips.push(id),
            }
        }
        flush(&mut chips, &row, &mut empty);
        if empty {
            return None;
        }
        Some(Self { widget: row, strip })
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.widget
    }

    pub fn width(&self) -> i32 {
        self.widget.measure(gtk4::Orientation::Horizontal, -1).1
    }

    pub fn update(&self, snapshot: &NiriSessionSnapshot) {
        if let Some(strip) = &self.strip {
            strip.update(snapshot);
        }
    }
}
