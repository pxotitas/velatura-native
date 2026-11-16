//! GDK monitor topology and geometry observer.
//!
//! Must be created and dropped on the GTK/GDK main thread. GDK callbacks only
//! enqueue a debounced refresh; all Niri subprocess work stays off the UI thread.
use crate::niri_outputs::NiriOutput;
use crate::niri_session::{NiriRefreshHandle, NiriSession};
use gdk4::prelude::*;
use gdk4::{Display, Monitor, gio, glib};
use std::cell::RefCell;
use std::rc::Rc;

type GeometryHandlers = Rc<RefCell<Vec<(Monitor, glib::SignalHandlerId)>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GdkMonitorInfo {
    pub index: u32,
    pub connector: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale_factor: i32,
    pub refresh_rate_millihz: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorMatchKind {
    Connector,
    Geometry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorMatch {
    pub gdk_index: u32,
    pub output_name: String,
    pub kind: MonitorMatchKind,
}

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
    monitor_info: Rc<RefCell<Vec<GdkMonitorInfo>>>,
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
        let monitor_info = Rc::new(RefCell::new(read_monitor_info(&monitors)));

        let handlers_for_signal = Rc::clone(&geometry_handlers);
        let info_for_signal = Rc::clone(&monitor_info);
        let refresh_for_signal = refresh.clone();
        let items_changed_handler =
            monitors.connect_items_changed(move |model, _position, _removed, _added| {
                bind_geometry_notifications(
                    model,
                    &handlers_for_signal,
                    &refresh_for_signal,
                    &info_for_signal,
                );
                *info_for_signal.borrow_mut() = read_monitor_info(model);
                let _ = refresh_for_signal.refresh_outputs();
            });

        bind_geometry_notifications(&monitors, &geometry_handlers, &refresh, &monitor_info);

        Self {
            _display: display,
            monitors,
            items_changed_handler: Some(items_changed_handler),
            geometry_handlers,
            monitor_info,
        }
    }

    pub fn monitor_count(&self) -> u32 {
        self.monitors.n_items()
    }

    /// Current GDK monitor inventory; returned records contain no GTK objects.
    pub fn monitor_info(&self) -> Vec<GdkMonitorInfo> {
        self.monitor_info.borrow().clone()
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
    monitor_info: &Rc<RefCell<Vec<GdkMonitorInfo>>>,
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
        let monitor_info = Rc::clone(monitor_info);
        let monitor_for_signal = monitor.clone();
        let handler = monitor.connect_geometry_notify(move |_| {
            if let Some(info) = read_one_monitor(&monitor_for_signal, index) {
                let mut records = monitor_info.borrow_mut();
                if let Some(existing) = records.iter_mut().find(|item| item.index == index) {
                    *existing = info;
                }
            }
            let _ = refresh.refresh_outputs();
        });
        handlers.borrow_mut().push((monitor, handler));
    }
}

fn read_monitor_info(model: &gio::ListModel) -> Vec<GdkMonitorInfo> {
    (0..model.n_items())
        .filter_map(|index| {
            model
                .item(index)
                .and_then(|object| object.downcast::<Monitor>().ok())
                .and_then(|monitor| read_one_monitor(&monitor, index))
        })
        .collect()
}

fn read_one_monitor(monitor: &Monitor, index: u32) -> Option<GdkMonitorInfo> {
    let geometry = monitor.geometry();
    Some(GdkMonitorInfo {
        index,
        connector: monitor.connector().map(|value| value.to_string()),
        manufacturer: monitor.manufacturer().map(|value| value.to_string()),
        model: monitor.model().map(|value| value.to_string()),
        x: geometry.x(),
        y: geometry.y(),
        width: geometry.width(),
        height: geometry.height(),
        scale_factor: monitor.scale_factor(),
        refresh_rate_millihz: monitor.refresh_rate(),
    })
}

