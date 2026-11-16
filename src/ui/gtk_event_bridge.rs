//! Blocking backend event receiver forwarded onto the GTK main context.
//!
//! The session's std channel has one consumer. This adapter owns that receiver;
//! it never polls, and must be joined after `ApplicationRuntime::shutdown()`.

use crate::niri_session::NiriSessionEvent;
use gtk4::glib::MainContext;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

type LocalHandler = Box<dyn FnMut(NiriSessionEvent)>;

thread_local! {
    static HANDLERS: RefCell<HashMap<u64, LocalHandler>> = RefCell::new(HashMap::new());
}

static NEXT_HANDLER_ID: AtomicU64 = AtomicU64::new(1);

pub struct GtkEventBridge {
    handler_id: u64,
    worker: Option<JoinHandle<()>>,
}

impl GtkEventBridge {
    /// Spawn a single blocking receiver; every callback runs on the GTK thread.
    pub fn attach(
        receiver: Receiver<NiriSessionEvent>,
        context: MainContext,
        handler: impl FnMut(NiriSessionEvent) + 'static,
    ) -> Self {
        let handler_id = NEXT_HANDLER_ID.fetch_add(1, Ordering::Relaxed);
        HANDLERS.with(|handlers| handlers.borrow_mut().insert(handler_id, Box::new(handler)));

        let worker = thread::Builder::new()
            .name("velatura-gtk-event-bridge".into())
            .spawn(move || {
                while let Ok(event) = receiver.recv() {
                    context.invoke(move || dispatch_local(handler_id, event));
                }
            })
            .expect("failed to start GTK event bridge");

        Self {
            handler_id,
            worker: Some(worker),
        }
    }

    /// Join after the backend stops and closes its event sender.
    pub fn join(&mut self) {
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for GtkEventBridge {
    fn drop(&mut self) {
        HANDLERS.with(|handlers| {
            handlers.borrow_mut().remove(&self.handler_id);
        });
        // Do not join here: the receiver closes when the session shuts down.
        // Application shutdown explicitly stops the backend before calling join.
    }
}

fn dispatch_local(handler_id: u64, event: NiriSessionEvent) {
    HANDLERS.with(|handlers| {
        if let Some(handler) = handlers.borrow_mut().get_mut(&handler_id) {
            handler(event);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::niri::NiriState;
    use crate::niri_session::NiriSessionSnapshot;
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::{cell::RefCell, process::Command, rc::Rc};

    #[test]
    fn delivers_events_in_order_on_the_context_and_exits_on_disconnect() {
        let context = MainContext::new();
        let (sender, receiver) = mpsc::channel();
        let gtk_thread = thread::current().id();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let seen_by_handler = Arc::clone(&seen);
        let mut bridge = GtkEventBridge::attach(receiver, context.clone(), move |event| {
            if let NiriSessionEvent::SnapshotChanged(snapshot) = event {
                seen_by_handler
                    .lock()
                    .unwrap()
                    .push((snapshot.revision, thread::current().id()));
            }
        });

        for revision in [1, 2, 3] {
            sender
                .send(NiriSessionEvent::SnapshotChanged(NiriSessionSnapshot {
                    revision,
                    niri: NiriState::default(),
                    outputs: None,
                }))
                .unwrap();
        }
        drop(sender);

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while seen.lock().unwrap().len() < 3 && std::time::Instant::now() < deadline {
            context.iteration(true);
        }
        bridge.join();
        assert_eq!(
            *seen.lock().unwrap(),
            vec![(1, gtk_thread), (2, gtk_thread), (3, gtk_thread)]
        );
    }

    #[test]
    fn forwards_a_live_niri_snapshot_when_the_compositor_is_available() {
        let probe = Command::new("niri")
            .args(["msg", "--json", "outputs"])
            .output();
        if !matches!(probe, Ok(ref output) if output.status.success()) {
            eprintln!("skipping live GTK bridge test: Niri outputs query unavailable");
            return;
        }

        let mut session = crate::niri_session::NiriSession::spawn();
        let receiver = session.take_events().expect("event receiver available");
        let context = MainContext::new();
        let latest = Rc::new(RefCell::new(None));
        let latest_for_handler = Rc::clone(&latest);
        let mut bridge = GtkEventBridge::attach(receiver, context.clone(), move |event| {
            if let NiriSessionEvent::SnapshotChanged(snapshot) = event {
                *latest_for_handler.borrow_mut() = Some(snapshot);
            }
        });

        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            while context.iteration(false) {}
            if latest.borrow().as_ref().is_some_and(|snapshot| {
                snapshot.niri.connected && snapshot.outputs.as_ref().is_some_and(|v| !v.is_empty())
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }

        let snapshot = latest
            .borrow()
            .clone()
            .expect("live Niri snapshot received");
        assert!(snapshot.niri.connected);
        assert!(
            snapshot
                .outputs
                .as_ref()
                .is_some_and(|outputs| !outputs.is_empty())
        );
        assert!(snapshot.revision > 0);

        session.stop();
        bridge.join();
    }
}
