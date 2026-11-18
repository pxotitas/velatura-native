//! Native Velatura frontend: GTK4 + layer-shell, ported from the Quickshell
//! original (`~/.config/quickshell`), which stays the source of truth.

pub mod bar;
pub mod capsule;
pub mod chip;
pub mod clock;
pub mod controls_panel;
pub mod css;
pub mod fillet;
pub mod island;
pub mod island_rest;
pub mod island_summary;
pub mod launcher_panel;
pub mod launcher_results;
pub mod launcher_store;
pub mod motion;
pub mod notification_layer;
pub mod palette;
pub mod settings;
pub mod settings_app;
pub mod surface;
pub mod system_panel;
pub mod tokens;
pub mod workspaces;
pub mod zone;

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use bar::Bar;
use settings::Settings;

use crate::niri_session::{NiriSession, NiriSessionEvent, NiriSessionSnapshot};
use crate::services::clipboard::{ClipboardEvent, ClipboardService};
use crate::services::notifications::{NotificationEvent, NotificationService};

/// Owns every bar surface and keeps them in step with the monitors.
pub struct Shell {
    app: gtk4::Application,
    settings: Rc<Settings>,
    bars: RefCell<Vec<Bar>>,
    reserves: RefCell<Vec<surface::Reservation>>,
    session: Arc<NiriSession>,
    notifications: Rc<NotificationService>,
    clipboard: ClipboardService,
    _css: Rc<css::Theme>,
    _settings_app: Rc<settings_app::SettingsApp>,
    _palette: Option<gtk4::gio::FileMonitor>,
}

