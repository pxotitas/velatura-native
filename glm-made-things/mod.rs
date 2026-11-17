//! GTK4 presentation layer.
//!
//! Fresh port of the Quickshell/QML UI on GTK4: the shared visual tokens, the
//! layer-shell surface setup, the morphing island and the composed shell.
//! Widgets never touch system APIs; they consume the services through the
//! typed handles in `shell::ServiceHandles`.

pub mod island;
pub mod launcher;
pub mod layer;
pub mod morph;
pub mod shell;
pub mod theme;
