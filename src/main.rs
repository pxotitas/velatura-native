//! GTK4 application entry point and native CLI.

use gtk4::Application;
use gtk4::gio;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::env;
use std::process::{Command, ExitCode, Stdio};
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use velatura_core::application_runtime::ApplicationRuntime;
use velatura_core::niri_session::NiriSessionEvent;
use velatura_core::ui::gtk_event_bridge::GtkEventBridge;

const APPLICATION_ID: &str = "io.github.pxdritz.velatura";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None | Some("run") => run_application(),
        Some("help" | "--help" | "-h") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("version" | "--version" | "-V") => {
            println!("Velatura {VERSION}");
            ExitCode::SUCCESS
        }
        Some("stop") => stop_application(),
        Some("restart") => restart_application(),
        Some("ipc") => unavailable_command("ipc", args.collect::<Vec<_>>().join(" ")),
        Some(command) if is_product_command(command) => {
            unavailable_command(command, args.collect::<Vec<_>>().join(" "))
        }
        Some(command) => {
            eprintln!("{{\"error\":\"comando desconhecido: {command}; use 'velatura help'\"}}");
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "Velatura {VERSION}\n\
         uso: velatura <comando> [argumentos]\n\n\
         aplicação:\n\
           run | (sem argumento)  iniciar a aplicação GTK4\n\
           stop                   encerrar a instância ativa\n\
           restart                reiniciar a aplicação\n\
           version                mostrar versão\n\
           help                   mostrar esta ajuda\n\n\
         comandos de domínio (contrato legado preservado):\n\
           status                 snapshot geral\n\
           timer start|pause|resume|cancel\n\
           note add|list|get|append|archive|remove\n\
           task add|list|move|done|due|remove\n\
           open <panel> | close | panels\n\
           vm list|start|open|pause|resume|stop|kill\n\
           media [play|pause|toggle|next|prev]\n\
           capture screen|text|region|window\n\
           watch [--interval segundos]\n\
           game | layout | focus | nightlight\n\
           bar | widgets\n\
           volume [set|mute] | brightness [set|step]\n\
           theme list|set | wallpaper set|list|random\n\
           say <texto> | lock | reload\n\
           ipc call <target> <verb> [argumentos...]\n\
           complete [palavras...]\n\n\
         O catálogo está disponível, mas os comandos de domínio/IPC ainda\n\
         precisam ser conectados aos serviços nativos. Nenhum deles chama\n\
         nem encaminha comandos ao Quickshell legado."
    );
}

fn is_product_command(command: &str) -> bool {
    matches!(
        command,
        "status"
            | "timer"
            | "note"
            | "task"
            | "open"
            | "close"
            | "panels"
            | "vm"
            | "media"
            | "capture"
            | "watch"
            | "game"
            | "layout"
            | "focus"
            | "nightlight"
            | "bar"
            | "widgets"
            | "volume"
            | "brightness"
            | "theme"
            | "wallpaper"
            | "say"
            | "lock"
            | "reload"
            | "complete"
    )
}

fn unavailable_command(command: &str, arguments: String) -> ExitCode {
    eprintln!(
        "{{\"error\":\"comando '{command}' catalogado, mas ainda não conectado ao backend nativo; argumentos: {}\"}}",
        json_escape(&arguments)
    );
    ExitCode::from(2)
}

