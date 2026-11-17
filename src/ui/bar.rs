//! One bar surface per monitor (`Bar.qml`). Full-screen, transparent, never
//! resized: the island morphs inside it and the input region follows.

use gtk4::gdk;
use gtk4::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

use super::fillet::Fillet;
use super::island::Island;
use super::settings::Settings;
use super::zone::Zone;
use crate::niri_session::{NiriSession, NiriSessionSnapshot};
use crate::services::island_state::Layer;
use crate::services::notifications::NotificationService;
use super::{surface, tokens};

pub struct Bar {
    pub window: gtk4::ApplicationWindow,
    pub connector: String,
    island: Rc<Island>,
    zones: [Option<Rc<Zone>>; 2],
}

impl Bar {
    pub fn new(
        app: &gtk4::Application,
        monitor: &gdk::Monitor,
        settings: Rc<Settings>,
        session: Arc<NiriSession>,
        open_panel: Rc<dyn Fn(&str)>,
        notifications: Rc<NotificationService>,
    ) -> Self {
        let window = gtk4::ApplicationWindow::new(app);
        surface::configure(&window, monitor);

        let stage = gtk4::Fixed::new();
        window.set_child(Some(&stage));

        let island = Island::new(Rc::clone(&settings), true, notifications, Arc::clone(&session));
        let fillets = [Fillet::new(true), Fillet::new(false)];
        stage.put(&island.capsule, 0.0, 0.0);
        for fillet in &fillets {
            stage.put(fillet, 0.0, 0.0);
            fillet.set_visible(settings.notched());
        }

        let output = monitor.connector().map(|name| name.to_string()).unwrap_or_default();
        let zones: [Option<Rc<Zone>>; 2] = ["left", "right"].map(|side| {
            Zone::new(&settings, side, &output, Arc::clone(&session), Rc::clone(&open_panel)).map(|zone| {
                stage.put(zone.widget(), 0.0, 0.0);
                Rc::new(zone)
            })
        });

        let screen_width = monitor.geometry().width();
        let place = {
            let (stage, island, window) = (stage.clone(), Rc::clone(&island), window.clone());
            let settings = Rc::clone(&settings);
            let zones = zones.clone();
            move || {
                let (width, height) = island.size();
                let left = (screen_width - width) / 2;
                let top = if settings.notched() { 0 } else { settings.bar_margin };
                stage.move_(&island.capsule, f64::from(left), f64::from(top));

                let color = island.surface_color().gdk();
                let side = tokens::RADIUS_NOTCH * 2;
                stage.move_(&fillets[0], f64::from(left - side), 0.0);
                stage.move_(&fillets[1], f64::from(left + width), 0.0);
                for fillet in &fillets {
                    fillet.set_color(color);
                }

                // Grouped: the sides hug the island and give way when it takes
                // the band; the lane is centred on the island's rest line.
                let lane = if settings.notched() { settings.bar_margin / 2 } else { settings.bar_margin };
                let away = island.layer() != Layer::Modules;
                if let Some(zone) = &zones[0] {
                    let x = left - tokens::CAPSULE_SPACING - zone.width();
                    stage.move_(zone.widget(), f64::from(x), f64::from(lane));
                    zone.widget().set_visible(!away);
                }
                if let Some(zone) = &zones[1] {
                    let x = left + width + tokens::CAPSULE_SPACING;
                    stage.move_(zone.widget(), f64::from(x), f64::from(lane));
                    zone.widget().set_visible(!away);
                }

                // The bar's band, plus the island's shape with 12 px under it
                // so its bottom edge still counts.
                let band = settings.bar_margin + settings.bar_height + 8;
                surface::set_input_rects(
                    &window,
                    &[(0, 0, screen_width, band), (left, 0, width, top + height + 12)],
                );
            }
        };
        island.connect_changed(place.clone());
        window.connect_realize({
            let place = place.clone();
            move |_| place()
        });
        window.connect_map({
            let place = place.clone();
            move |_| place()
        });

        // The zones change width as the strips animate; the sides follow.
        let last = std::cell::Cell::new((0, 0));
        stage.add_tick_callback({
            let (zones, place) = (zones.clone(), place.clone());
            move |_, _| {
                let widths = (
                    zones[0].as_ref().map_or(0, |zone| zone.width()),
                    zones[1].as_ref().map_or(0, |zone| zone.width()),
                );
                if last.replace(widths) != widths {
                    place();
                }
                gtk4::glib::ControlFlow::Continue
            }
        });

        island.sync_layer();
        window.present();

        Self { window, connector: output, island, zones }
    }

    pub fn update(&self, snapshot: &NiriSessionSnapshot) {
        for zone in self.zones.iter().flatten() {
            zone.update(snapshot);
        }
    }

    pub fn island(&self) -> &Rc<Island> {
        &self.island
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        self.window.destroy();
    }
}

pub fn monitors() -> Vec<gdk::Monitor> {
    gdk::Display::default()
        .map(|display| {
            display
                .monitors()
                .iter::<gdk::Monitor>()
                .filter_map(Result::ok)
                .collect()
        })
        .unwrap_or_default()
}

pub fn watch_monitors(callback: impl Fn() + 'static) {
    if let Some(display) = gdk::Display::default() {
        display.monitors().connect_items_changed(move |_, _, _, _| callback());
    }
}
