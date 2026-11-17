//! Launcher panel: search field, mode sigils, results and activation.
//!
//! Port of `bar/island/LauncherPanel.qml` on top of `services::launcher`.
//! Modes route by the first character: `=` calculate, `>` desk, `@` windows,
//! `!` timer, `'` clipboard, `:` emoji, plain text applications. This slice
//! implements the modes with real backing services: applications, clipboard,
//! windows and the calculator; timer/emoji arrive with their services.
//!
//! A mode row keeps the launcher open by rewriting the query with its sigil;
//! a panel row asks the island to open a panel through the typed channel.

use gtk4::gdk;
use gtk4::glib;
use gtk4::pango;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, Label, Orientation};

use crate::services::clipboard::Entry as ClipEntry;
use crate::services::launcher::{self, Application, LaunchError, LaunchHistory, Mode};

use super::shell::ServiceHandles;
use super::theme;

/// One resolved result row, spanning every mode.
#[derive(Debug, Clone, PartialEq)]
enum ResultRow {
    App(Application),
    Clip(ClipEntry),
    Window {
        id: i64,
        title: String,
        app_id: String,
        workspace: String,
    },
    Calculation {
        expression: String,
        value: String,
    },
    ModeSigil(Mode),
    Panel {
        panel: String,
        label: String,
    },
    Empty {
        message: String,
    },
}

/// The launcher's declared panel metrics (`LauncherService.qml`); the island
/// reads the same numbers, so the height math stays in sync.
pub fn panel_width() -> i32 {
    theme::LAUNCHER_PANEL_WIDTH
}

pub fn panel_height() -> i32 {
    theme::LAUNCHER_FIXED_HEIGHT
}

/// Everything the launcher needs from the running services.
#[derive(Clone)]
pub struct LaunchContext {
    pub services: ServiceHandles,
    applications: std::rc::Rc<std::cell::RefCell<Vec<Application>>>,
    history: std::rc::Rc<std::cell::RefCell<LaunchHistory>>,
    clipboard: std::rc::Rc<std::cell::RefCell<Vec<ClipEntry>>>,
}

impl LaunchContext {
    pub fn new(services: ServiceHandles) -> Self {
        Self {
            services,
            applications: std::rc::Rc::default(),
            history: std::rc::Rc::default(),
            clipboard: std::rc::Rc::default(),
        }
    }

    pub fn reload_applications(&self) {
        *self.applications.borrow_mut() = launcher::list_applications();
    }

    pub fn load_history(&self) {
        let path = crate::services::paths::state_directory().join("launcher-history.json");
        if let Ok(text) = std::fs::read_to_string(&path)
            && let Ok(history) = serde_json::from_str::<LaunchHistory>(&text)
        {
            *self.history.borrow_mut() = history;
        }
    }

