# Quickshell — visual reference captures

**Source of truth for the native GTK port.** These 16 PNG screenshots were supplied by the maintainer on 2026-11-18 and copied verbatim from `~/Pictures/Screenshots`. They are committed as reference material; do not modify or recompress the originals.

The captures show the actual Quickshell shell in use, including the top bar, resting/active Island, the media-playing state, control centre, notifications, and other visible surfaces. `CONTACT-SHEET.png` is a derived overview; the timestamped PNGs remain the authoritative images.

## Critical Island interpretation

The central capsule in the media-playing capture is expanded because media is playing. That is an activity state, not the default idle appearance. The original implementation distinguishes:

- `IslandRest.qml`: the compact resting face. The clock remains central; a running activity may occupy the sides, and a single media activity is split into artwork and spectrum.
- `IslandSummary.qml`: the hover glance. The face changes with the selected activity; the media face has artwork, track metadata and transport controls, alongside a large clock/date. Without a face, it shows the five-day calendar row.
- OSD and notification layers: separate transient/notification states, with their own geometry and arbitration.
- Control centre: a user-arranged grid of blocks, plus a top row of shortcuts. It is not a generic quick-settings dialog.

## Fidelity requirements

- Reproduce the original Quickshell implementation, not a redesign, interpretation, or approximation.
- Match geometry, spacing, typography, colours, corner radii, transitions, hover behaviour, hit targets, and responsive sizing.
- Preserve state priority and transitions between rest, summary, OSD, notifications, and expanded panels.
- Use the original QML/services as behavioural evidence. Screenshots are visual evidence; neither source should be treated as a substitute for the other.
- Never invent a control, state, animation, or layout that is not supported by the original source or these references.
- Do not mark a surface VERIFIED until it has been compared against these screenshots on the actual Niri session.

## Original source areas

The local original checkout is `~/.config/quickshell`. Particularly relevant files:

- `bar/island/IslandRest.qml`
- `bar/island/IslandSummary.qml`
- `bar/island/OsdLayer.qml`
- `bar/island/ControlsPanel.qml` and `bar/island/controls/*`
- `bar/widgets/DynamicIsland.qml`
- `services/ModuleService.qml`
- `services/MediaService.qml`, `CavaService.qml`, `TimerService.qml`, `RecorderService.qml`, `PrivacyService.qml`, `VmService.qml`, `NiriService.qml`, `LyricsService.qml`

Local paths above are explanatory only. The timestamped screenshots in this directory travel with the repository and are the portable visual reference.
