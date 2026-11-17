//! `DynamicIsland.qml`: the morphing centre capsule. `IslandState` picks the
//! layer, this sizes the capsule for it, swaps the content with a short
//! cross-fade and animates the shape on the island's own clock.
//!
//! Layers with content so far: modules (the clock) and notification. The
//! others (summary, OSD, panels) are sized when their content is ported, so
//! the island never morphs into an empty shape.

use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use super::capsule::Capsule;
use super::clock;
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
        let notification = NotificationLayer::new(service, session);
        stack.add_named(&notification.widget, Some("notification"));

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
        let page = if layer == Layer::Notification { "notification" } else { "modules" };
        if self.stack.visible_child_name().as_deref() != Some(page) {
            self.stack.set_visible_child_name(page);
        }
        self.apply_look();
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
