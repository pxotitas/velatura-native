//! Island system detail surfaces: Wi-Fi, Bluetooth, Sound and Battery.
//! The adapters are kept in services::system_controls; this widget owns
//! presentation and only refreshes while its panel is visible.
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::services::system_controls as controls;

pub const WIDTH: i32 = 420;
pub const HEIGHT: i32 = 500;

pub struct SystemPanel {
    pub widget: gtk4::Box,
    page: String,
    status: gtk4::Label,
    content: gtk4::Box,
}
impl SystemPanel {
    pub fn new(page: &str, close: impl Fn() + 'static) -> Self {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(16);
        root.set_margin_end(16);
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let back = gtk4::Button::with_label("‹");
        back.add_css_class("flat");
        back.connect_clicked(move |_| close());
        header.append(&back);
        let title = gtk4::Label::new(Some(match page {
            "wifi" => "Wi-Fi",
            "bluetooth" => "Bluetooth",
            "sound" => "Sound",
            "battery" => "Battery",
            _ => "System controls",
        }));
        title.add_css_class("heading");
        title.set_xalign(0.0);
        title.set_hexpand(true);
        header.append(&title);
        root.append(&header);
        let status = gtk4::Label::new(None);
        status.set_xalign(0.0);
        status.add_css_class("dim-label");
        root.append(&status);
        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content.set_vexpand(true);
        root.append(&content);
        let panel = Self {
            widget: root,
            page: page.into(),
            status,
            content,
        };
        panel.refresh();
        panel
    }
    fn clear(&self) {
        while let Some(child) = self.content.first_child() {
            self.content.remove(&child);
        }
    }
    fn action_button(&self, label: &str, action: impl Fn() + 'static) {
        let button = gtk4::Button::with_label(label);
        button.set_halign(gtk4::Align::Fill);
        button.connect_clicked(move |_| action());
        self.content.append(&button);
    }
    pub fn page(&self) -> &str {
        &self.page
    }
    pub fn refresh(&self) {
        self.clear();
        match self.page.as_str() {
            "wifi" => self.wifi(),
            "bluetooth" => self.bluetooth(),
            "sound" => self.sound(),
            "battery" => self.battery(),
            _ => self.status.set_text("Unsupported system page"),
        }
    }
    fn wifi(&self) {
        match controls::wifi_state() {
            Err(error) => self
                .status
                .set_text(&format!("NetworkManager unavailable · {error}")),
            Ok(state) => {
                let summary = if state.connected {
                    format!("Connected · {}", state.ssid)
                } else if state.enabled {
                    "Not connected".into()
                } else {
                    "Wi-Fi is off".into()
                };
                self.status.set_text(&summary);
                let this = Rc::new(RefCell::new(()));
                let _keep = this;
                let enabled = state.enabled;
                self.action_button(
                    if enabled {
                        "Turn Wi-Fi off"
                    } else {
                        "Turn Wi-Fi on"
                    },
                    move || {
                        let _ = controls::set_wifi(!enabled);
                    },
                );
                let no_networks = state.networks.is_empty();
                for network in &state.networks {
                    let label = format!(
                        "{}    {}%{}",
                        network.ssid,
                        network.signal,
                        if network.security.is_empty() {
                            ""
                        } else {
                            " · secured"
                        }
                    );
                    let ssid = network.ssid.clone();
                    self.action_button(&label, move || {
                        let _ = controls::connect_wifi(&ssid);
                    });
                }
                if no_networks && state.enabled {
                    self.content.append(&gtk4::Label::new(Some(
                        "No networks found. Try refreshing from NetworkManager.",
                    )));
                }
            }
        }
    }
    fn bluetooth(&self) {
        match controls::bluetooth_state() {
            Err(error) => self
                .status
                .set_text(&format!("Bluetooth unavailable · {error}")),
            Ok(state) => {
                self.status.set_text(if !state.enabled {
                    "Adapter off"
                } else if state.discovering {
                    "Looking for devices…"
                } else {
                    "Nearby and paired devices"
                });
                let enabled = state.enabled;
                self.action_button(
                    if enabled {
                        "Turn Bluetooth off"
                    } else {
                        "Turn Bluetooth on"
                    },
                    move || {
                        let _ = controls::set_bluetooth(!enabled);
                    },
                );
                let discovering = state.discovering;
                self.action_button(
                    if discovering {
                        "Stop scanning"
                    } else {
                        "Scan for devices"
                    },
                    move || {
                        let _ = controls::set_discovering(!discovering);
                    },
                );
                for device in state.devices {
                    let label = format!(
                        "{}  ·  {}",
                        if device.name.is_empty() {
                            &device.address
                        } else {
                            &device.name
                        },
                        if device.connected {
                            "Connected"
                        } else if device.paired {
                            "Paired"
                        } else {
                            "Available"
                        }
                    );
                    let address = device.address.clone();
                    let connected = device.connected;
                    self.action_button(&label, move || {
                        let _ = if connected {
                            controls::disconnect_bluetooth(&address)
                        } else {
                            controls::connect_bluetooth(&address)
                        };
                    });
                }
            }
        }
    }
    fn sound(&self) {
        match controls::audio_state() {
            Err(error) => self
                .status
                .set_text(&format!("PipeWire unavailable · {error}")),
            Ok(state) => {
                self.status.set_text(&format!(
                    "{} · {}%",
                    if state.muted { "Muted" } else { "Output" },
                    state.volume
                ));
                let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
                let down = gtk4::Button::with_label("−");
                let value = gtk4::Label::new(Some(&format!("{}%", state.volume)));
                value.set_hexpand(true);
                let up = gtk4::Button::with_label("+");
                let v = state.volume;
                down.connect_clicked(move |_| {
                    let _ = controls::set_volume(v.saturating_sub(5));
                });
                up.connect_clicked(move |_| {
                    let _ = controls::set_volume(v.saturating_add(5).min(100));
                });
                row.append(&down);
                row.append(&value);
                row.append(&up);
                self.content.append(&row);
                self.action_button(if state.muted { "Unmute" } else { "Mute" }, || {
                    let _ = controls::toggle_mute();
                });
                for sink in state.sinks {
                    let label = sink;
                    let l = gtk4::Label::new(Some(&label));
                    l.set_xalign(0.0);
                    self.content.append(&l);
                }
            }
        }
    }
    fn battery(&self) {
        match controls::battery_summary() {
            Ok(summary) => {
                self.status.set_text(&summary);
                self.content.append(&gtk4::Label::new(Some(
                    "Battery information provided by UPower.",
                )));
            }
            Err(error) => self
                .status
                .set_text(&format!("Battery unavailable · {error}")),
        }
    }
}
