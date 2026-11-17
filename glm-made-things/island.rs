//! Dynamic Island: one capsule that changes shape to fit whatever it shows.
//!
//! Port of `bar/widgets/DynamicIsland.qml` + `IslandState.qml`. The island
//! arbitrates its layer (`services::island_state`), sizes the capsule for
//! that layer before loading it, and animates the shape while contents
//! cross-fade. Motion tokens come from `Theme.qml` (morph 380 ms, medium
//! 200 ms, fast 140 ms, OutCubic).
//!
//! The island is owned as `Rc<RefCell<Island>>` by the bar surface; timers
//! capture `Rc` clones, which is safe because the timers are short-lived and
//! always removed or finished before the bar can drop the island.

use gtk4::gdk;
use gtk4::glib;
use gtk4::pango;
use gtk4::prelude::*;
use gtk4::{Button, GestureClick, Image, Label, Orientation, Stack, StackTransitionType};

use crate::services::island_state::{IslandState as Arbiter, Layer, TransientEvent, World};
use crate::services::notifications::Toast;

use super::launcher::LauncherPanel;
use super::theme;

/// Layer sizes, one entry per layer (`layerSizeFor`). Every value comes from
/// the original's declared metrics.
fn layer_size(layer: Layer, toast: Option<&Toast>, open_panel: Option<&str>) -> (i32, i32) {
    match layer {
        // At rest the island holds the clock core; activities widen it in
        // the original. Without the activity modules ported yet, rest is the
        // bare clock.
        Layer::Modules => (72, theme::CAPSULE_HEIGHT),
        Layer::Summary => (420, 150),
        Layer::Osd => (260, theme::CAPSULE_HEIGHT),
        Layer::Notification => match toast {
            Some(toast) => (Toast::toast_width(), toast.toast_height()),
            None => (400, 78),
        },
        Layer::Panel => match open_panel {
            Some("launcher") => (
                super::launcher::panel_width(),
                super::launcher::panel_height(),
            ),
            Some("notifications") => (420, 500),
            _ => (72, theme::CAPSULE_HEIGHT),
        },
    }
}

/// Easing: OutCubic, the original's fallback curve.
fn out_cubic(t: f64) -> f64 {
    let one_minus = 1.0 - t;
    1.0 - one_minus * one_minus * one_minus
}

pub struct Island {
    /// The morphing capsule.
    pub morph: super::morph::MorphBox,
    /// Content stack with cross-fade between layers.
    stack: Stack,
    rest_clock: Label,
    rest_date: Label,
    summary_time: Label,
    summary_date: Label,
    osd_icon: Image,
    osd_label: Label,
    osd_bar: gtk4::ProgressBar,
    notification_title: Label,
    notification_body: Label,
    notification_buttons: gtk4::Box,
    launcher_panel: LauncherPanel,
    notifications_list: gtk4::Box,
    /// Arbitration state machine.
    arbiter: Arbiter,
    /// The layer as of the last refresh, for the bar's surface sizing.
    last_layer: Layer,
    /// Whether a notification is on the island.
    current_toast: Option<Toast>,
    hover: HoverState,
    /// The morph animation source, replaced on every target change.
    animation: TimerSlot,
    current_width: f64,
    current_height: f64,
    /// The notifications history, kept for the panel.
    history: Vec<crate::services::notifications::HistoryEntry>,
    services: super::shell::ServiceHandles,
    /// The monitor's Niri connector, as reported by GDK.
    connector: Option<String>,
}

/// A timer handle that stays valid after the timer fires. A fired
/// one-shot's GLib source destroys itself, and cancelling a destroyed
/// source is an error under glib 0.22, so the callback clears its slot
/// first and only pending sources are ever removed.
#[derive(Clone, Default)]
struct TimerSlot(std::rc::Rc<std::cell::RefCell<Option<glib::SourceId>>>);

impl TimerSlot {
    fn set(&self, id: glib::SourceId) {
        *self.0.borrow_mut() = Some(id);
    }