impl Shell {
    pub fn start(app: &gtk4::Application, session: Arc<NiriSession>) -> Rc<Self> {
        let settings = Rc::new(Settings::load());
        let settings_app = settings_app::SettingsApp::new(app);
        let settings_action = gtk4::gio::SimpleAction::new("settings", None);
        settings_action.connect_activate({
            let settings_app = Rc::clone(&settings_app);
            move |_, _| settings_app.present()
        });
        app.add_action(&settings_action);
        let shell = Rc::new(Self {
            app: app.clone(),
            _palette: palette::watch(),
            _css: css::install(&settings),
            _settings_app: settings_app,
            notifications: Rc::new(NotificationService::start()),
            clipboard: ClipboardService::start(
                settings.clipboard_history,
                settings.clipboard_keep,
                settings.clipboard_images,
            ),
            session,
            settings,
            bars: RefCell::new(Vec::new()),
            reserves: RefCell::new(Vec::new()),
        });
        shell.sync();
        // Niri state reaches the bars from the session event stream.
        let weak = Rc::downgrade(&shell);
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            let Some(shell) = weak.upgrade() else {
                return gtk4::glib::ControlFlow::Break;
            };
            while let Ok(event) = shell.session.try_recv() {
                if let NiriSessionEvent::SnapshotChanged(snapshot) = event {
                    shell
                        .bars
                        .borrow()
                        .iter()
                        .for_each(|bar| bar.update(&snapshot));
                    shell.follow_focus(&snapshot);
                }
            }
            while let Ok(event) = shell.notifications.try_recv() {
                shell.notified(event);
            }
            while let Ok(ClipboardEvent::EntriesChanged(_)) = shell.clipboard.try_recv() {
                for bar in shell.bars.borrow().iter() {
                    bar.island().refresh_clipboard();
                }
            }
            gtk4::glib::ControlFlow::Continue
        });
        // The original OsdService follows real system values, including
        // changes made by compositor keybindings outside this process.
        let weak = Rc::downgrade(&shell);
        let last_levels = Rc::new(RefCell::new(None));
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(350), move || {
            let Some(shell) = weak.upgrade() else {
                return gtk4::glib::ControlFlow::Break;
            };
            if let Some(levels) = crate::services::control_levels::read() {
                if let Some(previous) = last_levels.replace(Some(levels)) {
                    let event =
                        if previous.volume != levels.volume || previous.muted != levels.muted {
                            Some(crate::services::island_state::TransientEvent {
                                icon: if levels.muted || levels.volume == 0 {
                                    "󰖀"
                                } else {
                                    "󰕾"
                                }
                                .into(),
                                label: format!("{}%", levels.volume),
                                progress: f32::from(levels.volume) / 100.0,
                            })
                        } else if previous.brightness != levels.brightness {
                            Some(crate::services::island_state::TransientEvent {
                                icon: "󰃠".into(),
                                label: format!("{}%", levels.brightness),
                                progress: f32::from(levels.brightness) / 100.0,
                            })
                        } else {
                            None
                        };
                    if let Some(event) = event {
                        for bar in shell.bars.borrow().iter() {
                            bar.island().request_osd(event.clone());
                        }
                    }
                } else {
                    last_levels.replace(Some(levels));
                }
            }
            gtk4::glib::ControlFlow::Continue
        });
        let weak = Rc::downgrade(&shell);
        bar::watch_monitors(move || {
            if let Some(shell) = weak.upgrade() {
                shell.sync();
            }
        });
        shell
    }

    /// One bar per monitor: created on hotplug, destroyed on removal.
    fn sync(&self) {
        let monitors = bar::monitors();
        let name = |monitor: &gtk4::gdk::Monitor| {
            monitor
                .connector()
                .map(|name| name.to_string())
                .unwrap_or_default()
        };
        // Keep reservations independently owned from the visible bars, as
        // the original BarReserve.qml does. This prevents layout jumps when
        // a bar surface is recreated during monitor synchronization.
        let mut reserves = self.reserves.borrow_mut();
        reserves.retain(|reserve| {
            monitors
                .iter()
                .any(|monitor| name(monitor) == reserve.connector)
        });
        for monitor in &monitors {
            if !reserves
                .iter()
                .any(|reserve| reserve.connector == name(monitor))
            {
                reserves.push(surface::Reservation::new(
                    &self.app,
                    monitor,
                    self.settings.bar_reserve(),
                ));
            }
        }
        drop(reserves);

        let mut bars = self.bars.borrow_mut();
        bars.retain(|bar| {
            monitors
                .iter()
                .any(|monitor| name(monitor) == bar.connector)
        });
        for monitor in &monitors {
            if !bars.iter().any(|bar| bar.connector == name(monitor)) {
                let bar = Bar::new(
                    &self.app,
                    monitor,
                    Rc::clone(&self.settings),
                    Arc::clone(&self.session),
                    Rc::clone(&self.notifications),
                );
                bar.update(&self.session.snapshot());
                if let Some(toast) = self.notifications.current() {
                    bar.island().set_notification(Some(toast));
                }
                bars.push(bar);
            }
        }
        drop(bars);
        self.follow_focus(&self.session.snapshot());
    }

    /// A notification reaches every island; only the live one shows it.
    fn notified(&self, event: NotificationEvent) {
        let bars = self.bars.borrow();
        match event {
            NotificationEvent::Arrived(toast) => {
                bars.iter()
                    .for_each(|bar| bar.island().set_notification(Some(toast.clone())));
            }
            NotificationEvent::Expired(id) | NotificationEvent::Closed(id) => {
                for bar in bars
                    .iter()
                    .filter(|bar| bar.island().notification_id() == Some(id))
                {
                    bar.island().set_notification(None);
                }
            }
            _ => {}
        }
    }

    /// The live island is the one on the focused output (`Island.active`).
    fn follow_focus(&self, snapshot: &NiriSessionSnapshot) {
        let focused = snapshot
            .niri
            .focused_workspace_id
            .as_ref()
            .and_then(|id| id.as_i64());
        let output = snapshot
            .niri
            .workspaces
            .iter()
            .find(|ws| focused.is_some() && ws["id"].as_i64() == focused)
            .and_then(|ws| ws["output"].as_str().map(str::to_string));
        for bar in self.bars.borrow().iter() {
            let live = output.as_deref().is_none_or(|name| name == bar.connector);
            bar.island().set_active(live);
        }
    }
}
