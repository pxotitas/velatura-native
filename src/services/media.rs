//! MPRIS media snapshot and transport controls through playerctl.
//!
//! The legacy shell reads the same desktop MPRIS players. Keeping this adapter
//! small makes it safe to poll from the GTK main loop without owning a DBus
//! connection in each widget.

use std::process::{Command, Stdio};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaSnapshot {
    pub title: String,
    pub artist: String,
    pub art_url: String,
    pub status: String,
    pub length_us: u64,
    pub position_s: f64,
}

impl MediaSnapshot {
    pub fn playing(&self) -> bool {
        self.status.eq_ignore_ascii_case("playing")
    }

    pub fn identity(&self) -> &str {
        if self.title.is_empty() {
            &self.artist
        } else {
            &self.title
        }
    }
}

fn playerctl(args: &[&str]) -> Option<String> {
    let output = Command::new("playerctl")
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn snapshot() -> Option<MediaSnapshot> {
    let metadata = playerctl(&[
        "metadata",
        "--format",
        "{{title}}\t{{artist}}\t{{mpris:artUrl}}\t{{mpris:length}}",
    ])?;
    let mut fields = metadata.splitn(4, '\t');
    let title = fields.next().unwrap_or_default().to_string();
    let artist = fields.next().unwrap_or_default().to_string();
    let art_url = fields.next().unwrap_or_default().to_string();
    let length_us = fields
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let status = playerctl(&["status"]).unwrap_or_default();
    let position_s = playerctl(&["position"])
        .and_then(|value| value.parse().ok())
        .unwrap_or(0.0);
    if title.is_empty() && artist.is_empty() {
        return None;
    }
    Some(MediaSnapshot {
        title,
        artist,
        art_url,
        status,
        length_us,
        position_s,
    })
}

pub fn previous() {
    let _ = playerctl(&["previous"]);
}
pub fn next() {
    let _ = playerctl(&["next"]);
}
pub fn toggle() {
    let _ = playerctl(&["play-pause"]);
}
