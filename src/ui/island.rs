//! `DynamicIsland.qml`: the morphing centre capsule. `IslandState` picks the
//! layer, this sizes the capsule for it, swaps the content with a short
//! cross-fade and animates the shape on the island's own clock.
//!
//! Layers with content so far: modules (the clock) and notification. The
//! others (summary, OSD, panels) are sized when their content is ported, so
//! the island never morphs into an empty shape.

use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use super::capsule::Capsule;
use super::controls_panel::ControlsPanel;
use super::launcher_panel::{self, LauncherPanel};
use super::launcher_store::Store;
use super::motion::{Curve, Tween};
use super::notification_layer::NotificationLayer;
use super::settings::Settings;
use super::system_panel::{self, SystemPanel};
use super::tokens;
use crate::niri_session::NiriSession;
use crate::services::island_state::{
    IslandState, Layer, TRANSIENT_DURATION_MS, TransientEvent, World,
};
use crate::services::notifications::{NotificationService, Toast, Urgency};

type KeyboardCallback = Box<dyn Fn(bool)>;

pub struct Island {
    pub capsule: Capsule,
    settings: Rc<Settings>,
    state: RefCell<IslandState>,
    world: Cell<World>,
    width: Tween,
    height: Tween,
    radius: Tween,
    hovered: Cell<bool>,
    stack: gtk4::Stack,
    notification: NotificationLayer,
    summary_view: gtk4::Box,
    osd_view: gtk4::Box,
    media_active: Cell<bool>,
    summary_visible: Cell<bool>,
    service: Rc<NotificationService>,
    panel_host: gtk4::Box,
    store: Rc<RefCell<Store>>,
    this: Weak<Island>,
    panel: Rc<RefCell<Option<LauncherPanel>>>,
    system_panel: Rc<RefCell<Option<SystemPanel>>>,
    controls_panel: Rc<RefCell<Option<ControlsPanel>>>,
    panel_open: Rc<Cell<bool>>,
    keyboard: Cell<bool>,
    on_keyboard: RefCell<Option<KeyboardCallback>>,
    toast: RefCell<Option<Toast>>,
    changed: RefCell<Option<Box<dyn Fn()>>>,
}

struct LayerSize {
    width: i32,
    height: i32,
    padding: i32,
}

