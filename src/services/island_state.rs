//! Dynamic Island arbitration.
//!
//! Direct port of `bar/island/IslandState.qml`. Layers are ranked and the
//! island shows the highest one that currently wants the screen:
//!
//! ```text
//! panel         opened by the user       until dismissed
//! notification  something arrived        until it expires or is closed
//! osd           volume, brightness…      shown, then expires
//! summary       the pointer rests on it  until it leaves
//! modules       nothing happening        always available
//! ```
//!
//! A notification outranks a transient, which only confirms something the
//! user just did. The summary sits just above rest because it only restates
//! what was already true. Keeping this logic free of GTK makes the exact
//! arbitration rules unit-testable against the original's behaviour.

/// How long a transient event holds the island before it falls away.
pub const TRANSIENT_DURATION_MS: u64 = 1800;

/// The five island layers, ranked by `layer()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Modules,
    Summary,
    Osd,
    Notification,
    Panel,
}

/// A transient OSD event as delivered by `OsdService.requested`.
#[derive(Debug, Clone, PartialEq)]
pub struct TransientEvent {
    pub icon: String,
    pub label: String,
    /// A bar is drawn when this is 0 or more; -1 for a message without a
    /// level, matching the original's `progress: real`.
    pub progress: f32,
}

impl Default for TransientEvent {
    fn default() -> Self {
        Self {
            icon: String::new(),
            label: String::new(),
            progress: -1.0,
        }
    }
}

/// State of the outside world the arbitration depends on.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct World {
    /// Whether a notification is currently on the island
    /// (`NotificationService.active`).
    pub notification_active: bool,
    /// Whether the current notification is urgent and waits for the user.
    pub notification_critical: bool,
}

/// Arbitration state machine. One per screen; only the live one arbitrates.
#[derive(Debug, Clone, Default)]
pub struct IslandState {
    /// Whether this island is the one being worked on (`Bar.live`).
    pub active: bool,
    transient: Option<TransientEvent>,
    /// Wall-clock instant when the transient expires. Kept in the same
    /// timebase the owner feeds back through `tick`.
    transient_expiry: Option<u64>,
    /// The panel the user opened; `""`/None means none.
    pub open_panel: Option<String>,
    /// Whether the pointer has rested on the island long enough.
    pub summary: bool,
}

