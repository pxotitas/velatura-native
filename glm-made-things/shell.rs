//! Shell composition: one bar surface per monitor with the live island.
//!
//! Port of `shell.qml` + `bar/Bar.qml`'s structure: left zone, the Dynamic
//! Island in the middle, right zone, with the exclusive reserve band keeping
//! windows below the capsule. Exactly one island is live; the rest stay at
//! rest, whatever arrives (`Bar.live`).

use std::rc::Rc;
use std::sync::Arc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Button, Fixed, Image, Label};

use crate::application_runtime::ApplicationRuntime;
use crate::niri_session::{NiriSession, NiriSessionEvent, NiriSessionSnapshot};
use crate::services::clipboard::ClipboardService;
use crate::services::island_state::Layer;
use crate::services::notifications::{NotificationEvent, NotificationService};

use super::island::Island;
use super::layer;
use super::theme;

/// Typed handles the UI consumes instead of touching system APIs directly.
/// UI widgets never run commands or touch the filesystem themselves.
#[derive(Clone)]
pub struct ServiceHandles {
    pub notifications: Arc<NotificationService>,
    pub clipboard: Arc<ClipboardService>,
    pub niri: Arc<NiriSession>,
    /// The live island listens here for `open panel` requests from widgets.
    pub panel_requests: async_channel::Sender<String>,
}

/// Everything a bar's event updates need, cloned into the pumps.
#[derive(Clone)]
struct BarRefs {
    island: Rc<std::cell::RefCell<Island>>,
    workspaces_box: gtk4::Box,
    notif_count: Label,
    notif_button: Button,
    canvas: Fixed,
    morph: super::morph::MorphBox,
    height_driver: gtk4::DrawingArea,
}

impl BarRefs {
    /// Centre the island and right-align the notification module. Runs on
    /// allocation callbacks, not on animation frames.
    fn layout(&self, width: i32) {
        let (island_width, _) = self.island.borrow().current_size();
        let center = ((width as f64 - island_width) / 2.0).round().max(0.0);
        self.canvas
            .move_(&self.morph, center, f64::from(theme::BAR_TOP_MARGIN));

        let preferred = self
            .notif_button
            .measure(gtk4::Orientation::Horizontal, -1)
            .1;
        let x = (width - preferred - theme::BAR_SIDE_MARGIN).max(0);
        self.canvas.move_(
            &self.notif_button,
            f64::from(x),
            f64::from(theme::BAR_TOP_MARGIN),
        );
    }

    /// Stretch the surface to hold an open panel; two discrete sizes, never
    /// a resize per animation frame. The exclusive zone stays at the reserve
    /// band, so windows keep clear of the capsule only.
    fn update_height(&self) {
        let needed = match self.island.borrow().layer() {
            Layer::Panel | Layer::Notification => {
                let (_, height) = self.island.borrow().current_size();
                theme::BAR_TOP_MARGIN + height as i32 + 8
            }
            _ => theme::BAR_RESERVE + 8,
        };
        self.height_driver.set_content_height(needed);
        self.height_driver.queue_resize();
    }

    /// Rebuild the workspace pills from the live Niri snapshot, matching the
    /// original's `WorkspacesWidget`.
    fn update_workspaces(&self, snapshot: &NiriSessionSnapshot) {
        while let Some(child) = self.workspaces_box.first_child() {
            self.workspaces_box.remove(&child);
        }
        let focused_id = snapshot
            .niri
            .focused_workspace_id
            .as_ref()
            .and_then(|value| value.as_i64());
        let mut indices: Vec<(i64, i64, bool)> = Vec::new();
        for workspace in &snapshot.niri.workspaces {
            let Some(idx) = workspace.get("idx").and_then(|value| value.as_i64()) else {
                continue;
            };
            let id = workspace
                .get("id")
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let is_focused = focused_id == Some(id)
                || workspace
                    .get("is_focused")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
            indices.push((idx, id, is_focused));
        }
        indices.sort_by_key(|(idx, _, _)| *idx);
        let niri = self.island.borrow().niri();
        for (idx, id, is_focused) in indices {
            let button = Button::new();
            button.set_label(&idx.to_string());
            button.set_has_frame(false);
            button.set_css_classes(if is_focused {
                &["velatura-module-label"]
            } else {
                &["velatura-module-muted"]
            });
            let niri = Arc::clone(&niri);
            button.connect_clicked(move |_| {
                let _ = niri.dispatch(["action", "focus-workspace", &id.to_string()]);
            });
            self.workspaces_box.append(&button);
        }
    }
}

