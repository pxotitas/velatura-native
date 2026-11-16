//! Core domain logic for the native Velatura backend.
//!
//! This crate keeps domain reducers independent from GTK and exposes system
//! adapters as explicit modules.

pub mod niri;
pub mod niri_actions;
pub mod niri_outputs;
pub mod niri_process;

pub mod niri_session;

#[cfg(feature = "gdk4-observer")]
pub mod application_runtime;
#[cfg(feature = "gdk4-observer")]
pub mod gdk_monitor_observer;