    /// Called from inside the timer's own callback: the source is dispatching
    /// and destroys itself afterwards, so the slot only stops tracking it.
    fn clear(&self) {
        *self.0.borrow_mut() = None;
    }

    fn cancel(&self) {
        if let Some(id) = self.0.borrow_mut().take() {
            id.remove();
        }
    }
}

#[derive(Default)]
struct HoverState {
    pointer_inside: bool,
    /// Sticky "the pointer has been over the island" (QML `pointerEntered`):
    /// survives leaving, reset only when the open panel changes.
    pointer_entered: bool,
    dwell: TimerSlot,
    grace: TimerSlot,
    leave_close: TimerSlot,
    summary_held: bool,
}

impl Island {
    /// Build the island and wrap it in the shared handle its timers use.
    /// The controllers need the `Rc`, so they attach after construction.
    pub fn new(
        services: super::shell::ServiceHandles,
        connector: Option<String>,
        active: bool,
    ) -> std::rc::Rc<std::cell::RefCell<Self>> {
        let morph = super::morph::MorphBox::new();
        morph.set_css_classes(&["velatura-island"]);

        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::Crossfade);
        stack.set_transition_duration(theme::DURATION_FAST_MS as u32);
        stack.set_vhomogeneous(false);
        stack.set_hhomogeneous(false);
        morph.child().append(&stack);

        // ── REST ────────────────────────────────────────────────────────────
        let rest_box = gtk4::Box::new(Orientation::Horizontal, 6);
        let rest_clock = Label::new(None);
        rest_clock.set_css_classes(&["velatura-module-label"]);
        let rest_date = Label::new(None);
        rest_date.set_css_classes(&["velatura-module-muted"]);
        rest_box.append(&rest_clock);
        rest_box.append(&rest_date);
        rest_box.set_halign(gtk4::Align::Center);
        rest_box.set_valign(gtk4::Align::Center);
        stack.add_named(&rest_box, Some("modules"));

        // ── SUMMARY ─────────────────────────────────────────────────────────
        // The clock glance: the time large at the far end, a square face of
        // glanceFaceHeight 150 in the original.
        let summary_box = gtk4::Box::new(Orientation::Vertical, 0);
        let summary_time = Label::new(None);
        summary_time.set_attributes(Some(&{
            let attrs = pango::AttrList::new();
            attrs.insert(pango::AttrSize::new(62 * pango::SCALE));
            attrs.insert(pango::AttrInt::new_weight(pango::Weight::Ultraheavy));
            attrs
        }));
        let summary_date = Label::new(None);
        summary_date.set_css_classes(&["velatura-muted"]);
        summary_box.append(&summary_time);
        summary_box.append(&summary_date);
        summary_box.set_halign(gtk4::Align::Center);
        summary_box.set_valign(gtk4::Align::Center);
        stack.add_named(&summary_box, Some("summary"));

        // ── OSD ─────────────────────────────────────────────────────────────
        let osd_box = gtk4::Box::new(Orientation::Horizontal, 10);
        let osd_icon = Image::new();
        let osd_label = Label::new(None);
        osd_label.set_css_classes(&["velatura-module-label"]);
        let osd_bar = gtk4::ProgressBar::new();
        osd_bar.set_css_classes(&["velatura-osd-level"]);
        osd_bar.set_hexpand(true);
        osd_box.append(&osd_icon);
        osd_box.append(&osd_label);
        osd_box.append(&osd_bar);
        osd_box.set_valign(gtk4::Align::Center);
        stack.add_named(&osd_box, Some("osd"));

