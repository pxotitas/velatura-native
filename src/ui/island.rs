//! `DynamicIsland.qml`: the morphing centre capsule. `IslandState` (already
//! ported) picks the layer, this sizes the capsule for it and animates the
//! shape on the island's own clock.
//!
//! Layers with their own content (summary, OSD, notification, panels) are
//! ported one by one; until a layer has content its size is not offered, so
//! the island never morphs into an empty shape.

use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use super::capsule::Capsule;
use super::clock;
use super::motion::{Curve, Tween};
use super::settings::Settings;
use super::tokens;
use crate::services::island_state::{IslandState, Layer, World};

pub struct Island {
    pub capsule: Capsule,
    settings: Rc<Settings>,
    state: RefCell<IslandState>,
    world: Cell<World>,
    width: Tween,
    height: Tween,
    hovered: Cell<bool>,
    changed: RefCell<Option<Box<dyn Fn()>>>,
}

struct LayerSize {
    width: i32,
    height: i32,
    padding: i32,
}

impl Island {
    pub fn new(settings: Rc<Settings>, active: bool) -> Rc<Self> {
        let capsule = Capsule::new();
        let notch_pad = Self::notch_pad_of(&settings);
        let rest = LayerSize {
            width: settings.rest_width(),
            height: settings.bar_height,
            padding: 0,
        };
        let island = Rc::new_cyclic(|weak: &Weak<Island>| {
            let (on_width, on_height) = (weak.clone(), weak.clone());
            Island {
                width: Tween::new(&capsule, f64::from(rest.width), move |_| {
                    if let Some(island) = on_width.upgrade() {
                        island.apply_geometry();
                    }
                }),
                height: Tween::new(&capsule, f64::from(rest.height + notch_pad), move |_| {
                    if let Some(island) = on_height.upgrade() {
                        island.apply_geometry();
                    }
                }),
                capsule: capsule.clone(),
                settings: Rc::clone(&settings),
                state: RefCell::new(IslandState::new(active)),
                world: Cell::new(World::default()),
                hovered: Cell::new(false),
                changed: RefCell::new(None),
            }
        });

        capsule.set_content(Some(&clock::build(&settings)));
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

    pub fn surface_color(&self) -> super::tokens::Rgba {
        let style = self.settings.surface();
        let lit = self.hovered.get() && self.layer() == Layer::Modules && !self.settings.unified();
        if !tokens::see_through(style) {
            if lit {
                tokens::ISLAND_SURFACE_HOVER
            } else if self.settings.notched() {
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
        match layer {
            // The other layers are sized when their content is ported.
            _ => LayerSize {
                width: self.settings.rest_width(),
                height: self.settings.bar_height,
                padding: 0,
            },
        }
    }

    /// Retargets the morph to the current layer, on the island's clock.
    pub fn sync_layer(&self) {
        let size = self.layer_size(self.layer());
        let notch_pad = self.notch_pad();
        let duration = tokens::DURATION_MORPH * self.settings.motion();
        let curve = Curve::from_id(&self.settings.motion_curve);
        self.width.animate(f64::from(size.width), duration, curve);
        self.height.animate(f64::from(size.height + notch_pad), duration, curve);
        let notch_half = notch_pad / 2;
        self.capsule.set_insets(
            size.padding + notch_half,
            size.padding,
            size.padding,
            size.padding + notch_half,
        );
    }

    fn apply_geometry(&self) {
        let (width, height) = self.size();
        self.capsule.set_size(width, height);
        self.apply_look();
    }

    fn apply_look(&self) {
        let (_, height) = self.size();
        let notched = self.settings.notched();
        let radius = (height as f32 / 2.0).min(tokens::RADIUS_LARGE + 4.0);
        let top = if notched { 0.0 } else { radius };
        self.capsule.set_radii([top, top, radius, radius]);
        self.capsule.set_color(self.surface_color().gdk());
        if let Some(changed) = self.changed.borrow().as_ref() {
            changed();
        }
    }
}
