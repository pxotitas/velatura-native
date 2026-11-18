//! Launcher service: application index, search ranking and safe launch.
//!
//! Port of `services/LauncherService.qml` and `scripts/applications.py`.
//! Desktop entries are scanned in XDG precedence order (first entry wins, so
//! a user override shadows the system copy), ranked by name match, decayed
//! launch frecency, favourites and name, and launched through `gtk-launch`
//! with the desktop id as a single argv entry — no shell, no string
//! concatenation of user data into commands.

use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

/// One indexed application, mirroring `applications.py`'s fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Application {
    pub kind: String,
    /// Desktop file basename, e.g. `firefox.desktop`.
    pub id: String,
    pub name: String,
    pub subtitle: String,
    /// Lowercase haystack of name, keywords and generic name.
    pub keywords: String,
    pub icon: String,
    pub exec: String,
    pub wmclass: String,
    pub categories: Vec<String>,
}

/// Application directories in XDG precedence order.
pub fn application_dirs() -> Vec<PathBuf> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local/share"));
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    let mut dirs = vec![data_home.join("applications")];
    for dir in data_dirs.split(':').filter(|d| !d.is_empty()) {
        dirs.push(PathBuf::from(dir).join("applications"));
    }
    dirs
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// Parse one `.desktop` file. Only `Type=Application` entries that are
/// displayed and not hidden are indexed, matching the original exactly.
pub fn read_entry(path: &std::path::Path) -> Option<Application> {
    let file = std::fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);
    let mut in_entry = false;
    let mut name = String::new();
    let mut comment = String::new();
    let mut generic_name = String::new();
    let mut keywords = String::new();
    let mut icon = String::new();
    let mut exec = String::new();
    let mut wmclass = String::new();
    let mut categories = String::new();
    for line in reader.lines() {
        let line = line.ok()?;
        if line.starts_with('[') {
            if in_entry {
                break;
            }
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().to_string();
        match key {
            "Type" => {
                if value != "Application" {
                    return None;
                }
            }
            "NoDisplay" if value == "true" => return None,
            "Hidden" if value == "true" => return None,
            "Name" => name = value,
            "Comment" => comment = value,
            "GenericName" => generic_name = value,
            "Keywords" => keywords = value,
            "Icon" => icon = value,
            "Exec" => exec = value,
            "StartupWMClass" => wmclass = value,
            "Categories" => categories = value,
            _ => {}
        }
    }
    let name = name.trim().to_string();
    if name.is_empty() {
        return None;
    }
    let keyword_haystack = format!("{name} {keywords} {generic_name}").to_lowercase();
    let subtitle = if comment.trim().is_empty() {
        if generic_name.trim().is_empty() {
            "Application".to_string()
        } else {
            generic_name.trim().to_string()
        }
    } else {
        comment.trim().to_string()
    };
    Some(Application {
        kind: "app".to_string(),
        id: path.file_name()?.to_string_lossy().into_owned(),
        name,
        subtitle,
        keywords: keyword_haystack,
        icon: icon.trim().to_string(),
        exec: exec.trim().to_string(),
        wmclass: wmclass.trim().to_string(),
        categories: categories
            .split(';')
            .filter(|part| !part.is_empty())
            .map(str::to_string)
            .collect(),
    })
}

/// Scan every application directory, first entry wins per XDG precedence.
pub fn list_applications() -> Vec<Application> {
    let mut applications: HashMap<String, Application> = HashMap::new();
    for directory in application_dirs() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !file_name.ends_with(".desktop") || applications.contains_key(file_name) {
                continue;
            }
            if let Some(app) = read_entry(&path) {
                applications.insert(file_name.to_string(), app);
            }
        }
    }
    let mut sorted: Vec<Application> = applications.into_values().collect();
    sorted.sort_by_key(|app| app.name.to_lowercase());
    sorted
}

// ── FRECENCY ────────────────────────────────────────────────────────────────

/// Decay half-life of a launch's weight: 30 days, from the original.
pub const HALF_LIFE_MS: f64 = 30.0 * 24.0 * 3600.0 * 1000.0;

/// Decayed launch weight, `count * 0.5^(age/halfLife)`, from `scoreOf`.
pub fn score(count: f64, last: u64, now_ms: u64) -> f64 {
    if count == 0.0 {
        return 0.0;
    }
    let age = now_ms.saturating_sub(last) as f64;
    count * 0.5f64.powf(age / HALF_LIFE_MS)
}

/// Recorded launch row persisted in the launcher history.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LaunchRow {
    pub count: f64,
    pub last: u64,
}

/// The launcher history: desktop id -> launch row.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LaunchHistory {
    pub launches: HashMap<String, LaunchRow>,
}

