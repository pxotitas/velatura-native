//! Property animation equivalent to a QML `Behavior`: retargeting mid-flight
//! restarts the duration from the current value, on the frame clock.

use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Curve {
    OutCubic,
    OutQuint,
    OutBack,
    Linear,
}

impl Curve {
    pub fn from_id(id: &str) -> Self {
        match id {
            "OutQuint" => Self::OutQuint,
            "OutBack" => Self::OutBack,
            "Linear" => Self::Linear,
            _ => Self::OutCubic,
        }
    }

    pub fn ease(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::OutCubic => 1.0 - (1.0 - t).powi(3),
            Self::OutQuint => 1.0 - (1.0 - t).powi(5),
            Self::OutBack => {
                // Qt's default overshoot.
                let s = 1.70158;
                let u = t - 1.0;
                u * u * ((s + 1.0) * u + s) + 1.0
            }
        }
    }
}

struct Inner {
    widget: glib::WeakRef<gtk4::Widget>,
    value: Cell<f64>,
    from: Cell<f64>,
    to: Cell<f64>,
    started: Cell<Option<i64>>,
    duration_us: Cell<i64>,
    curve: Cell<Curve>,
    tick: RefCell<Option<gtk4::TickCallbackId>>,
    on_change: Box<dyn Fn(f64)>,
}

#[derive(Clone)]
pub struct Tween(Rc<Inner>);

impl Tween {
    pub fn new(
        widget: &impl IsA<gtk4::Widget>,
        initial: f64,
        on_change: impl Fn(f64) + 'static,
    ) -> Self {
        Self(Rc::new(Inner {
            widget: widget.upcast_ref::<gtk4::Widget>().downgrade(),
            value: Cell::new(initial),
            from: Cell::new(initial),
            to: Cell::new(initial),
            started: Cell::new(None),
            duration_us: Cell::new(0),
            curve: Cell::new(Curve::OutCubic),
            tick: RefCell::new(None),
            on_change: Box::new(on_change),
        }))
    }

    pub fn value(&self) -> f64 {
        self.0.value.get()
    }

    pub fn target(&self) -> f64 {
        self.0.to.get()
    }

    pub fn settled(&self) -> bool {
        (self.0.value.get() - self.0.to.get()).abs() < f64::EPSILON
    }

    pub fn jump(&self, to: f64) {
        self.stop();
        self.0.to.set(to);
        self.0.value.set(to);
        (self.0.on_change)(to);
    }

    /// Animate to `to` over `duration_ms`; zero (motion off) jumps.
    pub fn animate(&self, to: f64, duration_ms: f64, curve: Curve) {
        let inner = &self.0;
        if inner.tick.borrow().is_none() && (inner.value.get() - to).abs() < f64::EPSILON {
            inner.to.set(to);
            return;
        }
        if (inner.to.get() - to).abs() < f64::EPSILON && inner.tick.borrow().is_some() {
            return;
        }
        if duration_ms <= 0.0 {
            self.jump(to);
            return;
        }
        inner.from.set(inner.value.get());
        inner.to.set(to);
        inner.started.set(None);
        inner.duration_us.set((duration_ms * 1000.0) as i64);
        inner.curve.set(curve);
        if inner.tick.borrow().is_some() {
            return;
        }
        let Some(widget) = inner.widget.upgrade() else {
            return;
        };
        let weak = Rc::downgrade(&self.0);
        let id = widget.add_tick_callback(move |_, clock| {
            let Some(inner) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let now = clock.frame_time();
            let start = inner.started.get().unwrap_or_else(|| {
                inner.started.set(Some(now));
                now
            });
            let t = (now - start) as f64 / inner.duration_us.get() as f64;
            let done = t >= 1.0;
            let value = if done {
                inner.to.get()
            } else {
                let eased = inner.curve.get().ease(t);
                inner.from.get() + (inner.to.get() - inner.from.get()) * eased
            };
            inner.value.set(value);
            (inner.on_change)(value);
            if done {
                inner.tick.borrow_mut().take();
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
        *inner.tick.borrow_mut() = Some(id);
    }

    fn stop(&self) {
        if let Some(id) = self.0.tick.borrow_mut().take() {
            id.remove();
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Some(id) = self.tick.get_mut().take() {
            id.remove();
        }
    }
}
