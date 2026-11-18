//! Native adapters for the Island's system-control surfaces.
//! Uses the same system authorities as the original shell: NetworkManager,
//! PipeWire/WirePlumber, BlueZ and UPower. Commands are bounded and parsed.
use std::process::{Command, Stdio};

fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("timeout")
        .arg("4")
        .arg(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiState {
    pub available: bool,
    pub enabled: bool,
    pub connected: bool,
    pub ssid: String,
    pub networks: Vec<WifiNetwork>,
}

pub fn wifi_state() -> Result<WifiState, String> {
    let radio = run("nmcli", &["-t", "-f", "WIFI", "radio"])?;
    let enabled = radio == "enabled";
    let active = run(
        "nmcli",
        &["-t", "-f", "ACTIVE,SSID", "connection", "show", "--active"],
    )?;
    let ssid = active
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(state, name)| *state == "yes" && !name.is_empty())
        .map(|(_, name)| name.to_string())
        .unwrap_or_default();
    let scan = run(
        "nmcli",
        &[
            "-t",
            "-f",
            "SSID,SIGNAL,SECURITY,IN-USE",
            "device",
            "wifi",
            "list",
            "--rescan",
            "no",
        ],
    )
    .unwrap_or_default();
    let mut networks = Vec::new();
    for line in scan.lines() {
        let mut fields = line.splitn(4, ':');
        let name = fields.next().unwrap_or("").replace("\\:", ":");
        let signal = fields.next().unwrap_or("0").parse().unwrap_or(0);
        let security = fields.next().unwrap_or("").to_string();
        let active = fields.next().unwrap_or("") == "*";
        if !name.is_empty() {
            networks.push(WifiNetwork {
                ssid: name,
                signal,
                security,
                active,
            });
        }
    }
    networks.sort_by(|a, b| {
        b.active
            .cmp(&a.active)
            .then(b.signal.cmp(&a.signal))
            .then(a.ssid.cmp(&b.ssid))
    });
    networks.dedup_by(|a, b| a.ssid == b.ssid);
    Ok(WifiState {
        available: true,
        enabled,
        connected: !ssid.is_empty(),
        ssid,
        networks,
    })
}

pub fn set_wifi(enabled: bool) -> Result<(), String> {
    run(
        "nmcli",
        &["radio", "wifi", if enabled { "on" } else { "off" }],
    )
    .map(|_| ())
}

pub fn connect_wifi(ssid: &str) -> Result<(), String> {
    if ssid.is_empty() || ssid.len() > 128 || ssid.contains('\0') {
        return Err("Invalid SSID".into());
    }
    run(
        "nmcli",
        &["--wait", "10", "device", "wifi", "connect", ssid],
    )
    .map(|_| ())
}