impl LaunchHistory {
    /// The stored count keeps the already-decayed value plus one, exactly as
    /// `record` in the original does, so old weight fades over time.
    pub fn record(&mut self, id: &str, now_ms: u64) {
        if id.is_empty() {
            return;
        }
        let score = self.score_of(id, now_ms) + 1.0;
        self.launches.insert(
            id.to_string(),
            LaunchRow {
                count: score,
                last: now_ms,
            },
        );
    }

    pub fn score_of(&self, id: &str, now_ms: u64) -> f64 {
        self.launches
            .get(id)
            .map(|row| score(row.count, row.last, now_ms))
            .unwrap_or(0.0)
    }
}

// ── SEARCH ──────────────────────────────────────────────────────────────────

/// The launcher's modes, in the original's declaration order. The
/// prefix-less mode must stay last.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode {
    pub id: &'static str,
    pub prefix: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub hint: &'static str,
    pub empty: &'static str,
}

pub const MODES: &[Mode] = &[
    Mode {
        id: "calculate",
        prefix: "=",
        label: "Calculate",
        icon: "󰃬",
        hint: "Work something out",
        empty: "An expression — 12 * 34, (5 + 5) / 2",
    },
    Mode {
        id: "desk",
        prefix: ">",
        label: "Desk",
        icon: "󰍜",
        hint: "Open a panel or an action",
        empty: "Nothing on the desk by that name",
    },
    Mode {
        id: "windows",
        prefix: "@",
        label: "Windows",
        icon: "󰖯",
        hint: "Find an open window",
        empty: "No window by that name is open",
    },
    Mode {
        id: "timer",
        prefix: "!",
        label: "Timer",
        icon: "󰔟",
        hint: "Start a countdown",
        empty: "A duration — 25m, 90s, 1:30",
    },
    Mode {
        id: "clipboard",
        prefix: "'",
        label: "Clipboard",
        icon: "󰅍",
        hint: "Copy something again",
        empty: "Nothing copied says that",
    },
    Mode {
        id: "emoji",
        prefix: ":",
        label: "Emoji",
        icon: "󰇲",
        hint: "Copy an emoji",
        empty: "No emoji by that name",
    },
    Mode {
        id: "apps",
        prefix: "",
        label: "Apps",
        icon: "󰀻",
        hint: "Search applications",
        empty: "No application matches",
    },
];

/// The mode a query resolves to: the first sigil that prefixes it wins, and
/// the prefix-less mode must be last.
pub fn mode_for(query: &str) -> &'static Mode {
    MODES
        .iter()
        .find(|mode| !mode.prefix.is_empty() && query.starts_with(mode.prefix))
        .unwrap_or(&MODES[MODES.len() - 1])
}

pub fn term_for(query: &str) -> &str {
    query
        .strip_prefix(mode_for(query).prefix)
        .unwrap_or(query)
        .trim()
}

/// Evaluate the arithmetic grammar accepted by LauncherService.qml.
/// Only decimal numbers, whitespace, parentheses and + - * / % are allowed.
pub fn evaluate(expression: &str) -> Option<f64> {
    if expression.is_empty()
        || !expression.chars().all(|c| {
            c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '%' | '(' | ')' | '.' | ' ')
        })
    {
        return None;
    }
    struct Parser<'a> {
        chars: Vec<char>,
        at: usize,
        source: &'a str,
    }
    impl Parser<'_> {
        fn skip(&mut self) {
            while self
                .chars
                .get(self.at)
                .is_some_and(|c| c.is_ascii_whitespace())
            {
                self.at += 1;
            }
        }
        fn take(&mut self, c: char) -> bool {
            self.skip();
            if self.chars.get(self.at) == Some(&c) {
                self.at += 1;
                true
            } else {
                false
            }
        }
        fn expression(&mut self) -> Option<f64> {
            let mut value = self.term()?;
            loop {
                if self.take('+') {
                    value += self.term()?;
                } else if self.take('-') {
                    value -= self.term()?;
                } else {
                    break;
                }
            }
            value.is_finite().then_some(value)
        }
        fn term(&mut self) -> Option<f64> {
            let mut value = self.factor()?;
            loop {
                if self.take('*') {
                    value *= self.factor()?;
                } else if self.take('/') {
                    value /= self.factor()?;
                } else if self.take('%') {
                    value %= self.factor()?;
                } else {
                    break;
                }
                if !value.is_finite() {
                    return None;
                }
            }
            Some(value)
        }
        fn factor(&mut self) -> Option<f64> {
            if self.take('+') {
                return self.factor();
            }
            if self.take('-') {
                return Some(-self.factor()?);
            }
            if self.take('(') {
                let value = self.expression()?;
                if !self.take(')') {
                    return None;
                }
                return Some(value);
            }
            self.skip();
            let start = self.at;
            let mut dot = false;
            while let Some(c) = self.chars.get(self.at) {
                if c.is_ascii_digit() {
                    self.at += 1;
                } else if *c == '.' && !dot {
                    dot = true;
                    self.at += 1;
                } else {
                    break;
                }
            }
            if start == self.at {
                return None;
            }
            self.source
                .get(self.char_byte(start)..self.char_byte(self.at))?
                .parse()
                .ok()
        }
        fn char_byte(&self, index: usize) -> usize {
            self.chars.iter().take(index).map(|c| c.len_utf8()).sum()
        }
    }
    let mut parser = Parser {
        chars: expression.chars().collect(),
        at: 0,
        source: expression,
    };
    let value = parser.expression()?;
    parser.skip();
    (parser.at == parser.chars.len() && value.is_finite()).then_some(value)
}