/// Match GDK monitors to active Niri outputs. Connector equality is preferred;
/// geometry is a fallback only when unique on both sides. No name/model guesses.
pub fn match_gdk_monitors(
    monitors: &[GdkMonitorInfo],
    outputs: &[NiriOutput],
) -> Vec<MonitorMatch> {
    let mut matches = Vec::new();
    let mut used_outputs = std::collections::HashSet::new();
    let mut unmatched_monitors = Vec::new();

    for monitor in monitors {
        let Some(connector) = monitor
            .connector
            .as_deref()
            .filter(|value| !value.is_empty())
        else {
            unmatched_monitors.push(monitor);
            continue;
        };
        let candidates: Vec<_> = outputs
            .iter()
            .enumerate()
            .filter(|(index, output)| {
                !output.disabled && !used_outputs.contains(index) && output.name == connector
            })
            .collect();
        if let [(output_index, output)] = candidates.as_slice() {
            used_outputs.insert(*output_index);
            matches.push(MonitorMatch {
                gdk_index: monitor.index,
                output_name: output.name.clone(),
                kind: MonitorMatchKind::Connector,
            });
        } else {
            unmatched_monitors.push(monitor);
        }
    }

    for monitor in unmatched_monitors {
        let gdk_geometry_matches = monitors
            .iter()
            .filter(|other| same_geometry(monitor, other))
            .count();
        let output_candidates: Vec<_> = outputs
            .iter()
            .enumerate()
            .filter(|(index, output)| {
                !output.disabled
                    && !used_outputs.contains(index)
                    && output.x == i64::from(monitor.x)
                    && output.y == i64::from(monitor.y)
                    && output.width == i64::from(monitor.width)
                    && output.height == i64::from(monitor.height)
            })
            .collect();
        if gdk_geometry_matches == 1
            && let [(output_index, output)] = output_candidates.as_slice()
        {
            let output_geometry_matches = outputs
                .iter()
                .filter(|candidate| {
                    !candidate.disabled
                        && candidate.x == output.x
                        && candidate.y == output.y
                        && candidate.width == output.width
                        && candidate.height == output.height
                })
                .count();
            if output_geometry_matches == 1 {
                used_outputs.insert(*output_index);
                matches.push(MonitorMatch {
                    gdk_index: monitor.index,
                    output_name: output.name.clone(),
                    kind: MonitorMatchKind::Geometry,
                });
            }
        }
    }
    matches.sort_by_key(|matched| matched.gdk_index);
    matches
}

fn same_geometry(left: &GdkMonitorInfo, right: &GdkMonitorInfo) -> bool {
    left.x == right.x
        && left.y == right.y
        && left.width == right.width
        && left.height == right.height
}

fn clear_geometry_notifications(handlers: &GeometryHandlers) {
    for (monitor, handler) in handlers.borrow_mut().drain(..) {
        monitor.disconnect(handler);
    }
}

#[cfg(test)]
mod matching_tests {
    use super::*;
    use crate::niri_outputs::NiriResolution;

    fn output(name: &str, x: i64, disabled: bool) -> NiriOutput {
        NiriOutput {
            name: name.into(),
            description: String::new(),
            make: String::new(),
            model: String::new(),
            serial: String::new(),
            x,
            y: 0,
            width: 100,
            height: 80,
            refresh_rate: 60.0,
            mode: "100x80@60".into(),
            scale: 1.0,
            transform: "Normal".into(),
            disabled,
            vrr: false,
            vrr_supported: false,
            resolutions: vec![NiriResolution {
                width: 100,
                height: 80,
                refreshes: vec![60.0],
            }],
        }
    }

    fn monitor(index: u32, connector: Option<&str>, x: i32) -> GdkMonitorInfo {
        GdkMonitorInfo {
            index,
            connector: connector.map(str::to_owned),
            manufacturer: None,
            model: None,
            x,
            y: 0,
            width: 100,
            height: 80,
            scale_factor: 1,
            refresh_rate_millihz: 60_000,
        }
    }

    #[test]
    fn matches_connectors_first_and_uses_only_unique_geometry_fallback() {
        let monitors = vec![monitor(0, Some("DP-1"), 0), monitor(1, None, 100)];
        let outputs = vec![output("DP-1", 0, false), output("Virtual-1", 100, false)];
        assert_eq!(
            match_gdk_monitors(&monitors, &outputs),
            vec![
                MonitorMatch {
                    gdk_index: 0,
                    output_name: "DP-1".into(),
                    kind: MonitorMatchKind::Connector
                },
                MonitorMatch {
                    gdk_index: 1,
                    output_name: "Virtual-1".into(),
                    kind: MonitorMatchKind::Geometry
                },
            ]
        );
    }

    #[test]
    fn leaves_ambiguous_disabled_and_removed_outputs_unmatched() {
        let monitors = vec![
            monitor(0, None, 0),
            monitor(1, None, 0),
            monitor(2, Some("gone"), 200),
        ];
        let outputs = vec![
            output("A", 0, false),
            output("B", 0, false),
            output("off", 200, true),
        ];
        assert!(match_gdk_monitors(&monitors, &outputs).is_empty());
    }

    #[test]
    fn removal_reconnection_and_geometry_change_are_reflected_by_next_inventory() {
        let original_output = output("eDP-1", 0, false);
        let original_monitor = monitor(0, Some("eDP-1"), 0);
        assert_eq!(
            match_gdk_monitors(
                std::slice::from_ref(&original_monitor),
                std::slice::from_ref(&original_output),
            )[0]
            .output_name,
            "eDP-1"
        );
        assert!(match_gdk_monitors(&[], std::slice::from_ref(&original_output)).is_empty());

        let moved_output = output("eDP-1", 300, false);
        let reconnected_monitor = monitor(0, Some("eDP-1"), 300);
        let reconnected = match_gdk_monitors(
            std::slice::from_ref(&reconnected_monitor),
            std::slice::from_ref(&moved_output),
        );
        assert_eq!(reconnected.len(), 1);
        assert_eq!(reconnected[0].output_name, "eDP-1");
        assert_eq!(reconnected[0].kind, MonitorMatchKind::Connector);
    }
}
