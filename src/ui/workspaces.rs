//! `WorkspacesWidget.qml`: the workspaces Niri reports for this output, as one
//! capsule. Focused, occupied and empty differ by shape and weight alone:
//! dots, bars, rings, or the written numbers on a sliding accent pill.

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib, graphene, gsk, pango};
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::motion::{Curve, Tween};
use super::palette;
use super::settings::Settings;
use super::tokens::{self, Rgba};
use crate::niri_session::{NiriSession, NiriSessionSnapshot};
use std::sync::Arc;

const DOT: f64 = 6.0;
const ACTIVE_WIDTH: f64 = 22.0;
const BAR_WIDTH: f64 = 3.0;
const RING: f64 = 7.0;
const GLYPH_PAD: f64 = 6.0;
const INDICATOR_DIM: Rgba = Rgba::hex(0x4d, 0x4d, 0x4d);

pub struct Slot {
    id: i64,
    idx: i64,
    focused: bool,
    occupied: bool,
    mark: Tween,
    height: Tween,
    lit: Tween,
    full: Tween,
    occ: Tween,
    glyph_width: Cell<f64>,
}

#[derive(Default)]
pub struct State {
    slots: Vec<Slot>,
    hovered: Option<usize>,
    pill_x: Option<Tween>,
    pill_w: Option<Tween>,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Strip {
        pub state: RefCell<State>,
        pub settings: RefCell<Option<Rc<Settings>>>,
        pub style: RefCell<String>,
        pub output: RefCell<String>,
        pub session: RefCell<Option<Arc<NiriSession>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Strip {
        const NAME: &'static str = "VelaturaWorkspaceStrip";
        type Type = super::Strip;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Strip {}

    impl WidgetImpl for Strip {
        fn measure(&self, orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let extent = if orientation == gtk4::Orientation::Horizontal {
                self.obj().content_width().round() as i32
            } else {
                self.settings.borrow().as_ref().map_or(32, |s| s.bar_height)
            };
            (extent, extent, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            self.obj().paint(snapshot);
        }
    }
}

glib::wrapper! {
    pub struct Strip(ObjectSubclass<imp::Strip>) @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

fn blend(from: Rgba, to: Rgba, t: f64) -> Rgba {
    let t = t as f32;
    Rgba(
        from.0 + (to.0 - from.0) * t,
        from.1 + (to.1 - from.1) * t,
        from.2 + (to.2 - from.2) * t,
        from.3 + (to.3 - from.3) * t,
    )
}

fn fill_round(snapshot: &gtk4::Snapshot, x: f64, y: f64, w: f64, h: f64, color: Rgba) {
    if w <= 0.0 || h <= 0.0 || color.3 <= 0.0 {
        return;
    }
    let bounds = graphene::Rect::new(x as f32, y as f32, w as f32, h as f32);
    let r = (w.min(h) / 2.0) as f32;
    let size = graphene::Size::new(r, r);
    snapshot.push_rounded_clip(&gsk::RoundedRect::new(bounds, size, size, size, size));
    snapshot.append_color(&color.gdk(), &bounds);
    snapshot.pop();
}

impl Strip {
    pub fn new(settings: Rc<Settings>, output: &str, session: Arc<NiriSession>) -> Self {
        let strip: Self = glib::Object::new();
        let imp = strip.imp();
        *imp.style.borrow_mut() = "dots".into();
        *imp.output.borrow_mut() = output.to_string();
        *imp.settings.borrow_mut() = Some(settings);
        *imp.session.borrow_mut() = Some(session);
        strip.set_valign(gtk4::Align::Start);

        let motion = gtk4::EventControllerMotion::new();
        let weak = strip.downgrade();
        motion.connect_motion(move |_, x, _| {
            if let Some(strip) = weak.upgrade() {
                strip.set_hover(strip.slot_at(x));
            }
        });
        let weak = strip.downgrade();
        motion.connect_leave(move |_| {
            if let Some(strip) = weak.upgrade() {
                strip.set_hover(None);
            }
        });
        strip.add_controller(motion);

        let click = gtk4::GestureClick::new();
        let weak = strip.downgrade();
        click.connect_released(move |_, _, x, _| {
            if let Some(strip) = weak.upgrade() {
                strip.focus_at(x);
            }
        });
        strip.add_controller(click);

        palette::on_change({
            let weak = strip.downgrade();
            move || {
                if let Some(strip) = weak.upgrade() {
                    strip.queue_draw();
                }
            }
        });
        strip
    }

    pub fn set_style(&self, style: &str) {
        *self.imp().style.borrow_mut() = style.to_string();
        self.queue_resize();
    }

    fn style(&self) -> String {
        self.imp().style.borrow().clone()
    }

    fn written(&self) -> bool {
        matches!(
            self.style().as_str(),
            "numbers" | "roman" | "kanji" | "greek"
        )
    }

    fn settings(&self) -> Rc<Settings> {
        self.imp()
            .settings
            .borrow()
            .clone()
            .expect("strip has settings")
    }

    fn spacing(&self) -> f64 {
        if self.written() { 2.0 } else { 8.0 }
    }

    fn pill_height(&self) -> f64 {
        20f64.min(f64::from(self.settings().bar_height) - 10.0)
    }

    fn glyph_of(&self, n: i64) -> String {
        match self.style().as_str() {
            "roman" => {
                let mut left = n;
                let mut out = String::new();
                for (value, mark) in [(10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")] {
                    while left >= value {
                        out.push_str(mark);
                        left -= value;
                    }
                }
                out
            }
            "kanji" => {
                let digits = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
                if n < 10 {
                    digits[n.max(0) as usize].to_string()
                } else {
                    let tens = if n >= 20 {
                        digits[(n / 10) as usize]
                    } else {
                        ""
                    };
                    format!("{tens}十{}", digits[(n % 10) as usize])
                }
            }
            "greek" => "αβγδεζηθικλμνξοπρστυφχψω"
                .chars()
                .nth((n - 1).max(0) as usize)
                .map_or_else(|| n.to_string(), |c| c.to_string()),
            _ => n.to_string(),
        }
    }

    fn layout_of(&self, text: &str) -> pango::Layout {
        let layout = self.create_pango_layout(Some(text));
        let mut font = pango::FontDescription::new();
        font.set_family(&super::css::font_of(&self.settings().font_family));
        font.set_weight(pango::Weight::Semibold);
        font.set_absolute_size(f64::from(tokens::FONT_SIZE_SMALL) * f64::from(pango::SCALE));
        layout.set_font_description(Some(&font));
        layout
    }

    fn mark_target(&self, slot: &Slot) -> f64 {
        match self.style().as_str() {
            "bars" => BAR_WIDTH,
            "rings" => RING + 2.0,
            "dots" => {
                if slot.focused {
                    ACTIVE_WIDTH
                } else {
                    DOT
                }
            }
            _ => self
                .pill_height()
                .max(slot.glyph_width.get() + 2.0 * GLYPH_PAD),
        }
    }

    fn content_width(&self) -> f64 {
        let state = self.imp().state.borrow();
        let strip: f64 = state
            .slots
            .iter()
            .map(|s| s.mark.value() + self.spacing())
            .sum();
        (strip - self.spacing()).max(0.0) + if self.written() { 12.0 } else { 20.0 }
    }

    fn pad(&self) -> f64 {
        if self.written() { 6.0 } else { 10.0 }
    }

    fn slot_at(&self, x: f64) -> Option<usize> {
        let state = self.imp().state.borrow();
        let mut cursor = self.pad();
        for (index, slot) in state.slots.iter().enumerate() {
            let width = slot.mark.value() + self.spacing();
            if x >= cursor && x < cursor + width {
                return Some(index);
            }
            cursor += width;
        }
        None
    }

    fn set_hover(&self, hovered: Option<usize>) {
        let imp = self.imp();
        {
            let mut state = imp.state.borrow_mut();
            if state.hovered == hovered {
                return;
            }
            state.hovered = hovered;
            let fast = tokens::DURATION_FAST * self.settings().motion();
            for (index, slot) in state.slots.iter().enumerate() {
                let over = hovered == Some(index);
                slot.full.animate(
                    f64::from(u8::from(over || slot.focused)),
                    fast,
                    Curve::OutCubic,
                );
                slot.lit.animate(
                    f64::from(u8::from(over || slot.focused || slot.occupied)),
                    fast,
                    Curve::OutCubic,
                );
            }
        }
        self.set_cursor_from_name(hovered.map(|_| "pointer"));
    }

    fn focus_at(&self, x: f64) {
        let Some(index) = self.slot_at(x) else { return };
        let (idx, output) = {
            let state = self.imp().state.borrow();
            (state.slots[index].idx, self.imp().output.borrow().clone())
        };
        if idx <= 0 {
            return;
        }
        if let Some(session) = self.imp().session.borrow().as_ref() {
            // `focusWorkspaceOn`: another output is focused first.
            if !output.is_empty() {
                let _ = session.dispatch(["focus-monitor".to_string(), output]);
            }
            let _ = session.dispatch(["focus-workspace".to_string(), idx.to_string()]);
        }
    }

    pub fn update(&self, snapshot: &NiriSessionSnapshot) {
        let output = self.imp().output.borrow().clone();
        let mut items: Vec<&Value> = snapshot
            .niri
            .workspaces
            .iter()
            .filter(|ws| output.is_empty() || ws["output"].as_str() == Some(output.as_str()))
            .collect();
        items.sort_by_key(|ws| ws["idx"].as_i64().unwrap_or(0));
        let active = items
            .iter()
            .find(|ws| ws["is_active"].as_bool() == Some(true))
            .and_then(|ws| ws["idx"].as_i64())
            .unwrap_or(0);

        let settings = self.settings();
        let medium = tokens::DURATION_MEDIUM * settings.motion();
        let fast = tokens::DURATION_FAST * settings.motion();
        let curve = Curve::from_id(&settings.motion_curve);
        let written = self.written();
        let this = self.downgrade();
        let redraw = move |_: f64| {
            if let Some(strip) = this.upgrade() {
                strip.queue_resize();
                strip.queue_draw();
            }
        };

        let mut slots = std::mem::take(&mut self.imp().state.borrow_mut().slots);
        let mut next = Vec::new();
        let hovered_id = {
            let state = self.imp().state.borrow();
            state.hovered.and_then(|i| slots.get(i)).map(|s| s.id)
        };
        for ws in items {
            let id = ws["id"].as_i64().unwrap_or(0);
            let idx = ws["idx"].as_i64().unwrap_or(0);
            let focused = idx == active;
            let occupied = snapshot
                .niri
                .windows
                .iter()
                .any(|window| window["workspace_id"].as_i64() == Some(id));
            let mut slot = match slots.iter().position(|s| s.id == id) {
                Some(position) => slots.remove(position),
                None => {
                    let make = |value: f64| Tween::new(self, value, redraw.clone());
                    Slot {
                        id,
                        idx,
                        focused,
                        occupied,
                        mark: make(0.0),
                        height: make(0.0),
                        lit: make(f64::from(u8::from(focused || occupied))),
                        full: make(f64::from(u8::from(focused))),
                        occ: make(f64::from(u8::from(occupied))),
                        glyph_width: Cell::new(0.0),
                    }
                }
            };
            slot.idx = idx;
            slot.focused = focused;
            slot.occupied = occupied;
            if written {
                let text = self.glyph_of(idx);
                slot.glyph_width
                    .set(f64::from(self.layout_of(&text).pixel_size().0));
            }
            let hover = hovered_id == Some(id);
            let mark = self.mark_target(&slot);
            if slot.mark.value() == 0.0 && slot.mark.settled() {
                slot.mark.jump(mark);
                slot.height.jump(self.height_target(&slot));
            } else {
                slot.mark.animate(mark, medium, curve);
                slot.height
                    .animate(self.height_target(&slot), medium, curve);
            }
            slot.full
                .animate(f64::from(u8::from(focused || hover)), fast, Curve::OutCubic);
            slot.lit.animate(
                f64::from(u8::from(focused || hover || occupied)),
                fast,
                Curve::OutCubic,
            );
            slot.occ
                .animate(f64::from(u8::from(occupied)), fast, Curve::OutCubic);
            next.push(slot);
        }

        let mut guard = self.imp().state.borrow_mut();
        let state = &mut *guard;
        state.slots = next;
        if written {
            let target = state.slots.iter().position(|s| s.focused).map(|index| {
                let before: f64 = state.slots[..index]
                    .iter()
                    .map(|s| s.mark.target() + self.spacing())
                    .sum();
                (
                    before + self.pad() + self.spacing() / 2.0,
                    state.slots[index].mark.target(),
                )
            });
            if let Some((x, width)) = target {
                let pill_x = state
                    .pill_x
                    .get_or_insert_with(|| Tween::new(self, x, redraw.clone()));
                let pill_w = state
                    .pill_w
                    .get_or_insert_with(|| Tween::new(self, width, redraw.clone()));
                pill_x.animate(x, medium, curve);
                pill_w.animate(width, medium, curve);
            }
        }
        drop(guard);
        self.queue_resize();
        self.queue_draw();
    }

    fn height_target(&self, slot: &Slot) -> f64 {
        match self.style().as_str() {
            "bars" => {
                if slot.focused {
                    14.0
                } else if slot.occupied {
                    9.0
                } else {
                    5.0
                }
            }
            "rings" => {
                if slot.focused {
                    RING + 2.0
                } else {
                    RING
                }
            }
            _ => DOT,
        }
    }

    fn paint(&self, snapshot: &gtk4::Snapshot) {
        let settings = self.settings();
        let palette = palette::current();
        let (width, height) = (f64::from(self.width()), f64::from(self.height()));
        if width <= 0.0 {
            return;
        }

        // The capsule: ground and a one-pixel rim.
        let style = settings.surface();
        fill_round(snapshot, 0.0, 0.0, width, height, tokens::ground_of(style));
        let bounds = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
        let r = (height / 2.0) as f32;
        let size = graphene::Size::new(r, r);
        let outline = gsk::RoundedRect::new(bounds, size, size, size, size);
        let rim: gdk::RGBA = tokens::rim_of(style).gdk();
        snapshot.append_border(&outline, &[1.0; 4], &[rim; 4]);

        let state = self.imp().state.borrow();
        let written = self.written();
        let style = self.style();
        let mid = height / 2.0;
        let spacing = self.spacing();

        if written {
            if let (Some(x), Some(w)) = (&state.pill_x, &state.pill_w) {
                let pill = self.pill_height();
                fill_round(
                    snapshot,
                    x.value(),
                    mid - pill / 2.0,
                    w.value(),
                    pill,
                    palette.accent,
                );
            }
        }

        let mut cursor = self.pad();
        for (index, slot) in state.slots.iter().enumerate() {
            let mark = slot.mark.value();
            let slot_width = mark + spacing;
            let (lit, full, occ) = (slot.lit.value(), slot.full.value(), slot.occ.value());
            let centre = cursor + slot_width / 2.0;
            match style.as_str() {
                "dots" | "bars" => {
                    let w = if style == "bars" {
                        BAR_WIDTH
                    } else {
                        (slot_width - spacing).max(0.0)
                    };
                    let h = slot.height.value();
                    let mut color = blend(INDICATOR_DIM, palette.accent, lit);
                    color.3 = (1.0 - 0.45 * occ * (1.0 - full)) as f32;
                    fill_round(snapshot, centre - w / 2.0, mid - h / 2.0, w, h, color);
                }
                "rings" => {
                    let d = slot.height.value();
                    let (x, y) = (centre - d / 2.0, mid - d / 2.0);
                    let fill_alpha = full.max(occ * 0.55);
                    let mut fill = palette.accent;
                    fill.3 = fill_alpha as f32;
                    fill_round(snapshot, x, y, d, d, fill);
                    let ring = graphene::Rect::new(x as f32, y as f32, d as f32, d as f32);
                    let rr = (d / 2.0) as f32;
                    let rs = graphene::Size::new(rr, rr);
                    let mut edge = INDICATOR_DIM;
                    edge.3 = (1.0 - fill_alpha) as f32;
                    let rounded = gsk::RoundedRect::new(ring, rs, rs, rs, rs);
                    snapshot.append_border(&rounded, &[1.5; 4], &[edge.gdk(); 4]);
                }
                _ => {
                    let layout = self.layout_of(&self.glyph_of(slot.idx));
                    let (gw, gh) = layout.pixel_size();
                    let color = if slot.focused {
                        palette.accent_text
                    } else if state.hovered == Some(index) {
                        palette.accent
                    } else if slot.occupied {
                        palette.text
                    } else {
                        INDICATOR_DIM
                    };
                    snapshot.save();
                    snapshot.translate(&graphene::Point::new(
                        (centre - f64::from(gw) / 2.0) as f32,
                        (mid - f64::from(gh) / 2.0) as f32,
                    ));
                    snapshot.append_layout(&layout, &color.gdk());
                    snapshot.restore();
                }
            }
            cursor += slot_width;
        }
    }
}