impl IslandState {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            ..Self::default()
        }
    }

    /// The resolved layer, in the original's priority order.
    pub fn layer(&self, world: World) -> Layer {
        if !self.active {
            return Layer::Modules;
        }
        if self.open_panel.is_some() {
            return Layer::Panel;
        }
        if world.notification_active {
            return Layer::Notification;
        }
        if self.transient.is_some() {
            return Layer::Osd;
        }
        if self.summary {
            return Layer::Summary;
        }
        Layer::Modules
    }

    pub fn expanded(&self) -> bool {
        self.active && self.open_panel.is_some()
    }

    /// The visible transient, if it has not expired yet.
    pub fn transient(&self) -> Option<&TransientEvent> {
        self.transient.as_ref()
    }

    /// An OSD event arriving while a panel is open is dropped rather than
    /// queued: by the time the panel closes the value would be stale, and
    /// showing it then would look like a ghost.
    pub fn request_osd(&mut self, now_ms: u64, event: TransientEvent) {
        if !self.active || self.open_panel.is_some() {
            return;
        }
        self.transient = Some(event);
        self.transient_expiry = Some(now_ms + TRANSIENT_DURATION_MS);
    }

    /// Expire the transient once its clock runs out. Returns whether the
    /// visible state changed.
    pub fn tick(&mut self, now_ms: u64) -> bool {
        if let Some(expiry) = self.transient_expiry
            && now_ms >= expiry
        {
            self.transient = None;
            self.transient_expiry = None;
            return true;
        }
        false
    }

    /// A panel takes over immediately; whatever was flashing is now noise.
    /// The notification goes too, without being marked as acted on: opening a
    /// a panel is not the same as answering what was on screen. Returns
    /// `true` when the caller must tell the notification service to dismiss
    /// its current notification (the original calls `NotificationService
    /// .dismiss()` here).
    pub fn open(&mut self, panel: &str, world: World) -> bool {
        if !self.active {
            return false;
        }
        let dismiss_notification = world.notification_active;
        self.transient = None;
        self.transient_expiry = None;
        self.summary = false;
        self.open_panel = Some(panel.to_string());
        dismiss_notification
    }

    pub fn close(&mut self) {
        self.open_panel = None;
    }

    /// Handing the island to another screen puts this one back to rest at
    /// once: the layer is already forced above, and what is left is the state
    /// behind it, which would otherwise come back when the island returned.
    pub fn set_active(&mut self, active: bool) -> bool {
        if self.active == active {
            return false;
        }
        self.active = active;
        if active {
            return false;
        }
        self.open_panel = None;
        self.summary = false;
        self.transient = None;
        self.transient_expiry = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn osd() -> TransientEvent {
        TransientEvent {
            icon: "speaker".into(),
            label: "42%".into(),
            progress: 0.42,
        }
    }

    #[test]
    fn rest_shows_modules_without_any_event() {
        let state = IslandState::new(true);
        assert_eq!(state.layer(World::default()), Layer::Modules);
    }

    #[test]
    fn notification_outranks_transient() {
        let mut state = IslandState::new(true);
        state.request_osd(0, osd());
        let world = World {
            notification_active: true,
            notification_critical: false,
        };
        assert_eq!(state.layer(world), Layer::Notification);
        // And it comes back to the OSD while the transient is still alive.
        assert_eq!(state.layer(World::default()), Layer::Osd);
    }

    #[test]
    fn panel_is_topmost() {
        let mut state = IslandState::new(true);
        state.request_osd(0, osd());
        state.open("controls", World::default());
        let world = World {
            notification_active: true,
            notification_critical: false,
        };
        assert_eq!(state.layer(world), Layer::Panel);
    }

    #[test]
    fn transient_expires_after_1800ms() {
        let mut state = IslandState::new(true);
        state.request_osd(1000, osd());
        assert!(!state.tick(2799));
        assert_eq!(state.layer(World::default()), Layer::Osd);
        assert!(state.tick(2800));
        assert_eq!(state.layer(World::default()), Layer::Modules);
    }

    #[test]
    fn osd_dropped_while_panel_open() {
        let mut state = IslandState::new(true);
        state.open("launcher", World::default());
        state.request_osd(0, osd());
        assert!(state.transient().is_none());
        assert_eq!(state.layer(World::default()), Layer::Panel);
    }

    #[test]
    fn open_cancels_transient_and_summary_and_reports_notification() {
        let mut state = IslandState::new(true);
        state.request_osd(0, osd());
        state.summary = true;
        let world = World {
            notification_active: true,
            notification_critical: false,
        };
        assert!(state.open("controls", world));
        assert!(state.transient().is_none());
        assert!(!state.summary);
        assert!(state.expanded());
    }

    #[test]
    fn opening_without_notification_does_not_request_dismissal() {
        let mut state = IslandState::new(true);
        assert!(!state.open("controls", World::default()));
    }

    #[test]
    fn losing_live_resets_everything() {
        let mut state = IslandState::new(true);
        state.request_osd(0, osd());
        state.summary = true;
        state.open("controls", World::default());
        assert!(state.set_active(false));
        assert_eq!(state.layer(World::default()), Layer::Modules);
        assert!(state.open_panel.is_none());
        assert!(state.transient().is_none());
        assert!(!state.summary);
    }

    #[test]
    fn inactive_island_never_arbitrates() {
        let mut state = IslandState::new(false);
        state.request_osd(0, osd());
        state.summary = true;
        let world = World {
            notification_active: true,
            notification_critical: false,
        };
        assert_eq!(state.layer(world), Layer::Modules);
    }
}