    pub fn save_history(&self) {
        let path = crate::services::paths::state_directory().join("launcher-history.json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string(&*self.history.borrow()) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn reload_clipboard(&self) {
        *self.clipboard.borrow_mut() = self.services.clipboard.entries();
    }

    pub fn record(&self, id: &str) {
        self.history
            .borrow_mut()
            .record(id, crate::services::now_ms());
        self.save_history();
    }

    /// Resolve a query into result rows, dispatching by the mode sigil like
    /// `LauncherService.qml`'s mode stack. Applications, clipboard, windows
    /// and the calculator resolve real data; the desk, timer and emoji modes
    /// arrive with their services and answer with the original's empty text.
    fn resolve(&self, query: &str) -> Vec<ResultRow> {
        let mode = launcher::mode_for(query);
        let term = launcher::term_for(query);
        match mode.id {
            "calculate" => {
                return evaluate(term).map_or_else(
                    || {
                        vec![ResultRow::Empty {
                            message: mode.empty.to_string(),
                        }]
                    },
                    |value| {
                        vec![ResultRow::Calculation {
                            expression: term.to_string(),
                            value: format_value(value),
                        }]
                    },
                );
            }
            "clipboard" => {
                let entries = self.services.clipboard.entries();
                return crate::services::clipboard::search(
                    &entries,
                    term,
                    theme::LAUNCHER_MAX_RESULTS,
                )
                .into_iter()
                .cloned()
                .map(ResultRow::Clip)
                .collect();
            }
            "windows" => {
                let snapshot = self.services.niri.snapshot();
                let workspaces: std::collections::HashMap<i64, i64> = snapshot
                    .niri
                    .workspaces
                    .iter()
                    .filter_map(|workspace| {
                        Some((
                            workspace.get("id")?.as_i64()?,
                            workspace.get("idx")?.as_i64()?,
                        ))
                    })
                    .collect();
                let term = term.to_lowercase();
                return snapshot
                    .niri
                    .windows
                    .iter()
                    .filter_map(|window| {
                        let id = window.get("id")?.as_i64()?;
                        let title = window
                            .get("title")
                            .and_then(|value| value.as_str())
                            .unwrap_or("")
                            .to_string();
                        let app_id = window
                            .get("app_id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("")
                            .to_string();
                        let workspace = window
                            .get("workspace_id")
                            .and_then(|value| value.as_i64())
                            .and_then(|workspace_id| workspaces.get(&workspace_id).copied())
                            .unwrap_or(0)
                            .to_string();
                        if !term.is_empty()
                            && !title.to_lowercase().contains(&term)
                            && !app_id.to_lowercase().contains(&term)
                        {
                            return None;
                        }
                        Some(ResultRow::Window {
                            id,
                            title,
                            app_id,
                            workspace,
                        })
                    })
                    .take(theme::LAUNCHER_MAX_RESULTS)
                    .collect();
            }
            "desk" | "timer" | "emoji" => {
                return vec![ResultRow::Empty {
                    message: mode.empty.to_string(),
                }];
            }
            _ => {}
        }
        // Applications by use, with the other modes' sigils offered on an
        // empty query, exactly as the original's plain-text mode does.
        let apps = self.applications.borrow();
        let ranked = launcher::rank(
            &apps,
            term,
            &self.history.borrow(),
            &[],
            crate::services::now_ms(),
            true,
            theme::LAUNCHER_MAX_RESULTS,
        );
        drop(apps);
        let mut rows: Vec<ResultRow> = ranked.into_iter().map(ResultRow::App).collect();
        if term.is_empty() {
            for sigil in launcher::MODES
                .iter()
                .filter(|mode| !mode.prefix.is_empty())
            {
                rows.push(ResultRow::ModeSigil(*sigil));
            }
            // The one panel wired today: the notifications history.
            rows.push(ResultRow::Panel {
                panel: "notifications".to_string(),
                label: "Notifications".to_string(),
            });
        }
        if rows.is_empty() {
            rows.push(ResultRow::Empty {
                message: mode.empty.to_string(),
            });
        }
        rows
    }
}

/// The panel widget. Lives inside the island's stack; the island calls
/// `on_open`/`on_closed` with the layer.
pub struct LauncherPanel {
    root: GtkBox,
    field: Entry,
    rows: GtkBox,
    hint: Label,
    results: std::rc::Rc<std::cell::RefCell<Vec<ResultRow>>>,
    context: LaunchContext,
    panel_requests: async_channel::Sender<String>,
}

impl LauncherPanel {
    pub fn new(services: ServiceHandles, panel_requests: async_channel::Sender<String>) -> Self {
        let context = LaunchContext::new(services);
        let root = GtkBox::new(Orientation::Vertical, theme::LAUNCHER_GAP);
        root.set_css_classes(&["velatura-panel-inner"]);

        let field = Entry::new();
        field.set_placeholder_text(Some("Search applications"));
        field.set_css_classes(&["velatura-launcher-field"]);

        let hint = Label::new(Some("Apps — Search applications"));
        hint.set_css_classes(&["velatura-muted"]);
        hint.set_halign(gtk4::Align::Start);

        let rows = GtkBox::new(Orientation::Vertical, theme::LAUNCHER_ROW_SPACING);
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&rows));
        scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scroll.set_vexpand(true);

        root.append(&field);
        root.append(&hint);
        root.append(&scroll);

        let mut panel = Self {
            root,
            field,
            rows,
            hint,
            results: std::rc::Rc::default(),
            context,
            panel_requests,
        };
        panel.wire();
        panel
    }

    pub fn root(&self) -> &GtkBox {
        &self.root
    }

    /// Called when the island shows the launcher: focus the field and load
    /// the application index, history and clipboard entries.
    pub fn on_open(&mut self) {
        self.context.reload_applications();
        self.context.load_history();
        self.context.reload_clipboard();
        self.field.grab_focus();
        self.rebuild(&self.field.text());
    }

    /// The query lives in the panel; cleared when the panel closes.
    pub fn on_closed(&mut self) {
        self.field.set_text("");
        *self.results.borrow_mut() = Vec::new();
    }