/// Rank applications for a term, mirroring the original's ordering: name
/// starts with the term first, then decayed launches plus the favourite
/// bonus, then dock order between equal favourites, then name.
pub fn rank(
    applications: &[Application],
    term: &str,
    history: &LaunchHistory,
    favourites: &[String],
    now_ms: u64,
    by_use: bool,
    max_results: usize,
) -> Vec<Application> {
    let term = term.to_lowercase();
    let found: Vec<&Application> = applications
        .iter()
        .filter(|app| {
            term.is_empty()
                || app.name.to_lowercase().contains(&term)
                || app.keywords.contains(&term)
        })
        .collect();

    let mut ranked = found;
    ranked.sort_by(|a, b| {
        if !term.is_empty() {
            let lead_a = a.name.to_lowercase().starts_with(&term);
            let lead_b = b.name.to_lowercase().starts_with(&term);
            if lead_a != lead_b {
                return if lead_a {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
        }
        if by_use {
            let weight_a = history.score_of(&a.id, now_ms)
                + i32::from(favourites.iter().any(|f| f == &a.id)) as f64;
            let weight_b = history.score_of(&b.id, now_ms)
                + i32::from(favourites.iter().any(|f| f == &b.id)) as f64;
            if (weight_b - weight_a).abs() > 0.0001 {
                return weight_b
                    .partial_cmp(&weight_a)
                    .unwrap_or(std::cmp::Ordering::Equal);
            }
            let kept_a = favourites.iter().position(|f| f == &a.id);
            let kept_b = favourites.iter().position(|f| f == &b.id);
            if let (Some(ka), Some(kb)) = (kept_a, kept_b)
                && ka != kb
            {
                return ka.cmp(&kb);
            }
        }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });
    ranked.into_iter().take(max_results).cloned().collect()
}

// ── LAUNCH ──────────────────────────────────────────────────────────────────

/// Launch a desktop entry by id through `gtk-launch` with fixed, separate
/// argv entries. The id is validated against the desktop id charset before
/// it reaches the process: `[A-Za-z0-9._-]`, and it must end in `.desktop`.
/// Nothing from the UI or the index is ever interpolated into a shell line.
pub fn launch_desktop(desktop_id: &str) -> Result<(), LaunchError> {
    if desktop_id.is_empty() {
        return Err(LaunchError::EmptyId);
    }
    if !desktop_id.ends_with(".desktop")
        || desktop_id.len() > 255
        || !desktop_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '@'))
    {
        return Err(LaunchError::InvalidId);
    }
    std::process::Command::new("gtk-launch")
        .arg(desktop_id)
        .stdin(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| LaunchError::Spawn(error.to_string()))
}

