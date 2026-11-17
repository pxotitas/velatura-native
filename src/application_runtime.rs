//! Application-owned backend lifecycle.
//!
//! Construct once from the GTK application's activate path, retain it for the
//! application's lifetime, and call shutdown from the application's shutdown
//! path. The UI owns the GLib main loop; this type owns backend workers and
//! GDK monitor subscriptions.
use crate::gdk_monitor_observer::{GdkMonitorInfo, GdkMonitorObserver, GdkMonitorObserverError};
use crate::gdk_monitor_observer::{MonitorMatch, match_gdk_monitors};
use crate::niri_session::{NiriSession, NiriSessionEvent, NiriSessionSnapshot};
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum ApplicationRuntimeError {
    MonitorObserver(GdkMonitorObserverError),
}

impl std::fmt::Display for ApplicationRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MonitorObserver(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ApplicationRuntimeError {}

/// Owns the Niri backend and GDK observer for one application lifetime.
pub struct ApplicationRuntime {
    session: Option<std::sync::Arc<NiriSession>>,
    monitor_observer: Option<GdkMonitorObserver>,
}

impl ApplicationRuntime {
    /// Start using the Niri executable from PATH and GTK's default GDK display.
    pub fn start() -> Result<Self, ApplicationRuntimeError> {
        Self::start_with_program("niri")
    }

    /// Injectable executable for integration tests and custom installations.
    pub fn start_with_program(program: impl Into<String>) -> Result<Self, ApplicationRuntimeError> {
        let session = std::sync::Arc::new(NiriSession::with_program(program));
        let monitor_observer = GdkMonitorObserver::attach_default(&session)
            .map_err(ApplicationRuntimeError::MonitorObserver)?;

        Ok(Self {
            session: Some(session),
            monitor_observer: Some(monitor_observer),
        })
    }

    /// Access snapshots, signals, diagnostics and action results for the UI.
    pub fn session(&self) -> std::sync::Arc<NiriSession> {
        self.session
            .as_ref()
            .expect("application runtime is shut down")
            .clone()
    }

    /// Number of monitors currently exposed by GDK.
    pub fn monitor_count(&self) -> u32 {
        self.monitor_observer
            .as_ref()
            .map_or(0, GdkMonitorObserver::monitor_count)
    }

    /// Current GDK monitor inventory, captured on the GTK thread.
    pub fn monitor_info(&self) -> Vec<GdkMonitorInfo> {
        self.monitor_observer
            .as_ref()
            .map_or_else(Vec::new, GdkMonitorObserver::monitor_info)
    }

    /// Resolve current GDK monitors to the latest active Niri output inventory.
    pub fn monitor_matches(&self) -> Vec<MonitorMatch> {
        let Some(outputs) = self.snapshot().outputs else {
            return Vec::new();
        };
        match_gdk_monitors(&self.monitor_info(), &outputs)
    }

    /// Get the latest atomic Niri/output snapshot for presentation adapters.
    pub fn snapshot(&self) -> NiriSessionSnapshot {
        self.session().snapshot()
    }

    /// Transfer the session's single event consumer to the presentation bridge.
    pub fn take_events(&mut self) -> Option<Receiver<NiriSessionEvent>> {
        self.session.as_ref()?.take_events()
    }

    /// Disconnect UI-thread GDK observers before stopping backend workers.
    /// Safe to call more than once.
    pub fn shutdown(&mut self) {
        self.monitor_observer.take();
        if let Some(session) = self.session.take() {
            session.stop();
        }
    }
}

impl Drop for ApplicationRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_attaches_to_gdk_and_shutdown_is_idempotent() {
        // GDK cannot open a display before GTK initialization. Keep this
        // integration check feature-gated and harmless in headless CI.
        if unsafe { gtk4_sys::gtk_init_check() } == 0 {
            eprintln!("skipping application runtime test: GTK could not open a display");
            return;
        }

        let mut runtime = ApplicationRuntime::start().expect("default GDK display should exist");
        assert!(runtime.monitor_count() > 0);
        runtime.shutdown();
        runtime.shutdown();
    }
}