        // ── NOTIFICATION ────────────────────────────────────────────────────
        // One row, as an incoming call is on a phone's island: the text, and
        // up to two short buttons at the end; longer sets stack under it.
        let notification_box = gtk4::Box::new(Orientation::Vertical, 8);
        let toast_row = gtk4::Box::new(Orientation::Horizontal, 12);
        let notification_title = Label::new(None);
        notification_title.set_css_classes(&["velatura-notification-title"]);
        notification_title.set_hexpand(true);
        notification_title.set_halign(gtk4::Align::Start);
        notification_title.set_ellipsize(pango::EllipsizeMode::End);
        let notification_body = Label::new(None);
        notification_body.set_css_classes(&["velatura-notification-body"]);
        notification_body.set_halign(gtk4::Align::Start);
        notification_body.set_ellipsize(pango::EllipsizeMode::End);
        let text_column = gtk4::Box::new(Orientation::Vertical, 2);
        text_column.append(&notification_title);
        text_column.append(&notification_body);
        toast_row.append(&text_column);
        let notification_buttons = gtk4::Box::new(Orientation::Horizontal, 8);
        notification_buttons.set_halign(gtk4::Align::End);
        notification_box.append(&toast_row);
        notification_box.append(&notification_buttons);
        notification_box.set_valign(gtk4::Align::Center);
        stack.add_named(&notification_box, Some("notification"));

        // ── PANELS ──────────────────────────────────────────────────────────
        let launcher_panel = LauncherPanel::new(services.clone(), services.panel_requests.clone());
        stack.add_named(launcher_panel.root(), Some("panel:launcher"));
        let notifications_panel = gtk4::Box::new(Orientation::Vertical, 8);
        let notifications_list = gtk4::Box::new(Orientation::Vertical, 4);
        notifications_panel.append(&notifications_list);
        stack.add_named(&notifications_panel, Some("panel:notifications"));

        let mut island = Self {
            morph,
            stack,
            rest_clock,
            rest_date,
            summary_time,
            summary_date,
            osd_icon,
            osd_label,
            osd_bar,
            notification_title,
            notification_body,
            notification_buttons,
            launcher_panel,
            notifications_list,
            arbiter: Arbiter::new(active),
            last_layer: Layer::Modules,
            current_toast: None,
            hover: HoverState::default(),
            animation: TimerSlot::default(),
            current_width: 72.0,
            current_height: theme::CAPSULE_HEIGHT as f64,
            history: Vec::new(),
            services,
            connector,
        };

        // The first frame does not animate in from nothing.
        island.apply_size(72.0, theme::CAPSULE_HEIGHT as f64);
        island.refresh(false);
        let island = std::rc::Rc::new(std::cell::RefCell::new(island));

