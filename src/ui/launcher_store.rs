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
                })
                .collect(),
        )
    }

    /// Recorded here and not in the launch itself, so dock launches do not
    /// count towards the ranking.
    pub fn launch(&mut self, entry: &Entry) {
        if entry.kind != "app" {
            return;
        }
        self.history.record(&entry.id, now_ms());
        self.save_history();
        if let Err(error) = launcher::launch_desktop(&entry.id) {
            eprintln!("{{\"error\":\"launch failed: {error:?}\"}}");
        }
    }

    fn save_history(&self) {
        let Ok(text) = serde_json::to_string_pretty(&self.history) else { return };
        let temporary = self.history_path.with_extension("json.tmp");
        if std::fs::write(&temporary, text).is_ok() {
            let _ = std::fs::rename(&temporary, &self.history_path);
        }
    }
}