/// One monitor's bar: a layer surface with the three zones on a fixed
/// canvas, the island in the middle.
pub struct BarHandle {
    /// The layer surface; kept alive for the bar's lifetime.
    #[allow(dead_code)]
    window: ApplicationWindow,
    refs: BarRefs,
}

impl BarHandle {
    fn new(
        application: &gtk4::Application,
        services: &ServiceHandles,
        monitor: &gtk4::gdk::Monitor,
        connector: Option<String>,
        active: bool,
    ) -> Self {
        let window = ApplicationWindow::new(application);
        window.set_css_classes(&["velatura-bar"]);
        layer::setup_bar_surface(&window, Some(monitor));

        let canvas = Fixed::new();
        window.set_child(Some(&canvas));

        // ── LEFT: workspaces ────────────────────────────────────────────────
        let workspaces_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        workspaces_box.set_css_classes(&["velatura-capsule"]);
        workspaces_box.set_valign(gtk4::Align::Center);

        // ── CENTER: the island ──────────────────────────────────────────────
        let island = Island::new(services.clone(), connector, active);
        let morph = island.borrow().morph.clone();

        // ── RIGHT: notifications ────────────────────────────────────────────
        let right_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let bell = Image::new();
        bell.set_icon_name(Some("preferences-system-notifications-symbolic"));
        let notif_count = Label::new(None);
        notif_count.set_css_classes(&["velatura-module-muted"]);
        right_box.append(&bell);
        right_box.append(&notif_count);
        let notif_button = Button::new();
        notif_button.set_child(Some(&right_box));
        notif_button.set_css_classes(&["velatura-module"]);
        notif_button.set_has_frame(false);
        notif_button.set_halign(gtk4::Align::Start);
        {
            let island = Rc::clone(&island);
            let services = services.clone();
            notif_button.connect_clicked(move |_| {
                if let Ok(mut island) = island.try_borrow_mut() {
                    island.open("notifications");
                } else {
                    let _ = services
                        .panel_requests
                        .send_blocking("notifications".to_string());
                }
            });
        }

        // Height driver: the surface follows the island's needs in two
        // discrete sizes; the exclusive zone stays at the reserve band.
        let height_driver = gtk4::DrawingArea::new();
        height_driver.set_content_height(theme::BAR_RESERVE + 8);

        canvas.put(&height_driver, 0.0, 0.0);
        canvas.put(
            &workspaces_box,
            f64::from(theme::BAR_SIDE_MARGIN),
            f64::from(theme::BAR_TOP_MARGIN),
        );
        canvas.put(&morph, 0.0, f64::from(theme::BAR_TOP_MARGIN));
        canvas.put(&notif_button, 0.0, f64::from(theme::BAR_TOP_MARGIN));

        let refs = BarRefs {
            island: Rc::clone(&island),
            workspaces_box,
            notif_count,
            notif_button,
            canvas,
            morph: morph.clone(),
            height_driver,
        };
        // Keep the zones laid out as the surface resizes (hotplug, geometry).
        {
            let refs = refs.clone();
            morph.connect_allocate(move |_width, _height| {
                refs.layout(bar_width(&refs.morph));
                refs.update_height();
            });
        }
        // GTK4 windows are hidden until presented; presenting here is what
        // actually maps the layer surface on the Wayland output.
        window.present();
        Self { window, refs }
    }
}

/// The composed shell. Built on the application's activate path; its event
/// sources are removed on shutdown.
pub struct Shell {
    bars: Vec<BarHandle>,
    /// The services outlive the bars through the `Arc` clones inside them;
    /// this handle keeps the original owners alive for the shell's lifetime.
    #[allow(dead_code)]
    handles: ServiceHandles,
    sources: Vec<glib::SourceId>,
    forwarders: Vec<Forwarder>,
}