fn run_application() -> ExitCode {
    let application = Application::builder()
        .application_id(APPLICATION_ID)
        .flags(gio::ApplicationFlags::default())
        .build();

    let quit_action = gio::SimpleAction::new("quit", None);
    let app_for_quit = application.downgrade();
    quit_action.connect_activate(move |_, _| {
        if let Some(app) = app_for_quit.upgrade() {
            app.quit();
        }
    });
    application.add_action(&quit_action);

    let runtime: Rc<RefCell<Option<ApplicationRuntime>>> = Rc::default();
    let runtime_for_activate = Rc::clone(&runtime);
    let event_bridge: Rc<RefCell<Option<GtkEventBridge>>> = Rc::default();
    let bridge_for_activate = Rc::clone(&event_bridge);

    application.connect_activate(move |app| {
        if let Some(window) = app.windows().first() {
            window.present();
            return;
        }

        let startup = if runtime_for_activate.borrow().is_none() {
            match ApplicationRuntime::start() {
                Ok(started) => {
                    *runtime_for_activate.borrow_mut() = Some(started);
                    Ok(())
                }
                Err(error) => Err(error.to_string()),
            }
        } else {
            Ok(())
        };

        if let Some(display) = gtk4::gdk::Display::default() {
            let _provider = velatura_core::ui::theme::install(
                &display,
                &velatura_core::ui::theme::VisualTheme::default(),
            );
        }

        let window = match startup {
            Ok(()) => {
                let runtime_ref = runtime_for_activate.borrow();
                let backend = runtime_ref
                    .as_ref()
                    .expect("runtime must exist after successful startup");
                velatura_core::ui::bootstrap::build_bootstrap_window_with_snapshot(
                    app,
                    backend.monitor_count(),
                    backend.session().refresh_handle(),
                    &backend.snapshot(),
                )
            }
            Err(error) => velatura_core::ui::bootstrap::build_error_window(app, &error),
        };
        window.present();

        if bridge_for_activate.borrow().is_none() {
            let receiver = runtime_for_activate
                .borrow_mut()
                .as_mut()
                .and_then(ApplicationRuntime::take_events);
            if let Some(receiver) = receiver {
                let weak_window = window.downgrade();
                *bridge_for_activate.borrow_mut() = Some(GtkEventBridge::attach(
                    receiver,
                    gtk4::glib::MainContext::default(),
                    move |event| {
                        if let NiriSessionEvent::SnapshotChanged(snapshot) = event
                            && let Some(window) = weak_window.upgrade()
                        {
                            window.set_title(Some(
                                &velatura_core::ui::bootstrap::backend_window_title(&snapshot),
                            ));
                        }
                    },
                ));
            }
        }
    });

    let runtime_for_shutdown = Rc::clone(&runtime);
    let bridge_for_shutdown = Rc::clone(&event_bridge);
    application.connect_shutdown(move |_| {
        if let Some(mut backend) = runtime_for_shutdown.borrow_mut().take() {
            backend.shutdown();
        }
        if let Some(mut bridge) = bridge_for_shutdown.borrow_mut().take() {
            bridge.join();
        }
    });

    // `run` is our own CLI subcommand, not a file to be opened by GApplication.
    // Pass only argv[0] to GTK so positional CLI arguments are not re-parsed as files.
    let program_name = env::args().next().unwrap_or_else(|| "velatura".to_string());
    application.run_with_args(&[program_name]).into()
}

fn stop_application() -> ExitCode {
    match Command::new("gapplication")
        .args(["action", APPLICATION_ID, "quit"])
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("{{\"ok\":true,\"command\":\"stop\"}}");
            ExitCode::SUCCESS
        }
        Ok(output) => {
            let detail = if output.stderr.is_empty() {
                String::from_utf8_lossy(&output.stdout)
            } else {
                String::from_utf8_lossy(&output.stderr)
            };
            eprintln!(
                "{{\"error\":\"não foi possível encerrar a aplicação: {}\"}}",
                json_escape(detail.trim())
            );
            ExitCode::from(1)
        }
        Err(error) => {
            eprintln!(
                "{{\"error\":\"não foi possível executar gapplication: {}\"}}",
                json_escape(&error.to_string())
            );
            ExitCode::from(1)
        }
    }
}

fn restart_application() -> ExitCode {
    let stopped = Command::new("gapplication")
        .args(["action", APPLICATION_ID, "quit"])
        .output();

    if !matches!(stopped, Ok(ref output) if output.status.success()) {
        eprintln!("{{\"error\":\"não há uma instância ativa ou não foi possível encerrá-la\"}}");
        return ExitCode::from(1);
    }

    thread::sleep(Duration::from_millis(350));
    match env::current_exe().and_then(|executable| {
        Command::new(executable)
            .arg("run")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    }) {
        Ok(_) => {
            println!("{{\"ok\":true,\"command\":\"restart\"}}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "{{\"error\":\"não foi possível iniciar novamente: {}\"}}",
                json_escape(&error.to_string())
            );
            ExitCode::from(1)
        }
    }
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_escape_produces_valid_string_contents() {
        assert_eq!(json_escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
    }
}
