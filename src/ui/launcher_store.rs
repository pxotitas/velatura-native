//! What the launcher searches and what choosing a result does
//! (`LauncherService.qml`). Applications only so far: the other modes are
//! reported as not ported rather than answered with nothing.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::settings::Settings;
use crate::services::launcher::{self, Application, LaunchHistory};

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub kind: &'static str,
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub icon: String,
    /// Clipboard payload file, present for image entries.
    pub file: Option<String>,
    /// Kept on the dock, which is why it ranks near the top of an empty query.
    pub favourite: bool,
}

pub enum Outcome {
    Entries(Vec<Entry>),
    /// The mode's label: its backend does not exist yet.
    Unported(&'static str),
}

pub struct Store {
    apps: Vec<Application>,
    history: LaunchHistory,
    favourites: Vec<String>,
    by_use: bool,
    max: usize,
    history_path: PathBuf,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

impl Store {
    pub fn new(settings: &Settings) -> Self {
        Self {
            apps: Vec::new(),
            history: LaunchHistory::default(),
            favourites: settings.dock_pinned.clone(),
            by_use: settings.launcher_order != "alphabetical",
            max: settings.launcher_results.max(1),
            history_path: Settings::path().with_file_name("launcher-history.json"),
        }
    }

    /// Rescans the application directories and rereads the launch history.
    pub fn refresh(&mut self) {
        self.apps = launcher::list_applications();
        self.history = std::fs::read_to_string(&self.history_path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
    }

    pub fn search(&self, query: &str) -> Outcome {
        let mode = launcher::mode_for(query);
        if mode.id == "calculate" {
            let expression = launcher::term_for(query);
            return Outcome::Entries(
                launcher::evaluate(expression)
                    .map(|value| {
                        vec![Entry {
                            kind: "calculation",
                            id: value.to_string(),
                            name: value.to_string(),
                            subtitle: format!("{expression}  ·  Enter copies it"),
                            icon: "accessories-calculator".into(),
                            file: None,
                            favourite: false,
                        }]
                    })
                    .unwrap_or_default(),
            );
        }
        if mode.id == "desk" {
            let term = launcher::term_for(query).to_lowercase();
            let settings = Entry {
                kind: "settings",
                id: "settings".into(),
                name: "Settings".into(),
                subtitle: "Configure Velatura".into(),
                icon: "preferences-system".into(),
                file: None,
                favourite: false,
            };
            return Outcome::Entries(
                if term.is_empty() || "settings".contains(&term) || term.contains("settings") {
                    vec![settings]
                } else {
                    Vec::new()
                },
            );
        }
        if mode.id == "clipboard" {
            let history = crate::services::clipboard::entries();
            let entries =
                crate::services::clipboard::search(&history, launcher::term_for(query), self.max);
            return Outcome::Entries(
                entries
                    .into_iter()
                    .map(|entry| Entry {
                        kind: "clip",
                        id: entry.key.clone(),
                        name: crate::services::clipboard::title(entry),
                        subtitle: crate::services::clipboard::describe(entry),
                        icon: if entry.kind == "image" {
                            "image-x-generic".into()
                        } else {
                            "edit-paste".into()
                        },
                        file: if entry.kind == "image" {
                            Some(entry.file.clone())
                        } else {
                            None
                        },
                        favourite: false,
                    })
                    .collect(),
            );
        }
        if mode.id != "apps" {
            return Outcome::Unported(mode.label);
        }
        let ranked = launcher::rank(
            &self.apps,
            launcher::term_for(query),
            &self.history,
            &self.favourites,
            now_ms(),
            self.by_use,
            self.max,
        );
        Outcome::Entries(
            ranked
                .into_iter()
                .map(|app| Entry {
                    kind: "app",
                    favourite: self.favourites.contains(&app.id),
                    id: app.id,
                    name: app.name,
                    subtitle: app.subtitle,
                    icon: app.icon,
                    file: None,
                })
                .collect(),
        )
    }

    /// Recorded here and not in the launch itself, so dock launches do not
    /// count towards the ranking.
    pub fn launch(&mut self, entry: &Entry) {
        if entry.kind == "calculation" {
            if let Err(error) = std::process::Command::new("wl-copy")
                .arg("--")
                .arg(&entry.id)
                .spawn()
            {
                eprintln!("calculator copy failed: {error}");
            }
            return;
        }
        if entry.kind == "settings" {
            if let Err(error) = std::process::Command::new("gapplication")
                .args(["action", "io.github.pxdritz.velatura", "settings"])
                .spawn()
            {
                eprintln!("settings action failed: {error}");
            }
            return;
        }
        if entry.kind == "clip" {
            if let Err(error) = crate::services::clipboard::restore_key(&entry.id) {
                eprintln!("clipboard restore failed: {error:?}");
            }
            return;
        }
        if entry.kind != "app" {
            return;
        }
        self.history.record(&entry.id, now_ms());
        self.save_history();
        if let Err(error) = launcher::launch_desktop(&entry.id) {
            eprintln!("{{\"error\":\"launch failed: {error:?}\"}}");
        }
    }

    pub fn forget_clip(&self, key: &str) {
        crate::services::clipboard::forget(key);
    }

    pub fn open_clip_image(&self, entry: &Entry) {
        if entry.kind != "clip" {
            return;
        }
        if let Some(file) = entry.file.as_deref() {
            let path = crate::services::clipboard::payload_path(file);
            if path.as_os_str().is_empty() {
                return;
            }
            if let Err(error) = std::process::Command::new("imv").arg(path).spawn() {
                eprintln!("clipboard image viewer failed: {error}");
            }
        }
    }

    fn save_history(&self) {
        let Ok(text) = serde_json::to_string_pretty(&self.history) else {
            return;
        };
        let temporary = self.history_path.with_extension("json.tmp");
        if std::fs::write(&temporary, text).is_ok() {
            let _ = std::fs::rename(&temporary, &self.history_path);
        }
    }
}
