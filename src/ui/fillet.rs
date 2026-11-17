//! `NotchFillet.qml`: the concave corner where an attached island meets the
//! screen edge — the square minus a disc centred on its far corner.

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib, gsk};
use std::cell::{Cell, RefCell};

use super::tokens;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Fillet {
        pub color: Cell<Option<gdk::RGBA>>,
        pub mirrored: Cell<bool>,
        pub path: RefCell<Option<gsk::Path>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Fillet {
        const NAME: &'static str = "VelaturaFillet";
        type Type = super::Fillet;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Fillet {}

    impl WidgetImpl for Fillet {
        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let side = tokens::RADIUS_NOTCH * 2;
            (side, side, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let Some(color) = self.color.get() else {
                return;
            };
            let side = (tokens::RADIUS_NOTCH * 2) as f32;
            let mirror = self.mirrored.get();
            let x = |value: f32| if mirror { side - value } else { value };
            // Cubic approximation of the quarter arc around the far corner.
            let k = 0.552_285 * side;
            let builder = gsk::PathBuilder::new();
            builder.move_to(x(0.0), 0.0);
            builder.line_to(x(side), 0.0);
            builder.cubic_to(x(side - k), 0.0, x(0.0), side - k, x(0.0), side);
            builder.close();
            let path = builder.to_path();
            snapshot.append_fill(&path, gsk::FillRule::Winding, &color);
        }
    }
}

glib::wrapper! {
    pub struct Fillet(ObjectSubclass<imp::Fillet>) @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Fillet {
    pub fn new(mirrored: bool) -> Self {
        let fillet: Self = glib::Object::new();
        fillet.imp().mirrored.set(mirrored);
        fillet.set_can_target(false);
        fillet
    }

    pub fn set_color(&self, color: gdk::RGBA) {
        self.imp().color.set(Some(color));
        self.queue_draw();
    }
}