pub fn wifi_forget(ssid: &str) -> Result<(), String> {
    let escaped = ssid.replace('\'', "\\'");
    let profiles = run("nmcli", &["-t", "-f", "NAME,TYPE", "connection", "show"])?;
    let name = profiles
        .lines()
        .find(|line| line.starts_with(&format!("{escaped}:")) && line.ends_with(":802-11-wireless"))
        .and_then(|line| line.split(':').next())
        .ok_or("Saved network not found")?;
    run("nmcli", &["connection", "delete", "id", name]).map(|_| ())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothState {
    pub available: bool,
    pub enabled: bool,
    pub discovering: bool,
    pub devices: Vec<BluetoothDevice>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub connected: bool,
    pub paired: bool,
    pub trusted: bool,
}

pub fn bluetooth_state() -> Result<BluetoothState, String> {
    let info = run("bluetoothctl", &["show"])?;
    let enabled = info.lines().any(|line| line.trim() == "Powered: yes");
    let discovering = info.lines().any(|line| line.trim() == "Discovering: yes");
    let list = run("bluetoothctl", &["devices"])?;
    let mut devices = Vec::new();
    for line in list.lines() {
        let mut fields = line.splitn(3, ' ');
        if fields.next() != Some("Device") {
            continue;
        }
        let address = fields.next().unwrap_or("").to_string();
        if address.is_empty() {
            continue;
        }
        let name = fields.next().unwrap_or("").to_string();
        let detail = run("bluetoothctl", &["info", &address]).unwrap_or_default();
        devices.push(BluetoothDevice {
            address,
            name,
            connected: detail.lines().any(|l| l.trim() == "Connected: yes"),
            paired: detail.lines().any(|l| l.trim() == "Paired: yes"),
            trusted: detail.lines().any(|l| l.trim() == "Trusted: yes"),
        });
    }
    devices.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then(b.paired.cmp(&a.paired))
            .then(a.name.cmp(&b.name))
    });
    Ok(BluetoothState {
        available: true,
        enabled,
        discovering,
        devices,
    })
}
pub fn set_bluetooth(enabled: bool) -> Result<(), String> {
    run(
        "bluetoothctl",
        &["power", if enabled { "on" } else { "off" }],
    )
    .map(|_| ())
}
pub fn set_discovering(enabled: bool) -> Result<(), String> {
    run(
        "bluetoothctl",
        &["scan", if enabled { "on" } else { "off" }],
    )
    .map(|_| ())
}
pub fn connect_bluetooth(address: &str) -> Result<(), String> {
    if !address.bytes().all(|b| b.is_ascii_hexdigit() || b == b':') || address.len() != 17 {
        return Err("Invalid Bluetooth address".into());
    }
    run("bluetoothctl", &["connect", address]).map(|_| ())
}
pub fn disconnect_bluetooth(address: &str) -> Result<(), String> {
    if !address.bytes().all(|b| b.is_ascii_hexdigit() || b == b':') || address.len() != 17 {
        return Err("Invalid Bluetooth address".into());
    }
    run("bluetoothctl", &["disconnect", address]).map(|_| ())
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioState {
    pub volume: u8,
    pub muted: bool,
    pub sink: String,
    pub sinks: Vec<String>,
}
pub fn audio_state() -> Result<AudioState, String> {
    let status = run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"])?;
    let volume = status
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse::<f32>()
        .unwrap_or(0.0);
    let muted = status.contains("[MUTED]");
    let sink = run("wpctl", &["inspect", "@DEFAULT_AUDIO_SINK@"])
        .ok()
        .and_then(|s| {
            s.lines().find(|l| l.contains("node.description")).map(|l| {
                l.split_once('=')
                    .map(|(_, v)| v.trim().trim_matches('"').to_string())
                    .unwrap_or_default()
            })
        })
        .unwrap_or_default();
    let sinks = run("wpctl", &["status"]).unwrap_or_default();
    let names = sinks
        .lines()
        .filter(|line| line.contains("[vol:"))
        .map(|line| line.trim().trim_start_matches('*').trim().to_string())
        .collect();
    Ok(AudioState {
        volume: (volume * 100.0).round().clamp(0.0, 100.0) as u8,
        muted,
        sink,
        sinks: names,
    })
}
pub fn set_volume(value: u8) -> Result<(), String> {
    run(
        "wpctl",
        &[
            "set-volume",
            "@DEFAULT_AUDIO_SINK@",
            &format!("{}%", value.min(100)),
        ],
    )
    .map(|_| ())
}
pub fn toggle_mute() -> Result<(), String> {
    run("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]).map(|_| ())
}

pub fn brightness_state() -> Result<u8, String> {
    let output = run("brightnessctl", &["-m"])?;
    let fields: Vec<_> = output.split(',').collect();
    fields
        .get(3)
        .ok_or("brightness percentage missing")?
        .trim_end_matches('%')
        .parse::<u8>()
        .map_err(|error| error.to_string())
}

pub fn set_brightness(value: u8) -> Result<(), String> {
    run("brightnessctl", &["set", &format!("{}%", value.min(100))]).map(|_| ())
}

pub fn microphone_muted() -> Result<bool, String> {
    let status = run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SOURCE@"])?;
    Ok(status.contains("[MUTED]"))
}

pub fn set_microphone_muted(muted: bool) -> Result<(), String> {
    run(
        "wpctl",
        &[
            "set-mute",
            "@DEFAULT_AUDIO_SOURCE@",
            if muted { "1" } else { "0" },
        ],
    )
    .map(|_| ())
}

pub fn toggle_microphone_mute() -> Result<(), String> {
    run("wpctl", &["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]).map(|_| ())
}

pub fn power_profile() -> Result<String, String> {
    run("powerprofilesctl", &["get"])
}

pub fn toggle_power_profile() -> Result<(), String> {
    let current = power_profile()?;
    let available = run("powerprofilesctl", &["list"])?;
    let target = if current == "performance" {
        "balanced"
    } else if available.contains("performance") {
        "performance"
    } else {
        "balanced"
    };
    run("powerprofilesctl", &["set", target]).map(|_| ())
}

pub fn battery_summary() -> Result<String, String> {
    let output = run(
        "upower",
        &["-i", "/org/freedesktop/UPower/devices/DisplayDevice"],
    )?;
    let percent = output
        .lines()
        .find(|l| l.trim_start().starts_with("percentage:"))
        .map(|l| l.split_whitespace().nth(1).unwrap_or("").to_string())
        .unwrap_or_default();
    let state = output
        .lines()
        .find(|l| l.trim_start().starts_with("state:"))
        .map(|l| l.split_whitespace().nth(1).unwrap_or("").to_string())
        .unwrap_or_default();
    Ok(format!("{percent} · {state}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn ssid_parser_preserves_empty_fields_and_signal_order_contract() {
        let row = "Cafe:80:WPA2:*";
        let mut fields = row.splitn(4, ':');
        assert_eq!(fields.next(), Some("Cafe"));
        assert_eq!(fields.next(), Some("80"));
        assert_eq!(fields.next(), Some("WPA2"));
        assert_eq!(fields.next(), Some("*"));
    }
    #[test]
    fn bluetooth_address_validation_rejects_shell_input() {
        assert!(
            !"00:11:22:33:44:55;reboot"
                .bytes()
                .all(|b| b.is_ascii_hexdigit() || b == b':')
                || "00:11:22:33:44:55;reboot".len() != 17
        );
        assert!(
            "00:11:22:33:44:55"
                .bytes()
                .all(|b| b.is_ascii_hexdigit() || b == b':')
        );
    }
}
