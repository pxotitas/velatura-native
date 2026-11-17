//! `DynamicIsland.qml`: the morphing centre capsule. `IslandState` picks the
//! layer, this sizes the capsule for it, swaps the content with a short
//! cross-fade and animates the shape on the island's own clock.
//!
//! Layers rendered so far: modules (the clock), OSD and notification.
//! Summary and panel routes still require their native providers and widgets.

use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use super::capsule::Capsule;
use super::clock;
use super::motion::{Curve, Tween};
use super::notification_layer::NotificationLayer;
use super::notification_panel::NotificationHistoryPanel;
use super::osd_layer::OsdLayer;
use super::settings::Settings;
use super::tokens;
use crate::niri_session::NiriSession;
use crate::services::island_state::{
    IslandState, Layer, TRANSIENT_DURATION_MS, TransientEvent, World,
};
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
    notification_history: NotificationHistoryPanel,
    osd: OsdLayer,
    osd_expiry: TimerSlot,
    toast: RefCell<Option<Toast>>,
    changed: RefCell<Option<Box<dyn Fn()>>>,
}

struct LayerSize {
    width: i32,
    height: i32,
    padding: i32,
}

/// A replaceable one-shot source. Its callback clears the slot before GLib
/// destroys the source, so cancellation never removes an already-fired ID.
#[derive(Clone, Default)]
struct TimerSlot(Rc<RefCell<Option<gtk4::glib::SourceId>>>);

impl TimerSlot {
    fn cancel(&self) {
        if let Some(source) = self.0.borrow_mut().take() {
            source.remove();
        }
    }

    fn clear(&self) {
        self.0.borrow_mut().take();
    }
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
        let osd = OsdLayer::new(&settings);
        stack.add_named(&osd.widget, Some("osd"));

        let island = Rc::new_cyclic(|weak: &Weak<Island>| {
            let close_weak = weak.clone();
            let notification_history =
                NotificationHistoryPanel::new(Rc::clone(&service), move || {
                    if let Some(island) = close_weak.upgrade() {
                        island.close_panel();
                    }
                });
            stack.add_named(&notification_history.widget, Some("notification-history"));
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
                notification_history,
                osd,
                osd_expiry: TimerSlot::default(),
                toast: RefCell::new(None),
                changed: RefCell::new(None),
            }
        });

        capsule.set_content(Some(&stack));
        capsule.set_focusable(true);
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

        let key = gtk4::EventControllerKey::new();
        let weak = Rc::downgrade(&island);
        key.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape
                && let Some(island) = weak.upgrade()
                && island.state.borrow().expanded()
            {
                island.close_panel();
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        capsule.add_controller(key);
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
        if !active {
            self.osd_expiry.cancel();
        }
        self.sync_layer();
    }

    /// Opens a panel on the live island and dismisses the current toast, as
    /// the original does when a panel takes over. Panel content is supplied
    /// by the UI host as those routes are ported.
    pub fn open_panel(&self, panel: &str) {
        let dismiss_notification = self.state.borrow_mut().open(panel, self.world.get());
        if self.state.borrow().expanded() {
            self.osd_expiry.cancel();
            if self.state.borrow().open_panel.as_deref() == Some("notifications") {
                self.notification_history.refresh();
            }
        }
        if dismiss_notification {
            self.notification.dismiss();
        }
        self.sync_layer();
    }

    pub fn close_panel(&self) {
        self.state.borrow_mut().close();
        self.sync_layer();
    }

    pub fn refresh_notification_panel(&self) {
        self.notification_history.refresh();
    }

    pub fn keyboard_panel_open(&self) -> bool {
        self.state.borrow().open_panel.as_deref() == Some("notifications")
    }

    /// Receives a real OSD event from a native service adapter. The event is
    /// dropped when this island is inactive or a panel currently owns it.
    pub fn request_osd(self: &Rc<Self>, event: TransientEvent) {
        let now = (gtk4::glib::monotonic_time().max(0) as u64) / 1000;
        self.state.borrow_mut().request_osd(now, event);
        let Some(current) = self.state.borrow().transient().cloned() else {
            return;
        };
        self.osd.set_event(&current);
        self.sync_layer();

        self.osd_expiry.cancel();
        let slot = self.osd_expiry.clone();
        let weak = Rc::downgrade(self);
        let source = gtk4::glib::timeout_add_local(
            std::time::Duration::from_millis(TRANSIENT_DURATION_MS),
            move || {
                slot.clear();
                if let Some(island) = weak.upgrade() {
                    let now = (gtk4::glib::monotonic_time().max(0) as u64) / 1000;
                    if island.state.borrow_mut().tick(now) {
                        island.sync_layer();
                    }
                }
                gtk4::glib::ControlFlow::Break
            },
        );
        *self.osd_expiry.0.borrow_mut() = Some(source);
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
            Layer::Osd => LayerSize {
                width: 260,
                height: self.settings.bar_height,
                padding: 10,
            },
            Layer::Panel if self.state.borrow().open_panel.as_deref() == Some("notifications") => {
                LayerSize {
                    width: 380,
                    height: NotificationHistoryPanel::height(
                        self.notification_history_count().min(6),
                    ),
                    padding: 4,
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
        let notification_panel = self.state.borrow().open_panel.as_deref() == Some("notifications");
        let page = match layer {
            Layer::Notification => "notification",
            Layer::Osd => "osd",
            Layer::Panel if notification_panel => "notification-history",
            _ => "modules",
        };
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

    fn notification_history_count(&self) -> usize {
        self.notification_history.count()
    }
}

impl Drop for Island {
    fn drop(&mut self) {
        self.osd_expiry.cancel();
    }
}