    fn wire(&mut self) {
        let weak_self = PanelWeak::new(self);
        self.field.connect_changed(move |field| {
            let query = field.text().to_string();
            if let Some(panel) = weak_self.upgrade() {
                panel.rebuild(&query);
            }
        });
        let weak_rows = self.rows.downgrade();
        let key = gtk4::EventControllerKey::new();
        key.connect_key_pressed(move |_controller, key, _code, _state| {
            let Some(rows) = weak_rows.upgrade() else {
                return glib::Propagation::Proceed;
            };
            match key {
                gdk::Key::Down => {
                    rows.child_focus(gtk4::DirectionType::Down);
                    glib::Propagation::Stop
                }
                gdk::Key::Up => {
                    rows.child_focus(gtk4::DirectionType::Up);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        self.field.add_controller(key);
    }

    fn rebuild(&self, query: &str) {
        let mode = launcher::mode_for(query);
        self.hint
            .set_text(&format!("{} — {}", mode.label, mode.hint));
        let results = self.context.resolve(query);
        *self.results.borrow_mut() = results.clone();

        while let Some(child) = self.rows.first_child() {
            self.rows.remove(&child);
        }
        for row in &results {
            match row {
                ResultRow::Empty { message } => {
                    let label = Label::new(Some(message.as_str()));
                    label.set_css_classes(&["velatura-muted"]);
                    label.set_halign(gtk4::Align::Start);
                    let holder = GtkBox::new(Orientation::Horizontal, 0);
                    holder.set_css_classes(&["velatura-launcher-row"]);
                    holder.append(&label);
                    self.rows.append(&holder);
                }
                _ => {
                    let button = Button::new();
                    button.set_css_classes(&["velatura-launcher-row"]);
                    let content = GtkBox::new(Orientation::Horizontal, 12);
                    let column = GtkBox::new(Orientation::Vertical, 2);
                    let name = Label::new(Some(row_title(row).as_str()));
                    name.set_css_classes(&["velatura-launcher-name"]);
                    name.set_halign(gtk4::Align::Start);
                    name.set_ellipsize(pango::EllipsizeMode::End);
                    let subtitle = Label::new(Some(row_subtitle(row).as_str()));
                    subtitle.set_css_classes(&["velatura-launcher-subtitle"]);
                    subtitle.set_halign(gtk4::Align::Start);
                    subtitle.set_ellipsize(pango::EllipsizeMode::End);
                    column.append(&name);
                    column.append(&subtitle);
                    content.append(&column);
                    button.set_child(Some(&content));

                    let context = self.context.clone();
                    let results = std::rc::Rc::clone(&self.results);
                    let field = self.field.downgrade();
                    let panel_requests = self.panel_requests.clone();
                    let clicked = row.clone();
                    button.connect_clicked(move |_| {
                        let index = row_index(&results, &clicked);
                        let Some(row) =
                            index.and_then(|index| results.borrow().get(index).cloned())
                        else {
                            return;
                        };
                        match row {
                            ResultRow::ModeSigil(mode) => {
                                // A mode row keeps the launcher open: its
                                // sigil is typed into the field.
                                if let Some(field) = field.upgrade() {
                                    field.set_text(&format!("{} ", mode.prefix));
                                    field.set_position(-1);
                                }
                            }
                            ResultRow::Panel { panel, .. } => {
                                let _ = panel_requests.send_blocking(panel);
                            }
                            plain => context.activate(&plain),
                        }
                    });
                    self.rows.append(&button);
                }
            }
        }
    }
}

fn row_index(
    results: &std::rc::Rc<std::cell::RefCell<Vec<ResultRow>>>,
    row: &ResultRow,
) -> Option<usize> {
    results.borrow().iter().position(|other| other == row)
}

fn row_title(row: &ResultRow) -> String {
    match row {
        ResultRow::App(app) => app.name.clone(),
        ResultRow::Clip(entry) => crate::services::clipboard::title(entry),
        ResultRow::Window { title, app_id, .. } => {
            if title.is_empty() {
                app_id.clone()
            } else {
                title.clone()
            }
        }
        ResultRow::Calculation { value, .. } => value.clone(),
        ResultRow::ModeSigil(mode) => mode.label.to_string(),
        ResultRow::Panel { label, .. } => label.clone(),
        ResultRow::Empty { message } => message.clone(),
    }
}

fn row_subtitle(row: &ResultRow) -> String {
    match row {
        ResultRow::App(app) => app.subtitle.clone(),
        ResultRow::Clip(entry) => crate::services::clipboard::describe(entry),
        ResultRow::Window {
            app_id, workspace, ..
        } => {
            format!("{app_id}  ·  workspace {workspace}")
        }
        ResultRow::Calculation { expression, value } => {
            format!("{expression}  ·  Enter copies {value}")
        }
        ResultRow::ModeSigil(mode) => mode.hint.to_string(),
        ResultRow::Panel { label, .. } => label.clone(),
        ResultRow::Empty { .. } => String::new(),
    }
}

/// Weak reference to the panel through its root widget, so GTK callbacks
/// never own it strongly.
struct PanelWeak {
    root: glib::WeakRef<GtkBox>,
    inner: std::rc::Rc<std::cell::RefCell<Option<PanelInner>>>,
}

struct PanelInner {
    field: glib::WeakRef<Entry>,
    rows: glib::WeakRef<GtkBox>,
    hint: glib::WeakRef<Label>,
    results: std::rc::Rc<std::cell::RefCell<Vec<ResultRow>>>,
    context: LaunchContext,
    panel_requests: async_channel::Sender<String>,
}

impl PanelWeak {
    fn new(panel: &LauncherPanel) -> Self {
        let root_ref = glib::WeakRef::new();
        root_ref.set(Some(&panel.root));
        let field_ref = glib::WeakRef::new();
        field_ref.set(Some(&panel.field));
        let rows_ref = glib::WeakRef::new();
        rows_ref.set(Some(&panel.rows));
        let hint_ref = glib::WeakRef::new();
        hint_ref.set(Some(&panel.hint));
        Self {
            root: root_ref,
            inner: std::rc::Rc::new(std::cell::RefCell::new(Some(PanelInner {
                field: field_ref,
                rows: rows_ref,
                hint: hint_ref,
                results: std::rc::Rc::clone(&panel.results),
                context: panel.context.clone(),
                panel_requests: panel.panel_requests.clone(),
            }))),
        }
    }

    fn upgrade(&self) -> Option<PanelRef<'_>> {
        let root = self.root.upgrade()?;
        Some(PanelRef { root, weak: self })
    }
}

struct PanelRef<'a> {
    #[allow(dead_code)]
    root: GtkBox,
    weak: &'a PanelWeak,
}

impl PanelRef<'_> {
    fn rebuild(&self, query: &str) {
        let guard = self.weak.inner.borrow();
        let Some(inner) = guard.as_ref() else {
            return;
        };
        let Some(rows) = inner.rows.upgrade() else {
            return;
        };
        let mode = launcher::mode_for(query);
        if let Some(hint) = inner.hint.upgrade() {
            hint.set_text(&format!("{} — {}", mode.label, mode.hint));
        }
        let results = inner.context.resolve(query);
        *inner.results.borrow_mut() = results.clone();

        while let Some(child) = rows.first_child() {
            rows.remove(&child);
        }
        for row in &results {
            match row {
                ResultRow::Empty { message } => {
                    let label = Label::new(Some(message.as_str()));
                    label.set_css_classes(&["velatura-muted"]);
                    label.set_halign(gtk4::Align::Start);
                    let holder = GtkBox::new(Orientation::Horizontal, 0);
                    holder.set_css_classes(&["velatura-launcher-row"]);
                    holder.append(&label);
                    rows.append(&holder);
                }
                _ => {
                    let button = Button::new();
                    button.set_css_classes(&["velatura-launcher-row"]);
                    let content = GtkBox::new(Orientation::Horizontal, 12);
                    let column = GtkBox::new(Orientation::Vertical, 2);
                    let name = Label::new(Some(row_title(row).as_str()));
                    name.set_css_classes(&["velatura-launcher-name"]);
                    name.set_halign(gtk4::Align::Start);
                    name.set_ellipsize(pango::EllipsizeMode::End);
                    let subtitle = Label::new(Some(row_subtitle(row).as_str()));
                    subtitle.set_css_classes(&["velatura-launcher-subtitle"]);
                    subtitle.set_halign(gtk4::Align::Start);
                    subtitle.set_ellipsize(pango::EllipsizeMode::End);
                    column.append(&name);
                    column.append(&subtitle);
                    content.append(&column);
                    button.set_child(Some(&content));

                    let clicked = row.clone();
                    let context = inner.context.clone();
                    let field_weak = inner.field.clone();
                    let panel_requests = inner.panel_requests.clone();
                    button.connect_clicked(move |_| {
                        match &clicked {
                            ResultRow::ModeSigil(mode) => {
                                // A mode row keeps the launcher open: its
                                // sigil is typed into the field.
                                if let Some(field) = field_weak.upgrade() {
                                    field.set_text(&format!("{} ", mode.prefix));
                                    field.set_position(-1);
                                }
                            }
                            ResultRow::Panel { panel, .. } => {
                                let _ = panel_requests.send_blocking(panel.clone());
                            }
                            plain => context.activate(plain),
                        }
                    });
                    rows.append(&button);
                }
            }
        }
    }
}

impl LaunchContext {
    fn activate(&self, row: &ResultRow) {
        match row {
            ResultRow::App(app) => {
                self.record(&app.id);
                if let Err(error) = launcher::launch_desktop(&app.id) {
                    report_launch_error(&error);
                }
            }
            ResultRow::Clip(entry) => {
                let _ = self.services.clipboard.restore(&entry.key);
            }
            ResultRow::Window { id, .. } => {
                // Separate, validated argv entries; no shell anywhere.
                let _ = self.services.niri.dispatch([
                    "action",
                    "focus-window",
                    "--id",
                    &id.to_string(),
                ]);
            }
            ResultRow::Calculation { value, .. } => copy_text(value),
            ResultRow::ModeSigil(_) | ResultRow::Panel { .. } | ResultRow::Empty { .. } => {}
        }
    }
}

fn report_launch_error(_error: &LaunchError) {
    // Kept quiet on the island; real failures surface through diagnostics.
}

fn copy_text(text: &str) {
    let _ = std::process::Command::new("wl-copy")
        .arg("--")
        .arg(text)
        .stdin(std::process::Stdio::null())
        .spawn();
}

fn format_value(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// Evaluate a whitelisted arithmetic expression by recursive descent. The
/// original calls a JavaScript evaluator behind the same whitelist.
fn evaluate(expression: &str) -> Option<f64> {
    struct Parser {
        tokens: Vec<char>,
        position: usize,
    }

    impl Parser {
        fn peek(&self) -> Option<char> {
            self.tokens.get(self.position).copied()
        }

        fn parse_expr(&mut self) -> Option<f64> {
            let mut value = self.parse_term()?;
            while matches!(self.peek(), Some('+') | Some('-')) {
                let operator = self.tokens[self.position];
                self.position += 1;
                let right = self.parse_term()?;
                value = if operator == '+' {
                    value + right
                } else {
                    value - right
                };
            }
            Some(value)
        }

        fn parse_term(&mut self) -> Option<f64> {
            let mut value = self.parse_factor()?;
            while matches!(self.peek(), Some('*') | Some('/')) {
                let operator = self.tokens[self.position];
                self.position += 1;
                let right = self.parse_factor()?;
                value = if operator == '*' {
                    value * right
                } else {
                    value / right
                };
            }
            Some(value)
        }

        fn parse_factor(&mut self) -> Option<f64> {
            match self.peek()? {
                '(' => {
                    self.position += 1;
                    let value = self.parse_expr()?;
                    (self.peek() == Some(')')).then(|| {
                        self.position += 1;
                        value
                    })
                }
                '-' => {
                    self.position += 1;
                    self.parse_factor().map(|value| -value)
                }
                _ => {
                    let start = self.position;
                    while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '.') {
                        self.position += 1;
                    }
                    if start == self.position {
                        return None;
                    }
                    self.tokens[start..self.position]
                        .iter()
                        .collect::<String>()
                        .parse::<f64>()
                        .ok()
                }
            }
        }
    }

    let mut parser = Parser {
        tokens: expression.chars().filter(|c| !c.is_whitespace()).collect(),
        position: 0,
    };
    let value = parser.parse_expr()?;
    (parser.position == parser.tokens.len()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_is_evaluated_safely() {
        assert_eq!(evaluate("12 * 34"), Some(408.0));
        assert_eq!(evaluate("(5 + 5) / 2"), Some(5.0));
        assert_eq!(evaluate("2+2*3"), Some(8.0));
        assert_eq!(evaluate("-4 / 2"), Some(-2.0));
        // Malformed input never evaluates.
        assert_eq!(evaluate("12 +"), None);
        assert_eq!(evaluate("(2"), None);
        // Letters and shell characters are impossible by whitelist.
        assert!(
            !"rm -rf /"
                .chars()
                .all(|c| "0123456789+-*/(). %".contains(c))
        );
    }

    #[test]
    fn values_format_without_noise() {
        assert_eq!(format_value(408.0), "408");
        assert_eq!(format_value(2.5), "2.5");
    }
}
