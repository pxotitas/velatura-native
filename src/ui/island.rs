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
use super::clock;
use super::launcher_panel::{self, LauncherPanel};
use super::motion::{Curve, Tween};
use super::notification_layer::NotificationLayer;
use super::settings::Settings;
use super::tokens;
use crate::niri_session::NiriSession;
use crate::services::island_state::{IslandState, Layer, World};
use crate::services::notifications::{NotificationService, Toast, Urgency};

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
    service: Rc<NotificationService>,
    panel_host: gtk4::Box,
    panel: Rc<RefCell<Option<LauncherPanel>>>,
    panel_open: Rc<Cell<bool>>,
    keyboard: Cell<bool>,
    on_keyboard: RefCell<Option<Box<dyn Fn(bool)>>>,
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
        let rest = LayerSize {
            width: settings.rest_width(),
            height: settings.bar_height,
            padding: 0,
        };
        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_transition_duration((tokens::DURATION_FAST * settings.motion()) as u32);
        stack.add_named(&clock::build(&settings), Some("modules"));
        let notification = NotificationLayer::new(Rc::clone(&service), session);
        stack.add_named(&notification.widget, Some("notification"));
        let panel_host = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        panel_host.set_valign(gtk4::Align::Start);
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
                service,
                panel_host,
                panel: Rc::default(),
                panel_open: Rc::default(),
                keyboard: Cell::new(false),
                on_keyboard: RefCell::new(None),
                toast: RefCell::new(None),
                changed: RefCell::new(None),
            }
        });

        capsule.set_content(Some(&stack));
        island.sync_layer();
        island.apply_geometry();

        let motion = gtk4::EventControllerMotion::new();
        let weak = Rc::downgrade(&island);
        motion.connect_enter(move |_, _, _| {
            if let Some(island) = weak.upgrade() {
                island.hovered.set(true);
                island.apply_look();
            }
        });
        let weak = Rc::downgrade(&island);
        motion.connect_leave(move |_| {
            if let Some(island) = weak.upgrade() {
                island.hovered.set(false);
                island.apply_look();
            }
        });
        capsule.add_controller(motion);
        island
    }

    /// `min(height / 2, radiusLarge + 4)` at rest.
    fn rest_radius(height: i32) -> f32 {
        (height as f32 / 2.0).min(tokens::RADIUS_LARGE + 4.0)
    }

    /// The attached island's strip above the capsule, up to the screen edge.
    fn notch_pad_of(settings: &Settings) -> i32 {
        if settings.notched() { settings.bar_margin } else { 0 }
    }

    pub fn notch_pad(&self) -> i32 {
        Self::notch_pad_of(&self.settings)
    }

    /// Current (animated) size of the shape, notch strip included.
    pub fn size(&self) -> (i32, i32) {
        (self.width.value().round() as i32, self.height.value().round() as i32)
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
            notification_critical: toast.as_ref().is_some_and(|t| t.urgency == Urgency::Critical),
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
            width: self.settings.rest_width(),
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
            Layer::Panel => LayerSize {
                width: launcher_panel::WIDTH,
                height: launcher_panel::HEIGHT,
                padding: 20,
            },
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
            Layer::Panel => "panel",
            _ => "modules",
        };
        if self.stack.visible_child_name().as_deref() != Some(page) {
            self.stack.set_visible_child_name(page);
        }
        self.apply_look();
        self.update_keyboard();
    }

    /// Opens a panel on the live island; a toast is dismissed as the panel
    /// takes over. Only the launcher has content so far.
    pub fn open_panel(&self, panel: &str) {
        if panel != "launcher" {
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
        if self.panel.borrow().is_none() {
            let launcher = LauncherPanel::new();
            self.panel_host.append(&launcher.widget);
            *self.panel.borrow_mut() = Some(launcher);
        }
        self.sync_layer();
    }

    pub fn close_panel(&self) {
        self.state.borrow_mut().close();
        self.panel_open.set(false);
        self.sync_layer();
        // Cleared on the way out, once the island has shrunk, so the query
        // does not resize it back mid-morph.
        let (host, panel, open) = (self.panel_host.clone(), Rc::clone(&self.panel), Rc::clone(&self.panel_open));
        let delay = (tokens::DURATION_MORPH * self.settings.motion()) as u64 + 60;
        glib::timeout_add_local_once(std::time::Duration::from_millis(delay), move || {
            if open.get() {
                return;
            }
            if let Some(launcher) = panel.borrow_mut().take() {
                host.remove(&launcher.widget);
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
