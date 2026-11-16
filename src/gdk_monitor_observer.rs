//! GDK monitor topology and geometry observer.
//!
//! Must be created and dropped on the GTK/GDK main thread. GDK callbacks only
//! enqueue a debounced refresh; all Niri subprocess work stays off the UI thread.
use crate::niri_session::{NiriRefreshHandle, NiriSession};
use gdk4::prelude::*;
use gdk4::{Display, Monitor, gio, glib};
use std::cell::RefCell;
use std::rc::Rc;

type GeometryHandlers = Rc<RefCell<Vec<(Monitor, glib::SignalHandlerId)>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GdkMonitorObserverError {
    NoDefaultDisplay,
}

impl std::fmt::Display for GdkMonitorObserverError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDefaultDisplay => formatter.write_str("GDK has no default display"),
        }
    }
}

impl std::error::Error for GdkMonitorObserverError {}

/// Watches GDK monitor hotplug and per-monitor geometry notifications.
pub struct GdkMonitorObserver {
    // Keep the display and model alive for the full subscription lifetime.
    _display: Display,
    monitors: gio::ListModel,
    items_changed_handler: Option<glib::SignalHandlerId>,
    geometry_handlers: GeometryHandlers,
}

impl GdkMonitorObserver {
    /// Attach to the default display after GTK has initialized its application.
    pub fn attach_default(session: &NiriSession) -> Result<Self, GdkMonitorObserverError> {
        let display = Display::default().ok_or(GdkMonitorObserverError::NoDefaultDisplay)?;
        Ok(Self::attach(display, session.refresh_handle()))
    }

    /// Attach to an explicit display. Useful for embedding and integration tests.
    pub fn attach(display: Display, refresh: NiriRefreshHandle) -> Self {
        let monitors = display.monitors();
        let geometry_handlers: GeometryHandlers = Rc::new(RefCell::new(Vec::new()));

        let handlers_for_signal = Rc::clone(&geometry_handlers);
        let refresh_for_signal = refresh.clone();
        let items_changed_handler =
            monitors.connect_items_changed(move |model, _position, _removed, _added| {
                bind_geometry_notifications(model, &handlers_for_signal, &refresh_for_signal);
                let _ = refresh_for_signal.refresh_outputs();
            });

        bind_geometry_notifications(&monitors, &geometry_handlers, &refresh);

        Self {
            _display: display,
            monitors,
            items_changed_handler: Some(items_changed_handler),
            geometry_handlers,
        }
    }

    pub fn monitor_count(&self) -> u32 {
        self.monitors.n_items()
    }
}

impl Drop for GdkMonitorObserver {
    fn drop(&mut self) {
        if let Some(handler) = self.items_changed_handler.take() {
            self.monitors.disconnect(handler);
        }
        clear_geometry_notifications(&self.geometry_handlers);
    }
}

fn bind_geometry_notifications(
    model: &gio::ListModel,
    handlers: &GeometryHandlers,
    refresh: &NiriRefreshHandle,
) {
    clear_geometry_notifications(handlers);

    for index in 0..model.n_items() {
        let Some(monitor) = model
            .item(index)
            .and_then(|object| object.downcast::<Monitor>().ok())
        else {
            continue;
        };

        let refresh = refresh.clone();
        let handler = monitor.connect_geometry_notify(move |_| {
            let _ = refresh.refresh_outputs();
        });
        handlers.borrow_mut().push((monitor, handler));
    }
}

fn clear_geometry_notifications(handlers: &GeometryHandlers) {
    for (monitor, handler) in handlers.borrow_mut().drain(..) {
        monitor.disconnect(handler);
    }
}