#[derive(Debug, Clone, PartialEq)]
pub enum LaunchError {
    EmptyId,
    InvalidId,
    Spawn(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_desktop(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn parses_entry_and_hides_hidden() {
        let dir =
            std::env::temp_dir().join(format!("velatura-launcher-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let good = write_desktop(
            &dir,
            "one.desktop",
            "[Desktop Entry]\nType=Application\nName=Foxtrot\nExec=foxtrot %U\nKeywords=ft;fox;\nComment=A test app\nIcon=fox\n",
        );
        assert!(read_entry(&good).is_some());
        let app = read_entry(&good).unwrap();
        assert_eq!(app.id, "one.desktop");
        assert_eq!(app.name, "Foxtrot");
        assert_eq!(app.subtitle, "A test app");
        assert!(app.keywords.contains("ft"));
        let hidden = write_desktop(
            &dir,
            "two.desktop",
            "[Desktop Entry]\nType=Application\nName=Hidden\nNoDisplay=true\nExec=x\n",
        );
        assert!(read_entry(&hidden).is_none());
        let wrong_type = write_desktop(
            &dir,
            "three.desktop",
            "[Desktop Entry]\nType=Link\nName=Link\n",
        );
        assert!(read_entry(&wrong_type).is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn frecency_decays_and_records() {
        let mut history = LaunchHistory::default();
        history.record("a.desktop", 0);
        assert!((history.score_of("a.desktop", 0) - 1.0).abs() < 1e-9);
        // One half-life halves the weight.
        let half = HALF_LIFE_MS as u64;
        assert!((history.score_of("a.desktop", half) - 0.5).abs() < 1e-9);
        history.record("a.desktop", half);
        assert!((history.score_of("a.desktop", half) - 1.5).abs() < 1e-9);
    }

    #[test]
    fn history_json_round_trip() {
        let mut history = LaunchHistory::default();
        history.record("x.desktop", 42);
        let json = serde_json::to_string(&history).unwrap();
        let back: LaunchHistory = serde_json::from_str(&json).unwrap();
        assert_eq!(back, history);
    }

    #[test]
    fn mode_sigils_route_and_terms_strip() {
        assert_eq!(mode_for("=1+2").id, "calculate");
        assert_eq!(term_for("=1+2"), "1+2");
        assert_eq!(mode_for(":cat").id, "emoji");
        assert_eq!(term_for(":cat"), "cat");
        // A plain word never routes by letter; only sigils do, exactly like
        // `modeFor` in the original.
        assert_eq!(mode_for("fox").id, "apps");
        assert_eq!(term_for("fox"), "fox");
        assert_eq!(mode_for("").id, "apps");
    }

    #[test]
    fn rank_prefers_prefix_then_weight_then_name() {
        let apps = vec![
            Application {
                kind: "app".into(),
                id: "zed.desktop".into(),
                name: "Zed".into(),
                subtitle: String::new(),
                keywords: "fast editor".into(),
                icon: String::new(),
                exec: "zed".into(),
                wmclass: String::new(),
                categories: vec![],
            },
            Application {
                kind: "app".into(),
                id: "foo.desktop".into(),
                name: "Firefox".into(),
                subtitle: String::new(),
                keywords: "firefox web".into(),
                icon: String::new(),
                exec: "firefox".into(),
                wmclass: String::new(),
                categories: vec![],
            },
            Application {
                kind: "app".into(),
                id: "feed.desktop".into(),
                name: "Feeds".into(),
                subtitle: String::new(),
                keywords: "feeds".into(),
                icon: String::new(),
                exec: "feeds".into(),
                wmclass: String::new(),
                categories: vec![],
            },
        ];
        let mut history = LaunchHistory::default();
        history.record("zed.desktop", 0);
        let ranked = rank(&apps, "f", &history, &[], 0, true, 10);
        // Prefix match beats frecency: Feeds and Firefox lead over Zed.
        assert_eq!(ranked[0].id, "feed.desktop");
        assert_eq!(ranked[1].id, "foo.desktop");
        assert_eq!(ranked[2].id, "zed.desktop");
        // No term: frecency wins.
        let ranked = rank(&apps, "", &history, &[], 0, true, 10);
        assert_eq!(ranked[0].id, "zed.desktop");
        // Alphabetical without byUse.
        let ranked = rank(&apps, "", &history, &[], 0, false, 10);
        assert_eq!(ranked[0].name, "Feeds");
    }

    #[test]
    fn calculator_obeys_precedence_parentheses_and_rejects_unsafe_input() {
        assert_eq!(evaluate("12 * (5 + 1)"), Some(72.0));
        assert_eq!(evaluate("10 / 4"), Some(2.5));
        assert_eq!(evaluate("10 % 4"), Some(2.0));
        assert_eq!(evaluate("-2 + 3"), Some(1.0));
        assert_eq!(evaluate("1 / 0"), None);
        assert_eq!(evaluate("1 + process"), None);
        assert_eq!(evaluate("(1 + 2"), None);
    }

    #[test]
    fn launch_rejects_bad_ids() {
        assert_eq!(launch_desktop(""), Err(LaunchError::EmptyId));
        assert!(matches!(
            launch_desktop("x; rm -rf"),
            Err(LaunchError::InvalidId)
        ));
        assert!(matches!(
            launch_desktop("no-ext"),
            Err(LaunchError::InvalidId)
        ));
        assert!(matches!(
            launch_desktop("$(echo hi).desktop"),
            Err(LaunchError::InvalidId)
        ));
        // A valid id fails at spawn in tests only because gtk-launch may be
        // absent; the point is that argv validation passed.
    }
}