impl Island {
    pub fn new(
        settings: Rc<Settings>,
        active: bool,
        service: Rc<NotificationService>,
        session: Arc<NiriSession>,
    ) -> Rc<Self> {
        let capsule = Capsule::new();
        let notch_pad = Self::notch_pad_of(&settings);
        let media_active =
            crate::services::media::snapshot().is_some_and(|snapshot| snapshot.playing());
        let rest = LayerSize {
            width: super::island_rest::width(&settings, media_active),
            height: settings.bar_height,
            padding: 0,
        };
        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_transition_duration((tokens::DURATION_FAST * settings.motion()) as u32);
        stack.add_named(
            &super::island_rest::build(&settings, media_active),
            Some("modules"),
        );
        let notification = NotificationLayer::new(Rc::clone(&service), session);
        // The glance is a real Quickshell IslandSummary face, not a generic
        // placeholder. It follows MPRIS and owns its media transport controls.
        let summary_view = super::island_summary::build();
        let osd_view = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        osd_view.set_halign(gtk4::Align::Center);
        stack.add_named(&notification.widget, Some("notification"));
        stack.add_named(&summary_view, Some("summary"));
        stack.add_named(&osd_view, Some("osd"));
        let panel_host = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        panel_host.set_vexpand(true);
        stack.add_named(&panel_host, Some("panel"));

        let island = Rc::new_cyclic(|weak: &Weak<Island>| {
            let tween = |initial: f64| {
                let weak = weak.clone();
                Tween::new(&capsule, initial, move |_| {
                    if let Some(island) = weak.upgrade() {
                        island.apply_geometry();
                    }
                })
            };
            Island {
                width: tween(f64::from(rest.width)),
                height: tween(f64::from(rest.height + notch_pad)),
                radius: tween(f64::from(Self::rest_radius(rest.height + notch_pad))),
                capsule: capsule.clone(),
                settings: Rc::clone(&settings),
                state: RefCell::new(IslandState::new(active)),
                world: Cell::new(World::default()),
                hovered: Cell::new(false),
                stack: stack.clone(),
                notification,
                summary_view,
                osd_view,
                media_active: Cell::new(media_active),
                summary_visible: Cell::new(false),
                service,
                panel_host,
                store: Rc::new(RefCell::new(Store::new(&settings))),
                this: weak.clone(),
                panel: Rc::default(),
                system_panel: Rc::default(),
                controls_panel: Rc::default(),
                panel_open: Rc::default(),
                keyboard: Cell::new(false),
                on_keyboard: RefCell::new(None),
                toast: RefCell::new(None),
                changed: RefCell::new(None),
            }
        });

        let summary_click = gtk4::GestureClick::new();
        let weak = Rc::downgrade(&island);
        summary_click.connect_released(move |_, _, _, _| {
            if let Some(island) = weak.upgrade() {
                island.open_panel("controls");
            }
        });
        island.summary_view.add_controller(summary_click);
        island.summary_view.set_cursor_from_name(Some("pointer"));
        capsule.set_content(Some(&stack));
        island.sync_layer();
        island.apply_geometry();

        let motion = gtk4::EventControllerMotion::new();
        let weak = Rc::downgrade(&island);
        motion.connect_enter(move |_, _, _| {
            if let Some(island) = weak.upgrade() {
                island.hovered.set(true);
                island.apply_look();
                island.schedule_summary();
            }
        });
        let weak = Rc::downgrade(&island);
        motion.connect_leave(move |_| {
            if let Some(island) = weak.upgrade() {
                island.hovered.set(false);
                island.apply_look();
                let weak = island.this.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(260), move || {
                    if let Some(island) = weak.upgrade() {
                        if !island.hovered.get() && island.state.borrow().open_panel.is_none() {
                            island.state.borrow_mut().summary = false;
                            island.summary_visible.set(false);
                            island.sync_layer();
                        }
                    }
                });
            }
        });
        capsule.add_controller(motion);
        // Track the same MPRIS "playing" transition as ModuleService, so
        // IslandRest grows from the clock core to the split activity layout.
        let weak = island.this.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(750), move || {
            let Some(island) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let active =
                crate::services::media::snapshot().is_some_and(|snapshot| snapshot.playing());
            if island.media_active.replace(active) != active && island.layer() == Layer::Modules {
                island.sync_layer();
            }
            glib::ControlFlow::Continue
        });
        island
    }

    /// `min(height / 2, radiusLarge + 4)` at rest.
    fn rest_radius(height: i32) -> f32 {
        (height as f32 / 2.0).min(tokens::RADIUS_LARGE + 4.0)
    }

    /// The attached island's strip above the capsule, up to the screen edge.
    fn notch_pad_of(settings: &Settings) -> i32 {
        if settings.notched() {
            settings.bar_margin
        } else {
            0
        }
    }

    pub fn notch_pad(&self) -> i32 {
        Self::notch_pad_of(&self.settings)
    }

    /// Current (animated) size of the shape, notch strip included.
    pub fn size(&self) -> (i32, i32) {
        (
            self.width.value().round() as i32,
            self.height.value().round() as i32,
        )
    }

    pub fn layer(&self) -> Layer {
        self.state.borrow().layer(self.world.get())
    }

    pub fn connect_changed(&self, callback: impl Fn() + 'static) {
        *self.changed.borrow_mut() = Some(Box::new(callback));
    }

    /// Only the screen that is live arbitrates; the others stay at rest.
    pub fn set_active(&self, active: bool) {
        self.state.borrow_mut().set_active(active);
        self.sync_layer();
    }

    /// `NotificationService.current` reaching this island.
    pub fn set_notification(&self, toast: Option<Toast>) {
        self.world.set(World {
            notification_active: toast.is_some(),
            notification_critical: toast
                .as_ref()
                .is_some_and(|t| t.urgency == Urgency::Critical),
        });
        if toast.is_some() {
            self.notification.set_toast(toast.as_ref());
        }
        *self.toast.borrow_mut() = toast;
        self.sync_layer();
    }

    pub fn notification_id(&self) -> Option<u32> {
        self.toast.borrow().as_ref().map(|toast| toast.id)
    }

    pub fn surface_color(&self) -> super::tokens::Rgba {
        let style = self.settings.surface();
        let at_rest = self.layer() == Layer::Modules;
        let lit = self.hovered.get() && at_rest && !self.settings.unified();
        if !tokens::see_through(style) {
            if lit {
                tokens::ISLAND_SURFACE_HOVER
            } else if self.settings.notched() || !at_rest {
                tokens::ground_of(style)
            } else {
                tokens::ISLAND_SURFACE
            }
        } else if lit {
            tokens::ground_lit(style)
        } else {
            tokens::ground_of(style)
        }
    }

    fn layer_size(&self, layer: Layer) -> LayerSize {
        let rest = LayerSize {
            width: super::island_rest::width(&self.settings, self.media_active.get()),
            height: self.settings.bar_height,
            padding: 0,
        };
        match layer {
            Layer::Notification => match self.toast.borrow().as_ref() {
                Some(toast) => LayerSize {
                    width: Toast::toast_width(),
                    height: toast.toast_height(),
                    padding: 12,
                },
                None => rest,
            },
            // Quickshell ModuleService.glanceFaceHeight = 150; the
            // summary's own margins are part of that measured face.
            Layer::Summary => LayerSize {
                width: 620,
                height: 150,
                padding: 0,
            },
            Layer::Osd => LayerSize {
                width: 260,
                height: self.settings.bar_height,
                padding: 10,
            },
            Layer::Panel => {
                let page = self.state.borrow().open_panel.clone().unwrap_or_default();
                if page == "controls" {
                    LayerSize {
                        width: super::controls_panel::WIDTH,
                        height: super::controls_panel::HEIGHT,
                        padding: 20,
                    }
                } else if matches!(page.as_str(), "wifi" | "bluetooth" | "sound" | "battery") {
                    LayerSize {
                        width: system_panel::WIDTH,
                        height: system_panel::HEIGHT,
                        padding: 20,
                    }
                } else {
                    LayerSize {
                        width: launcher_panel::WIDTH,
                        height: launcher_panel::HEIGHT,
                        padding: 20,
                    }
                }
            }
            _ => rest,
        }
    }

    /// Retargets the morph to the current layer, on the island's clock.
    pub fn sync_layer(&self) {
        let layer = self.layer();
        let size = self.layer_size(layer);
        let notch_pad = self.notch_pad();
        let duration = tokens::DURATION_MORPH * self.settings.motion();
        let curve = Curve::from_id(&self.settings.motion_curve);
        let total = size.height + notch_pad;
        let outer = size.width;
        self.width.animate(f64::from(outer), duration, curve);
        self.height.animate(f64::from(total), duration, curve);
        let radius = if layer == Layer::Modules {
            Self::rest_radius(total)
        } else {
            tokens::RADIUS_LARGE
        };
        self.radius.animate(
            f64::from(radius),
            tokens::DURATION_MEDIUM * self.settings.motion(),
            curve,
        );

        let notch_half = notch_pad / 2;
        self.capsule.set_insets(
            size.padding + notch_half,
            size.padding,
            size.padding,
            size.padding + notch_half,
        );
        let page = match layer {
            Layer::Notification => "notification",
            Layer::Summary => "summary",
            Layer::Osd => "osd",
            Layer::Panel => "panel",
            Layer::Modules => "modules",
        };
        if self.stack.visible_child_name().as_deref() != Some(page) {
            self.stack.set_visible_child_name(page);
        }
        self.apply_look();
        self.update_keyboard();
    }

    /// Starts the original hover-intent delay before showing the summary.
    fn schedule_summary(&self) {
        if !self.state.borrow().active || self.state.borrow().open_panel.is_some() {
            return;
        }
        let weak = self.this.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(80), move || {
            if let Some(island) = weak.upgrade() {
                if island.hovered.get() && !island.panel_is_open() {
                    island.state.borrow_mut().summary = true;
                    island.summary_visible.set(true);
                    island.sync_layer();
                }
            }
        });
    }

    /// A transient OSD takes over the summary/rest layer for 1.8 seconds.
    pub fn request_osd(&self, event: TransientEvent) {
        self.state
            .borrow_mut()
            .request_osd(crate::services::now_ms(), event.clone());
        if let Some(child) = self.osd_view.first_child() {
            self.osd_view.remove(&child);
        }
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        let icon = gtk4::Label::new(Some(&event.icon));
        icon.add_css_class("chip-glyph");
        let label = gtk4::Label::new(Some(&event.label));
        label.add_css_class("toast-cell");
        label.set_width_chars(4);
        label.set_xalign(1.0);
        row.append(&icon);
        row.append(&label);
        if event.progress >= 0.0 {
            let progress = gtk4::ProgressBar::new();
            progress.set_fraction(f64::from(event.progress.clamp(0.0, 1.0)));
            progress.set_width_request(130);
            progress.set_height_request(4);
            row.append(&progress);
        }
        self.osd_view.append(&row);
        self.sync_layer();
        let weak = self.this.clone();
        glib::timeout_add_local_once(
            std::time::Duration::from_millis(TRANSIENT_DURATION_MS),
            move || {
                if let Some(island) = weak.upgrade() {
                    if island.state.borrow_mut().tick(crate::services::now_ms()) {
                        island.sync_layer();
                    }
                }
            },
        );
    }

    /// Opens the requested native Island surface on the live output.
    pub fn open_panel(&self, panel: &str) {
        if !matches!(
            panel,
            "launcher" | "controls" | "wifi" | "bluetooth" | "sound" | "battery"
        ) {
            return;
        }
        let dismiss = self.state.borrow_mut().open(panel);
        if self.state.borrow().open_panel.is_none() {
            return;
        }
        if dismiss {
            self.service.dismiss();
        }
        self.panel_open.set(true);

        if panel == "launcher" {
            if let Some(detail) = self.system_panel.borrow_mut().take() {
                self.panel_host.remove(&detail.widget);
            }
            if let Some(controls) = self.controls_panel.borrow_mut().take() {
                self.panel_host.remove(&controls.widget);
            }
            if self.panel.borrow().is_none() {
                self.store.borrow_mut().refresh();
                let this = self.this.clone();
                let launcher = LauncherPanel::new(
                    Rc::clone(&self.store),
                    Rc::new(move || {
                        if let Some(island) = this.upgrade() {
                            island.close_panel();
                        }
                    }),
                );
                self.panel_host.append(&launcher.widget);
                *self.panel.borrow_mut() = Some(launcher);
            }
        } else if panel == "controls" {
            if let Some(launcher) = self.panel.borrow_mut().take() {
                self.panel_host.remove(&launcher.widget);
            }
            if let Some(detail) = self.system_panel.borrow_mut().take() {
                self.panel_host.remove(&detail.widget);
            }
            if self.controls_panel.borrow().is_none() {
                let this = self.this.clone();
                let close = move || {
                    if let Some(island) = this.upgrade() {
                        island.close_panel();
                    }
                };
                let this = self.this.clone();
                let open = move |page: &str| {
                    if let Some(island) = this.upgrade() {
                        island.open_panel(page);
                    }
                };
                let panel = ControlsPanel::new(Rc::clone(&self.service), close, open);
                self.panel_host.append(&panel.widget);
                *self.controls_panel.borrow_mut() = Some(panel);
            }
        } else {
            if let Some(launcher) = self.panel.borrow_mut().take() {
                self.panel_host.remove(&launcher.widget);
            }
            if let Some(controls) = self.controls_panel.borrow_mut().take() {
                self.panel_host.remove(&controls.widget);
            }
            if self
                .system_panel
                .borrow()
                .as_ref()
                .is_none_or(|current| current.page() != panel)
            {
                if let Some(old) = self.system_panel.borrow_mut().take() {
                    self.panel_host.remove(&old.widget);
                }
                let this = self.this.clone();
                let detail = SystemPanel::new(panel, move || {
                    if let Some(island) = this.upgrade() {
                        island.close_panel();
                    }
                });
                self.panel_host.append(&detail.widget);
                *self.system_panel.borrow_mut() = Some(detail);
            }
        }
        self.sync_layer();
    }

    pub fn close_panel(&self) {
        self.state.borrow_mut().close();
        self.panel_open.set(false);
        self.sync_layer();
        // Cleared on the way out, once the island has shrunk, so the query
        // does not resize it back mid-morph.
        let (host, panel, open) = (
            self.panel_host.clone(),
            Rc::clone(&self.panel),
            Rc::clone(&self.panel_open),
        );
        let system_panel = Rc::clone(&self.system_panel);
        let controls_panel = Rc::clone(&self.controls_panel);
        let delay = (tokens::DURATION_MORPH * self.settings.motion()) as u64 + 60;
        glib::timeout_add_local_once(std::time::Duration::from_millis(delay), move || {
            if open.get() {
                return;
            }
            if let Some(launcher) = panel.borrow_mut().take() {
                host.remove(&launcher.widget);
            }
            if let Some(detail) = system_panel.borrow_mut().take() {
                host.remove(&detail.widget);
            }
            if let Some(controls) = controls_panel.borrow_mut().take() {
                host.remove(&controls.widget);
            }
        });
    }

    pub fn toggle_panel(&self, panel: &str) {
        if self.state.borrow().open_panel.as_deref() == Some(panel) {
            self.close_panel();
        } else {
            self.open_panel(panel);
        }
    }

    pub fn panel_is_open(&self) -> bool {
        self.state.borrow().open_panel.is_some()
    }

    /// Refreshes a live launcher list when the clipboard history changes.
    pub fn refresh_clipboard(&self) {
        if let Some(panel) = self.panel.borrow().as_ref() {
            panel.refresh();
        }
    }

    /// The bar holds the keyboard while a panel is open on the live island.
    pub fn connect_keyboard(&self, callback: impl Fn(bool) + 'static) {
        *self.on_keyboard.borrow_mut() = Some(Box::new(callback));
    }

    fn update_keyboard(&self) {
        let want = self.state.borrow().expanded();
        if self.keyboard.replace(want) == want {
            return;
        }
        if let Some(callback) = self.on_keyboard.borrow().as_ref() {
            callback(want);
        }
        if want {
            // The field only takes focus once the surface has the keyboard.
            let panel = Rc::clone(&self.panel);
            glib::timeout_add_local_once(std::time::Duration::from_millis(40), move || {
                if let Some(launcher) = panel.borrow().as_ref() {
                    launcher.focus();
                }
            });
        }
    }

    fn apply_geometry(&self) {
        let (width, height) = self.size();
        self.capsule.set_size(width, height);
        self.apply_look();
    }

    fn apply_look(&self) {
        let radius = self.radius.value() as f32;
        let top = if self.settings.notched() { 0.0 } else { radius };
        self.capsule.set_radii([top, top, radius, radius]);
        self.capsule.set_color(self.surface_color().gdk());
        if let Some(changed) = self.changed.borrow().as_ref() {
            changed();
        }
    }
}