        // Click at rest opens the launcher panel until the control centre
        // lands; Escape closes any panel.
        let click = GestureClick::new();
        let weak_click = std::rc::Rc::clone(&island);
        click.connect_pressed(move |_gesture, _n, _x, _y| {
            let mut island = weak_click.borrow_mut();
            if island.arbiter.layer(island.world()) == Layer::Modules {
                let panel = island.arbiter.open("launcher");
                if panel {
                    island.services.notifications.dismiss();
                }
                island.refresh(false);
            }
        });
        island.borrow().morph.add_controller(click);
        let key = gtk4::EventControllerKey::new();
        let weak_key = std::rc::Rc::clone(&island);
        key.connect_key_pressed(move |_controller, key, _code, _state| {
            if key == gdk::Key::Escape {
                let mut island = weak_key.borrow_mut();
                island.arbiter.close();
                island.launcher_panel.on_closed();
                island.refresh(false);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        island.borrow().morph.add_controller(key);

        // Hover dwell opens the summary, leave closes it after a grace
        // period; a panel the pointer has entered closes when it leaves.
        let motion = gtk4::EventControllerMotion::new();
        let weak_enter = std::rc::Rc::clone(&island);
        motion.connect_enter(move |_controller, _x, _y| {
            Self::pointer_entered(&weak_enter);
        });
        let weak_leave = std::rc::Rc::clone(&island);
        motion.connect_leave(move |_controller| {
            Self::pointer_left(&weak_leave);
        });
        island.borrow().morph.add_controller(motion);

        island
    }

    fn world(&self) -> World {
        World {
            notification_active: self.current_toast.is_some(),
            notification_critical: self.current_toast.as_ref().is_some_and(|toast| {
                toast.urgency == crate::services::notifications::Urgency::Critical
            }),
        }
    }

    /// Whether this island is the one being worked on.
    pub fn is_live(&self) -> bool {
        self.arbiter.active
    }

    /// The resolved layer, as of the last refresh.
    pub fn layer(&self) -> Layer {
        self.last_layer
    }

    /// The Niri session behind the services, for widget actions.
    pub fn niri(&self) -> std::sync::Arc<crate::niri_session::NiriSession> {
        std::sync::Arc::clone(&self.services.niri)
    }

    /// The Niri output connector this bar's monitor belongs to, when GDK
    /// reported one. The shell uses it to follow the focused output.
    pub fn output_connector(&self) -> Option<String> {
        self.connector.clone()
    }

    /// Open a panel; a panel takes over immediately and dismisses whatever
    /// was flashing, per `IslandState.open`.
    pub fn open(&mut self, panel: &str) {
        if self.arbiter.open(panel) {
            self.services.notifications.dismiss();
        }
        // Per DynamicIsland's onOpenPanelChanged: a panel change stops the
        // leave-close and re-arms the sticky entered flag for the new panel.
        self.hover.leave_close.cancel();
        self.hover.pointer_entered = self.hover.pointer_inside && !panel.is_empty();
        self.refresh(false);
    }

    pub fn close(&mut self) {
        self.arbiter.close();
        self.launcher_panel.on_closed();
        self.hover.leave_close.cancel();
        self.hover.pointer_entered = false;
        self.refresh(false);
    }

    pub fn set_active(&mut self, active: bool) {
        self.arbiter.set_active(active);
        self.refresh(false);
    }

    pub fn tick(&mut self, now_ms: u64) {
        if self.arbiter.tick(now_ms) {
            self.refresh(false);
        }
    }

    pub fn request_osd(&mut self, event: TransientEvent) {
        self.arbiter.request_osd(crate::services::now_ms(), event);
        self.refresh(false);
    }

    /// A notification took the island.
    pub fn notification_arrived(&mut self, toast: Toast) {
        self.current_toast = Some(toast.clone());
        self.refresh(false);
    }

    /// The current notification left the island.
    pub fn notification_gone(&mut self) {
        if self.current_toast.take().is_some() {
            self.refresh(false);
        }
    }

    pub fn history_changed(&mut self, history: Vec<crate::services::notifications::HistoryEntry>) {
        self.history = history;
        if self.arbiter.open_panel.as_deref() == Some("notifications") {
            self.fill_notifications_list();
        }
    }

    fn fill_notifications_list(&mut self) {
        while let Some(child) = self.notifications_list.first_child() {
            self.notifications_list.remove(&child);
        }
        if self.history.is_empty() {
            let empty = Label::new(Some("No notifications"));
            empty.set_css_classes(&["velatura-muted"]);
            self.notifications_list.append(&empty);
        }
        for entry in self.history.iter().take(12) {
            let row = gtk4::Box::new(Orientation::Horizontal, 8);
            row.set_css_classes(&["velatura-launcher-row"]);
            let title = Label::new(Some(&entry.summary));
            title.set_css_classes(&["velatura-launcher-name"]);
            title.set_halign(gtk4::Align::Start);
            title.set_ellipsize(pango::EllipsizeMode::End);
            title.set_hexpand(true);
            let app = Label::new(Some(&entry.app_name));
            app.set_css_classes(&["velatura-launcher-subtitle"]);
            let remove = Button::new();
            remove.set_icon_name("window-close-symbolic");
            remove.set_css_classes(&["velatura-notification-button"]);
            let id = entry.id;
            let services = self.services.clone();
            remove.connect_clicked(move |_| {
                services.notifications.remove_from_history(id);
            });
            row.append(&title);
            row.append(&app);
            row.append(&remove);
            self.notifications_list.append(&row);
        }
        // Clear everything, at the end like the original's list.
        let clear = Button::new();
        clear.set_label("Clear");
        clear.set_css_classes(&["velatura-notification-button"]);
        let services = self.services.clone();
        clear.connect_clicked(move |_| {
            services.notifications.clear_history();
        });
        self.notifications_list.append(&clear);
    }

    fn pointer_entered(this: &std::rc::Rc<std::cell::RefCell<Self>>) {
        let mut self_ = this.borrow_mut();
        self_.hover.pointer_inside = true;
        self_.hover.pointer_entered = true;
        self_.hover.leave_close.cancel();
        self_.hover.grace.cancel();
        // 80 ms dwell: under a perceptible wait but longer than a moving
        // pointer spends crossing the clock.
        let weak = std::rc::Rc::downgrade(this);
        let slot = self_.hover.dwell.clone();
        let source =
            glib::timeout_add_local_once(std::time::Duration::from_millis(80), move || {
                // The one-shot destroys itself after dispatching; stop
                // tracking it so a later cancel cannot touch a dead source.
                slot.clear();
                if let Some(island) = weak.upgrade() {
                    let mut island = island.borrow_mut();
                    if island.arbiter.layer(island.world()) == Layer::Modules
                        && !island.hover.summary_held
                    {
                        island.arbiter.summary = true;
                        island.refresh(false);
                    }
                }
            });
        self_.hover.dwell.set(source);
    }

    fn pointer_left(this: &std::rc::Rc<std::cell::RefCell<Self>>) {
        let mut self_ = this.borrow_mut();
        self_.hover.pointer_inside = false;
        self_.hover.summary_held = false;
        self_.hover.dwell.cancel();
        // Grace period stops a pointer on the edge from flapping it open and
        // shut.
        let weak = std::rc::Rc::downgrade(this);
        let grace_slot = self_.hover.grace.clone();
        let grace_source =
            glib::timeout_add_local_once(std::time::Duration::from_millis(260), move || {
                grace_slot.clear();
                if let Some(island) = weak.upgrade() {
                    let mut island = island.borrow_mut();
                    if !island.hover.pointer_inside && island.arbiter.summary {
                        island.arbiter.summary = false;
                        island.refresh(false);
                    }
                }
            });
        self_.hover.grace.set(grace_source);
        // A panel the pointer has entered closes when it leaves it (380 ms).
        // One opened by key with the pointer elsewhere stays until the
        // pointer has been over it.
        if self_.arbiter.expanded() && self_.hover.pointer_entered {
            let weak = std::rc::Rc::downgrade(this);
            let leave_slot = self_.hover.leave_close.clone();
            let leave_source =
                glib::timeout_add_local_once(std::time::Duration::from_millis(380), move || {
                    leave_slot.clear();
                    if let Some(island) = weak.upgrade() {
                        let mut island = island.borrow_mut();
                        if !island.hover.pointer_inside && island.arbiter.expanded() {
                            island.close();
                        }
                    }
                });
            self_.hover.leave_close.set(leave_source);
        }
    }

    /// Recompute the layer, the target size and the visible content.
    pub fn refresh(&mut self, _animate_crossfade: bool) {
        let layer = self.arbiter.layer(self.world());
        self.last_layer = layer;
        let (width, height) = layer_size(
            layer,
            self.current_toast.as_ref(),
            self.arbiter.open_panel.as_deref(),
        );
        self.animate_to(width as f64, height as f64);

        match layer {
            Layer::Panel => match self.arbiter.open_panel.as_deref() {
                Some("launcher") => {
                    self.launcher_panel.on_open();
                    self.stack.set_visible_child_name("panel:launcher");
                }
                Some("notifications") => {
                    self.fill_notifications_list();
                    self.stack.set_visible_child_name("panel:notifications");
                }
                _ => self.stack.set_visible_child_name("modules"),
            },
            Layer::Notification => {
                if let Some(toast) = self.current_toast.clone() {
                    self.render_toast(&toast);
                }
                self.stack.set_visible_child_name("notification");
            }
            Layer::Osd => {
                if let Some(event) = self.arbiter.transient() {
                    self.osd_icon.set_icon_name(Some(&event.icon));
                    self.osd_label.set_text(&event.label);
                    if event.progress >= 0.0 {
                        self.osd_bar.set_visible(true);
                        self.osd_bar.set_fraction(event.progress as f64);
                    } else {
                        self.osd_bar.set_visible(false);
                    }
                }
                self.stack.set_visible_child_name("osd");
            }
            Layer::Summary => {
                self.stack.set_visible_child_name("summary");
            }
            Layer::Modules => {
                self.stack.set_visible_child_name("modules");
            }
        }
        self.update_clock();
    }

    fn render_toast(&mut self, toast: &Toast) {
        self.notification_title
            .set_text(&format!("{} — {}", toast.app_name, toast.summary));
        self.notification_body.set_text(&toast.body);
        while let Some(child) = self.notification_buttons.first_child() {
            self.notification_buttons.remove(&child);
        }
        for (key, label) in toast.buttons() {
            let button = Button::new();
            button.set_label(label);
            button.set_css_classes(&["velatura-notification-button"]);
            let id = toast.id;
            let key = key.clone();
            let services = self.services.clone();
            button.connect_clicked(move |_| {
                services.notifications.invoke_action(id, &key);
            });
            self.notification_buttons.append(&button);
        }
    }

    pub fn update_clock(&mut self) {
        if let Ok(now) = glib::DateTime::now_local() {
            let time = now.format("%H:%M").unwrap_or_default();
            let date = now.format("%a %d %b").unwrap_or_default();
            self.rest_clock.set_text(&time);
            self.rest_date.set_text(&date);
            self.summary_time.set_text(&time);
            self.summary_date.set_text(&date);
        }
    }

    /// Animate the capsule to the target size over the morph duration with
    /// the original's OutCubic curve; one animated value per axis. The bar
    /// surface keeps its size; only this box's allocation changes.
    fn animate_to(&mut self, target_width: f64, target_height: f64) {
        if (self.current_width - target_width).abs() < 0.5
            && (self.current_height - target_height).abs() < 0.5
        {
            self.apply_size(target_width, target_height);
            return;
        }
        let start_width = self.current_width;
        let start_height = self.current_height;
        let started = glib::monotonic_time() / 1000;
        let duration = theme::DURATION_MORPH_MS;
        self.animation.cancel();
        let slot = self.animation.clone();
        let morph_widget = self.morph.clone();
        let progress_source =
            glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                let elapsed = (glib::monotonic_time() / 1000 - started) as f64;
                let t = (elapsed / duration as f64).min(1.0);
                let eased = out_cubic(t);
                let width = start_width + (target_width - start_width) * eased;
                let height = start_height + (target_height - start_height) * eased;
                morph_widget.set_size_request(width.round() as i32, height.round() as i32);
                morph_widget.queue_resize();
                if t >= 1.0 {
                    // The source destroys itself after Break; stop tracking it
                    // so a later cancel cannot touch a dead source.
                    slot.clear();
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        self.animation.set(progress_source);
        self.current_width = target_width;
        self.current_height = target_height;
    }

    fn apply_size(&mut self, width: f64, height: f64) {
        self.morph
            .set_size_request(width.round() as i32, height.round() as i32);
        self.current_width = width;
        self.current_height = height;
    }

    /// The island's current target size, so the bar surface can size itself.
    pub fn current_size(&self) -> (f64, f64) {
        (self.current_width, self.current_height)
    }

    /// Whether the pointer-leave rule may close the open panel.
    pub fn leave_closes_panel(&self) -> bool {
        self.arbiter.expanded()
    }
}

impl Drop for Island {
    fn drop(&mut self) {
        // Only sources that have not fired yet are still alive; fired
        // one-shots destroyed themselves and their slots were cleared.
        self.animation.cancel();
        self.hover.dwell.cancel();
        self.hover.grace.cancel();
        self.hover.leave_close.cancel();
    }
}
