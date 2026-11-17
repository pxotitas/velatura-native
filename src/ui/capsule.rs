//! The island's shape: an exact-size rounded box that paints its own ground
//! and clips one child. The layer surface never resizes; this widget does,
//! frame by frame, inside it (`DynamicIsland.qml`'s clipped Rectangle).

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib, graphene, gsk};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Capsule {
        pub size: Cell<(i32, i32)>,
        /// Top-left, top-right, bottom-right, bottom-left.
        pub radii: Cell<[f32; 4]>,
        pub color: Cell<Option<gdk::RGBA>>,
        /// Outline drawn inside the edge: width and colour.
        pub border: Cell<Option<(f32, gdk::RGBA)>>,
        /// Top, left, right, bottom padding of the child inside the shape.
        pub insets: Cell<[i32; 4]>,
        pub child: RefCell<Option<gtk4::Widget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Capsule {
        const NAME: &'static str = "VelaturaCapsule";
        type Type = super::Capsule;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Capsule {
        fn dispose(&self) {
            if let Some(child) = self.child.take() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Capsule {
        fn measure(&self, orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let (w, h) = self.size.get();
            let extent = if orientation == gtk4::Orientation::Horizontal {
                w
            } else {
                h
            };
            (extent, extent, -1, -1)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            if let Some(child) = self.child.borrow().as_ref() {
                let [top, left, right, bottom] = self.insets.get();
                let rect = gdk::Rectangle::new(
                    left,
                    top,
                    (width - left - right).max(0),
                    (height - top - bottom).max(0),
                );
                child.size_allocate(&rect, baseline);
            }
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let (w, h) = (self.obj().width() as f32, self.obj().height() as f32);
            if w <= 0.0 || h <= 0.0 {
                return;
            }
            let cap = w.min(h) / 2.0;
            let [tl, tr, br, bl] = self.radii.get().map(|r| r.clamp(0.0, cap));
            let bounds = graphene::Rect::new(0.0, 0.0, w, h);
            let rounded = gsk::RoundedRect::new(
                bounds,
                graphene::Size::new(tl, tl),
                graphene::Size::new(tr, tr),
                graphene::Size::new(br, br),
                graphene::Size::new(bl, bl),
            );
            snapshot.push_rounded_clip(&rounded);
            if let Some(color) = self.color.get() {
                snapshot.append_color(&color, &bounds);
            }
            if let Some(child) = self.child.borrow().as_ref() {
                self.obj().snapshot_child(child, snapshot);
            }
            snapshot.pop();
            if let Some((width, color)) = self.border.get() {
                snapshot.append_border(&rounded, &[width; 4], &[color; 4]);
            }
        }
    }
}

glib::wrapper! {
    pub struct Capsule(ObjectSubclass<imp::Capsule>) @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Capsule {
    pub fn new() -> Self {
        let capsule: Self = glib::Object::new();
        capsule.set_overflow(gtk4::Overflow::Hidden);
        capsule
    }

    pub fn set_size(&self, width: i32, height: i32) {
        let imp = self.imp();
        if imp.size.get() != (width, height) {
            imp.size.set((width, height));
            self.queue_resize();
        }
    }

    pub fn set_radii(&self, radii: [f32; 4]) {
        self.imp().radii.set(radii);
        self.queue_draw();
    }

    pub fn set_color(&self, color: gdk::RGBA) {
        self.imp().color.set(Some(color));
        self.queue_draw();
    }

    pub fn set_border(&self, width: f32, color: gdk::RGBA) {
        self.imp().border.set(Some((width, color)));
        self.queue_draw();
    }

    pub fn set_insets(&self, top: i32, left: i32, right: i32, bottom: i32) {
        self.imp().insets.set([top, left, right, bottom]);
        self.queue_allocate();
    }

    pub fn set_content(&self, child: Option<&impl IsA<gtk4::Widget>>) {
        let imp = self.imp();
        if let Some(old) = imp.child.take() {
            old.unparent();
        }
        if let Some(child) = child {
            child.set_parent(self);
            *imp.child.borrow_mut() = Some(child.clone().upcast());
        }
        self.queue_resize();
    }
}

impl Default for Capsule {
    fn default() -> Self {
        Self::new()
    }
}
