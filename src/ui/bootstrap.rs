//! Minimal surface used to verify GTK activation and backend ownership.

use crate::niri_session::NiriRefreshHandle;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation};

/// Build the temporary test window. Replace its contents with the real Velatura
/// UI when the frontend is ready; do not move application lifecycle here.
pub fn build_bootstrap_window(
    app: &Application,
    monitor_count: u32,
    refresh: NiriRefreshHandle,
) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Velatura Native — bootstrap")
        .default_width(520)
        .default_height(300)
        .build();

    let root = GtkBox::new(Orientation::Vertical, 14);
    root.set_margin_top(24);
    root.set_margin_bottom(24);
    root.set_margin_start(24);
    root.set_margin_end(24);

    let title = Label::new(Some("Velatura Native"));
    title.add_css_class("title-1");
    title.set_xalign(0.0);

    let subtitle = Label::new(Some("GTK4 inicializado; backend Niri ativo."));
    subtitle.set_xalign(0.0);

    let monitor_status = Label::new(Some(&format!(
        "Monitores detectados pelo GDK: {monitor_count}"
    )));
    monitor_status.set_xalign(0.0);

    let action_status = Label::new(Some(
        "Superfície temporária de integração. A interface 1:1 ainda será portada.",
    ));
    action_status.set_xalign(0.0);
    action_status.set_wrap(true);

    let refresh_button = Button::with_label("Solicitar atualização dos monitores");
    let action_status_for_click = action_status.clone();
    refresh_button.connect_clicked(move |_| {
        let message = match refresh.refresh_outputs() {
            Ok(()) => "Solicitação de atualização enviada ao backend.".to_string(),
            Err(error) => format!("Não foi possível solicitar atualização: {error}"),
        };
        action_status_for_click.set_text(&message);
    });

    root.append(&title);
    root.append(&subtitle);
    root.append(&monitor_status);
    root.append(&refresh_button);
    root.append(&action_status);
    window.set_child(Some(&root));
    window
}

/// Show an explicit startup error instead of silently running without backend.
pub fn build_error_window(app: &Application, error: &str) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Velatura Native — erro de inicialização")
        .default_width(520)
        .default_height(220)
        .build();

    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_top(24);
    root.set_margin_bottom(24);
    root.set_margin_start(24);
    root.set_margin_end(24);

    let title = Label::new(Some("Não foi possível iniciar o backend"));
    title.add_css_class("title-2");
    title.set_xalign(0.0);

    let details = Label::new(Some(error));
    details.set_xalign(0.0);
    details.set_wrap(true);

    root.append(&title);
    root.append(&details);
    window.set_child(Some(&root));
    window
}