impl Shell {
    /// Build the shell surfaces and start the event pumps. Returns the shell
    /// to keep alive for the application's lifetime.
    pub fn build(application: &gtk4::Application, runtime: &mut ApplicationRuntime) -> Shell {
        let (panel_requests_tx, panel_requests_rx) = async_channel::unbounded::<String>();
        let handles = ServiceHandles {
            notifications: Arc::new(NotificationService::start()),
            clipboard: Arc::new(ClipboardService::start()),
            niri: runtime.session(),
            panel_requests: panel_requests_tx,
        };

        // One bar per GDK monitor; the connector links each bar to its Niri
        // output for the live-island rule. The first monitor hosts the live
        // island until the reducer tracks the focused output.
        let monitor_records = runtime.monitor_info();
        let mut bars = Vec::new();
        if let Some(display) = gtk4::gdk::Display::default() {
            let monitors = display.monitors();
            for index in 0..monitors.n_items() {
                let Some(monitor) = monitors
                    .item(index)
                    .and_then(|object| object.downcast::<gtk4::gdk::Monitor>().ok())
                else {
                    continue;
                };
                let connector = monitor_records
                    .get(index as usize)
                    .and_then(|info| info.connector.clone());
                bars.push(BarHandle::new(
                    application,
                    &handles,
                    &monitor,
                    connector,
                    index == 0,
                ));
            }
        }
        let mut shell = Shell {
            bars,
            handles: handles.clone(),
            sources: Vec::new(),
            forwarders: Vec::new(),
        };

        // ── NIRI EVENTS ─────────────────────────────────────────────────────
        // Snapshots drive the workspaces widget, the surface height and the
        // notification count. No polling: the bridge blocks on the channel.
        if let Some(receiver) = runtime.take_events() {
            let refs: Vec<BarRefs> = shell.bars.iter().map(|bar| bar.refs.clone()).collect();
            let (tx, rx) = async_channel::unbounded::<NiriSessionEvent>();
            shell.forwarders.push(Forwarder::spawn(receiver, tx));
            let source = glib::MainContext::default().spawn_local(async move {
                while let Ok(event) = rx.recv().await {
                    if let NiriSessionEvent::SnapshotChanged(snapshot) = &event {
                        for bar in &refs {
                            bar.update_workspaces(snapshot);
                            bar.update_height();
                            bar.layout(bar_width(&bar.morph));
                        }
                    }
                }
            });
            shell.sources.push(
                glib::JoinHandle::into_source_id(source)
                    .expect("shell futures run on the main context"),
            );
        }

        // ── NOTIFICATION EVENTS ─────────────────────────────────────────────
        {
            let refs: Vec<BarRefs> = shell.bars.iter().map(|bar| bar.refs.clone()).collect();
            let (tx, rx) = async_channel::unbounded::<NotificationEvent>();
            shell.forwarders.push(Forwarder::spawn_from_notifications(
                Arc::clone(&handles.notifications),
                tx,
            ));
            let source = glib::MainContext::default().spawn_local(async move {
                while let Ok(event) = rx.recv().await {
                    match &event {
                        NotificationEvent::Arrived(toast) => {
                            for bar in &refs {
                                let mut island = bar.island.borrow_mut();
                                if island.is_live() {
                                    island.notification_arrived(toast.clone());
                                }
                            }
                        }
                        NotificationEvent::Expired(_) | NotificationEvent::Closed(_) => {
                            for bar in &refs {
                                bar.island.borrow_mut().notification_gone();
                            }
                        }
                        NotificationEvent::HistoryChanged(history) => {
                            for bar in &refs {
                                bar.island.borrow_mut().history_changed(history.clone());
                                bar.notif_count.set_text(&match history.len() {
                                    0 => String::new(),
                                    1..=9 => history.len().to_string(),
                                    _ => "9+".to_string(),
                                });
                            }
                        }
                        _ => {}
                    }
                    for bar in &refs {
                        bar.update_height();
                        bar.layout(bar_width(&bar.morph));
                    }
                }
            });
            shell.sources.push(
                glib::JoinHandle::into_source_id(source)
                    .expect("shell futures run on the main context"),
            );
        }

        // ── CLIPBOARD EVENTS ────────────────────────────────────────────────
        // The launcher reads entries when it opens; the pump only keeps the
        // bridge contract uniform and will carry clipboard UI updates later.
        {
            let (tx, rx) = async_channel::unbounded::<crate::services::clipboard::ClipboardEvent>();
            shell.forwarders.push(Forwarder::spawn_from_clipboard(
                Arc::clone(&handles.clipboard),
                tx,
            ));
            let source = glib::MainContext::default().spawn_local(async move {
                while (rx.recv().await).is_ok() {
                    // Clipboard updates reach the UI through the launcher's
                    // reload on open; nothing to draw per event yet.
                }
            });
            shell.sources.push(
                glib::JoinHandle::into_source_id(source)
                    .expect("shell futures run on the main context"),
            );
        }

        // ── PANEL REQUESTS ──────────────────────────────────────────────────
        {
            let refs: Vec<BarRefs> = shell.bars.iter().map(|bar| bar.refs.clone()).collect();
            let source = glib::MainContext::default().spawn_local(async move {
                while let Ok(panel) = panel_requests_rx.recv().await {
                    // Only the live island opens panels.
                    for bar in &refs {
                        if bar.island.borrow().is_live() {
                            bar.island.borrow_mut().open(&panel);
                            bar.update_height();
                        }
                    }
                }
            });
            shell.sources.push(
                glib::JoinHandle::into_source_id(source)
                    .expect("shell futures run on the main context"),
            );
        }

        // ── CLOCK ───────────────────────────────────────────────────────────
        // One timer updates every island's clock and expires transients
        // through `tick` at the arbitration's resolution.
        {
            let refs: Vec<BarRefs> = shell.bars.iter().map(|bar| bar.refs.clone()).collect();
            let source =
                glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
                    for bar in &refs {
                        bar.island.borrow_mut().tick(crate::services::now_ms());
                    }
                    glib::ControlFlow::Continue
                });
            shell.sources.push(source);
        }

        // ── INITIAL STATE ───────────────────────────────────────────────────
        {
            let snapshot = runtime.snapshot();
            for bar in &shell.bars {
                bar.refs.update_workspaces(&snapshot);
                let count = handles.notifications.history().len();
                bar.refs.notif_count.set_text(&match count {
                    0 => String::new(),
                    1..=9 => count.to_string(),
                    _ => "9+".to_string(),
                });
                bar.refs.layout(bar_width(&bar.refs.morph));
                bar.refs.update_height();
            }
        }

        shell
    }

    /// Mark one monitor's island live and every other at rest, like
    /// `Bar.live`. The focused output follows the Niri snapshot.
    pub fn set_live_output(&mut self, focused_output: Option<&str>) {
        for bar in &self.bars {
            let is_focused = focused_output.is_none_or(|focused| {
                bar.refs.island.borrow().output_connector() == Some(focused.to_string())
            });
            bar.refs.island.borrow_mut().set_active(is_focused);
            bar.refs.update_height();
        }
    }
}

