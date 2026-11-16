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
            println!("Velatura Native {VERSION}");
            ExitCode::SUCCESS
        }
        Some("stop") => stop_application(),
        Some("restart") => restart_application(),
        Some("ipc") => {
            eprintln!(
                "{{\"error\":\"IPC nativo ainda não foi implementado; nenhum comando foi enviado ao shell legado.\"}}"
            );
            ExitCode::from(2)
        }
        Some(command) => {
            eprintln!(
                "{{\"error\":\"comando desconhecido: {command}; use 'velatura-native help'\"}}"
            );
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "Velatura Native {VERSION}\n\
         uso: velatura-native <comando>\n\n\
         comandos disponíveis:\n\
           run       inicia a aplicação GTK4 (padrão)\n\
           stop      solicita o encerramento da instância ativa\n\
           restart   encerra e inicia novamente a aplicação\n\
           version   mostra a versão\n\
           help      mostra esta ajuda\n\n\
         ainda não implementados:\n\
           ipc       será conectado à API tipada do backend; não encaminha\n\
                     chamadas ao Quickshell legado.\n\n\
         executável de desenvolvimento: velatura-native\n\
         o comando legado 'velatura' permanece intacto durante a migração."
    );
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

        let window = match startup {
            Ok(()) => {
                let runtime_ref = runtime_for_activate.borrow();
                let backend = runtime_ref
                    .as_ref()
                    .expect("runtime must exist after successful startup");
                velatura_core::ui::bootstrap::build_bootstrap_window(
                    app,
                    backend.monitor_count(),
                    backend.session().refresh_handle(),
                )
            }
            Err(error) => velatura_core::ui::bootstrap::build_error_window(app, &error),
        };
        window.present();
    });

    let runtime_for_shutdown = Rc::clone(&runtime);
    application.connect_shutdown(move |_| {
        if let Some(mut backend) = runtime_for_shutdown.borrow_mut().take() {
            backend.shutdown();
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
