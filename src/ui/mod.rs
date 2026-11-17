//! Native Velatura frontend: GTK4 + layer-shell, ported from the Quickshell
//! original (`~/.config/quickshell`), which stays the source of truth.

pub mod bar;
pub mod capsule;
pub mod chip;
pub mod clock;
pub mod css;
pub mod fillet;
pub mod island;
pub mod motion;
pub mod palette;
pub mod settings;
pub mod surface;
pub mod tokens;
pub mod workspaces;
pub mod zone;

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use bar::Bar;
use settings::Settings;

use crate::niri_session::{NiriSession, NiriSessionEvent};

/// Owns every bar surface and keeps them in step with the monitors.
pub struct Shell {
    app: gtk4::Application,
    settings: Rc<Settings>,
    bars: RefCell<Vec<Bar>>,
    session: Arc<NiriSession>,
    _css: gtk4::CssProvider,
    _palette: Option<gtk4::gio::FileMonitor>,
}

impl Shell {
    pub fn start(app: &gtk4::Application, session: Arc<NiriSession>) -> Rc<Self> {
        let settings = Rc::new(Settings::load());
        let shell = Rc::new(Self {
            app: app.clone(),
            _palette: palette::watch(),
            _css: css::install(&settings),
            session,
            settings,
            bars: RefCell::new(Vec::new()),
        });
        shell.sync();
        // Niri state reaches the bars from the session event stream.
        let weak = Rc::downgrade(&shell);
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            let Some(shell) = weak.upgrade() else { return gtk4::glib::ControlFlow::Break };
            while let Ok(event) = shell.session.try_recv() {
                if let NiriSessionEvent::SnapshotChanged(snapshot) = event {
                    shell.bars.borrow().iter().for_each(|bar| bar.update(&snapshot));
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
            monitor.connector().map(|name| name.to_string()).unwrap_or_default()
        };
        let mut bars = self.bars.borrow_mut();
        bars.retain(|bar| monitors.iter().any(|monitor| name(monitor) == bar.connector));
        for monitor in &monitors {
            if !bars.iter().any(|bar| bar.connector == name(monitor)) {
                let bar = Bar::new(
                    &self.app,
                    monitor,
                    Rc::clone(&self.settings),
                    Arc::clone(&self.session),
                    Rc::new(|_panel: &str| {}),
                );
                bar.update(&self.session.snapshot());
                bars.push(bar);
            }
        }
    }
}
