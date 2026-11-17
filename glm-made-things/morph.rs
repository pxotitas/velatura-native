//! MorphBox: a container that gives its child one exact allocation and
//! clips it with rounded corners.
//!
//! This is the piece that lets the island morph without resizing a layer
//! surface per frame: the bar surface keeps a stable size, and the island
//! widget animates this box's allocation inside it, like the original's
//! Rectangle with `clip: true`.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

/// Callback type for allocation observation; the bar re-centres the island.
type AllocateCallback = Box<dyn Fn(i32, i32)>;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MorphBox {
        pub child: gtk4::Box,
        /// Called with the box's width and height after each allocation; the
        /// bar uses it to keep the island centred without polling.
        pub on_allocate: std::cell::RefCell<Option<AllocateCallback>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MorphBox {
        const NAME: &'static str = "VelaturaMorphBox";
        type Type = super::MorphBox;
        type ParentType = gtk4::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("morphbox");
        }
    }

    impl ObjectImpl for MorphBox {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            gtk4::prelude::BoxExt::append(&*obj, &self.child);
            self.child.set_overflow(gtk4::Overflow::Hidden);
            // The child fills whatever allocation the box gives it.
            self.child.set_hexpand(true);
            self.child.set_vexpand(true);
            self.child.set_halign(gtk4::Align::Fill);
            self.child.set_valign(gtk4::Align::Fill);
        }

        fn dispose(&self) {
            // The parent chain-up happens automatically; only drop the
            // callback so no timer fires against a dead bar.
            *self.on_allocate.borrow_mut() = None;
        }
    }

    impl WidgetImpl for MorphBox {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            // The child fills the whole box; CSS border-radius rounds the
            // clip of whatever renders larger.
            self.child.allocate(width, height, baseline, None);
            if let Some(on_allocate) = self.on_allocate.borrow().as_ref() {
                on_allocate(width, height);
            }
        }

        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            // The box is exactly as large as it is told to be; the outside
            // animation owns the size. Zero natural size keeps it from
            // fighting the animation.
            (0, 0, -1, -1)
        }
    }

    impl BoxImpl for MorphBox {}
}

glib::wrapper! {
    pub struct MorphBox(ObjectSubclass<imp::MorphBox>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl MorphBox {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// The child every layer renders into.
    pub fn child(&self) -> &gtk4::Box {
        &self.imp().child
    }

    /// Observe this box's allocations; the bar re-centres the island here.
    pub fn connect_allocate<F: Fn(i32, i32) + 'static>(&self, callback: F) {
        *self.imp().on_allocate.borrow_mut() = Some(Box::new(callback) as AllocateCallback);
    }
}

impl Default for MorphBox {
    fn default() -> Self {
        Self::new()
    }
}
