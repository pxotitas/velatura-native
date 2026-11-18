//! Lightweight system-level samples used to surface real volume/brightness OSDs.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlLevels {
    pub volume: u8,
    pub muted: bool,
    pub brightness: u8,
}

/// Reads only the two values that drive the original shell's OSD. This is
/// intentionally separate from the full control-panel adapters.
pub fn read() -> Option<ControlLevels> {
    let audio = std::process::Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .ok()?;
    if !audio.status.success() {
        return None;
    }
    let audio = String::from_utf8_lossy(&audio.stdout);
    let volume = audio.split_whitespace().nth(1)?.parse::<f32>().ok()?;
    let muted = audio.contains("[MUTED]");
    let line = std::process::Command::new("brightnessctl")
        .args(["-m"])
        .output()
        .ok()?;
    if !line.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&line.stdout);
    let fields: Vec<_> = text.trim().split(',').collect();
    let brightness = fields.get(3)?.trim_end_matches('%').parse::<u8>().ok()?;
    Some(ControlLevels {
        volume: (volume * 100.0).round().clamp(0.0, 100.0) as u8,
        muted,
        brightness,
    })
}