fn bar_width(morph: &super::morph::MorphBox) -> i32 {
    gtk4::prelude::WidgetExt::root(morph)
        .and_then(|root| root.dynamic_cast::<ApplicationWindow>().ok())
        .map(|window| window.width().max(1))
        .unwrap_or(1)
}

impl Drop for Shell {
    fn drop(&mut self) {
        for source in self.sources.drain(..) {
            source.remove();
        }
        for forwarder in self.forwarders.drain(..) {
            forwarder
                .stopped
                .store(true, std::sync::atomic::Ordering::SeqCst);
            // The thread ends on its next wake; never join on the UI thread.
        }
    }
}

/// A backend-channel forwarder: blocks on the service's channel in a worker
/// thread and re-emits into the GTK main context. No polling loops.
struct Forwarder {
    #[allow(dead_code)]
    thread: std::thread::JoinHandle<()>,
    stopped: Arc<std::sync::atomic::AtomicBool>,
}

impl Forwarder {
    fn stopped() -> Arc<std::sync::atomic::AtomicBool> {
        Arc::new(std::sync::atomic::AtomicBool::new(false))
    }

    fn spawn_loop<T, F>(
        stopped: Arc<std::sync::atomic::AtomicBool>,
        next: F,
        tx: async_channel::Sender<T>,
    ) -> std::thread::JoinHandle<()>
    where
        T: Send + 'static,
        F: FnMut() -> Option<T> + Send + 'static,
    {
        let thread_stopped = stopped;
        std::thread::Builder::new()
            .name("velatura-bridge".into())
            .spawn(move || {
                let mut next = next;
                loop {
                    if thread_stopped.load(std::sync::atomic::Ordering::SeqCst) {
                        return;
                    }
                    match next() {
                        Some(event) => {
                            if tx.send_blocking(event).is_err() {
                                return;
                            }
                        }
                        None => return,
                    }
                }
            })
            .expect("bridge thread spawn cannot fail")
    }

    fn spawn(
        receiver: std::sync::mpsc::Receiver<NiriSessionEvent>,
        tx: async_channel::Sender<NiriSessionEvent>,
    ) -> Self {
        let stopped = Self::stopped();
        let thread = Self::spawn_loop(Arc::clone(&stopped), move || receiver.recv().ok(), tx);
        Self { thread, stopped }
    }

    fn spawn_from_notifications(
        notifications: Arc<NotificationService>,
        tx: async_channel::Sender<NotificationEvent>,
    ) -> Self {
        let stopped = Self::stopped();
        let thread = Self::spawn_loop(Arc::clone(&stopped), move || notifications.recv().ok(), tx);
        Self { thread, stopped }
    }

    fn spawn_from_clipboard(
        clipboard: Arc<ClipboardService>,
        tx: async_channel::Sender<crate::services::clipboard::ClipboardEvent>,
    ) -> Self {
        let stopped = Self::stopped();
        let thread = Self::spawn_loop(Arc::clone(&stopped), move || clipboard.recv().ok(), tx);
        Self { thread, stopped }
    }
}
