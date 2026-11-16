//! Application-owned backend lifecycle.
//!
//! Construct once from the GTK application's activate path, retain it for the
//! application's lifetime, and call shutdown from the application's shutdown
//! path. The UI owns the GLib main loop; this type owns backend workers and
//! GDK monitor subscriptions.
use crate::gdk_monitor_observer::{GdkMonitorObserver, GdkMonitorObserverError};
use crate::niri_session::NiriSession;

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
    session: Option<NiriSession>,
    monitor_observer: Option<GdkMonitorObserver>,
}

impl ApplicationRuntime {
    /// Start using the Niri executable from PATH and GTK's default GDK display.
    pub fn start() -> Result<Self, ApplicationRuntimeError> {
        Self::start_with_program("niri")
    }

    /// Injectable executable for integration tests and custom installations.
    pub fn start_with_program(program: impl Into<String>) -> Result<Self, ApplicationRuntimeError> {
        let session = NiriSession::with_program(program);
        let monitor_observer = GdkMonitorObserver::attach_default(&session)
            .map_err(ApplicationRuntimeError::MonitorObserver)?;

        Ok(Self {
            session: Some(session),
            monitor_observer: Some(monitor_observer),
        })
    }

    /// Access snapshots, signals, diagnostics and action results for the UI.
    pub fn session(&self) -> &NiriSession {
        self.session
            .as_ref()
            .expect("application runtime is shut down")
    }

    /// Number of monitors currently exposed by GDK.
    pub fn monitor_count(&self) -> u32 {
        self.monitor_observer
            .as_ref()
            .map_or(0, GdkMonitorObserver::monitor_count)
    }

    /// Disconnect UI-thread GDK observers before stopping backend workers.
    /// Safe to call more than once.
    pub fn shutdown(&mut self) {
        self.monitor_observer.take();
        if let Some(mut session) = self.session.take() {
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
