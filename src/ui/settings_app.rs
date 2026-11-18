//! Native Settings window. The schema is read from the audited original
//! catalogue; unknown keys are retained verbatim when a known value changes.
use gtk4::prelude::*;
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::rc::Rc;

const SCHEMA: &str = include_str!("../../docs/SETTINGS_SCHEMA.md");
const SECTIONS: &[(&str, &str, &[&str])] = &[
    (
        "THE SHELL",
        "bar",
        &[
            "barHeight",
            "barMargin",
            "barSideMargin",
            "barStyle",
            "barSides",
            "barLeft",
            "barRight",
            "barEverywhere",
            "barHidden",
            "screenCorners",
            "barShadow",
            "workspaceCount",
            "workspaceMax",
            "workspaceMode",
            "workspaceStyle",
            "captureKind",
            "captureShape",
        ],
    ),
    (
        "THE SHELL",
        "island",
        &[
            "islandAttached",
            "islandSummary",
            "clockFormat",
            "clockShowsDate",
            "clockShowsSeconds",
            "islandActivities",
            "islandActivitiesSeen",
            "notificationTimeout",
        ],
    ),
    (
        "THE SHELL",
        "controls",
        &[
            "centreButtons",
            "centreTop",
            "centreColumns",
            "centreRows",
            "centreBlocks",
            "centreToggles",
        ],
    ),
    (
        "THE SHELL",
        "launcher",
        &[
            "launcherResults",
            "launcherOrder",
            "launcherFits",
            "launcherPrefixes",
            "emojiTone",
            "clipboardHistory",
            "clipboardKeep",
            "clipboardImages",
            "clipboardWipeOnLock",
        ],
    ),
    (
        "THE SHELL",
        "dock",
        &[
            "dockEnabled",
            "dockPinned",
            "dockEdge",
            "dockAlignment",
            "dockIconSize",
            "dockMagnify",
            "dockOpacity",
            "dockGround",
            "dockRunning",
            "dockEverywhere",
            "dockAutohide",
            "dockLauncher",
            "dockShadow",
        ],
    ),
    (
        "THE DESK",
        "widgets",
        &[
            "desktopWidgets",
            "desktopTheme",
            "desktopOpacity",
            "desktopHidden",
            "deckOnEmpty",
            "spectrumOnEmpty",
            "widgetShadow",
        ],
    ),
    (
        "THE DESK",
        "modules",
        &[
            "lyrics",
            "weatherPlace",
            "githubUser",
            "markets",
            "petStyle",
            "cameraEnabled",
            "cameraEffect",
            "cameraReactions",
            "chipShape",
            "chipFigure",
            "notesHandwriting",
        ],
    ),
    (
        "THE DESK",
        "appearance",
        &[
            "surfaceStyle",
            "desktopGround",
            "terminalGround",
            "wallpaperTransition",
            "wallpaperDir",
            "matugenEnabled",
            "greeting",
            "fontFamily",
            "fontMono",
            "motionScale",
            "motionCurve",
            "animationPreset",
            "windowGlass",
            "windowShadow",
        ],
    ),
    ("THE DESK", "windows", &["compositor", "gameMode"]),
    ("THE DESK", "monitors", &["displays", "lidPolicy"]),
    (
        "THE DESK",
        "input",
        &["keyboard", "cursorColor", "cursorSize", "shakeToFind"],
    ),
    ("THE DESK", "keys", &["keys"]),
    (
        "THE SESSION",
        "session",
        &[
            "lockBlur",
            "lockClock",
            "lockMusic",
            "lockMusicGround",
            "userName",
            "userAvatar",
            "doNotDisturb",
            "recorderAudio",
            "idleLock",
            "idleScreen",
            "idleSuspend",
            "nightLight",
            "nightTemperature",
        ],
    ),
    ("THE SESSION", "system", &["language"]),
];

fn parse_default(raw: &str) -> Value {
    let raw = raw.trim();
    if raw == "null" {
        return Value::Null;
    }
    if raw == "({})" {
        return Value::Object(Map::new());
    }
    if let Ok(value) = serde_json::from_str(raw) {
        return value;
    }
    if raw.starts_with('[') {
        if let Ok(value) = serde_json::from_str(raw) {
            return value;
        }
    }
    Value::String(raw.trim_matches('"').to_string())
}

fn defaults() -> Map<String, Value> {
    let mut map = Map::new();
    for line in SCHEMA.lines().filter(|line| line.starts_with("| ")) {
        let columns: Vec<_> = line.trim_matches('|').split('|').map(str::trim).collect();
        if columns.len() != 3 || columns[0] == "Chave" || columns[0].starts_with("---") {
            continue;
        }
        map.insert(columns[0].to_string(), parse_default(columns[2]));
    }
    map
}

fn settings_path() -> std::path::PathBuf {
    super::settings::Settings::path()
}

fn load() -> Map<String, Value> {
    let mut values = defaults();
    if let Ok(text) = std::fs::read_to_string(settings_path()) {
        if let Ok(Value::Object(saved)) = serde_json::from_str::<Value>(&text) {
            values.extend(saved);
        }
    }
    values
}

fn save(values: &Map<String, Value>) -> Result<(), String> {
    let path = settings_path();
    let parent = path.parent().ok_or("settings path has no parent")?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.velatura-tmp");
    let text = serde_json::to_string_pretty(values).map_err(|error| error.to_string())?;
    std::fs::write(&temporary, text).map_err(|error| error.to_string())?;
    std::fs::rename(&temporary, &path).map_err(|error| error.to_string())
}

pub struct SettingsApp {
    window: gtk4::ApplicationWindow,
    values: Rc<RefCell<Map<String, Value>>>,
    page: gtk4::Stack,
    nav: gtk4::ListBox,
    status: gtk4::Label,
}

impl SettingsApp {
    pub fn new(app: &gtk4::Application) -> Rc<Self> {
        let window = gtk4::ApplicationWindow::new(app);
        window.set_title(Some("Settings"));
        window.set_default_size(980, 680);
        window.set_hide_on_close(true);
        window.add_css_class("velatura-settings-window");

        let values = Rc::new(RefCell::new(load()));
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        root.set_margin_top(14);
        root.set_margin_bottom(14);
        root.set_margin_start(14);
        root.set_margin_end(14);

        let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        sidebar.set_size_request(206, -1);
        sidebar.add_css_class("settings-sidebar");
        let heading = gtk4::Label::new(Some("VELATURA"));
        heading.add_css_class("settings-brand");
        heading.set_xalign(0.0);
        sidebar.append(&heading);

        let nav = gtk4::ListBox::new();
        nav.set_selection_mode(gtk4::SelectionMode::Single);
        nav.add_css_class("settings-navigation");
        sidebar.append(&nav);
        root.append(&sidebar);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        content.set_hexpand(true);
        content.set_vexpand(true);
        let top = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let title = gtk4::Label::new(Some("Settings"));
        title.add_css_class("settings-title");
        title.set_hexpand(true);
        title.set_xalign(0.0);
        top.append(&title);
        let close = gtk4::Button::with_label("Close");
        close.connect_clicked({
            let window = window.clone();
            move |_| window.set_visible(false)
        });
        top.append(&close);
        content.append(&top);

        let page = gtk4::Stack::new();
        page.set_transition_type(gtk4::StackTransitionType::Crossfade);
        page.set_transition_duration(140);
        page.set_hexpand(true);
        page.set_vexpand(true);
        content.append(&page);
        let status = gtk4::Label::new(Some("Changes are saved to the Velatura settings file."));
        status.add_css_class("settings-status");
        status.set_xalign(0.0);
        content.append(&status);
        root.append(&content);
        window.set_child(Some(&root));

        let app = Rc::new(Self {
            window,
            values,
            page,
            nav,
            status,
        });
        app.build_pages();
        app
    }

    fn build_pages(self: &Rc<Self>) {
        let mut selected = None;
        let mut last_group = "";
        for (group, id, keys) in SECTIONS {
            if *group != last_group {
                let label = gtk4::Label::new(Some(group));
                label.add_css_class("settings-category");
                label.set_xalign(0.0);
                label.set_margin_top(12);
                let heading = gtk4::ListBoxRow::new();
                heading.set_child(Some(&label));
                heading.set_selectable(false);
                heading.set_activatable(false);
                self.nav.append(&heading);
                last_group = group;
            }
            let row = gtk4::ListBoxRow::new();
            let label = gtk4::Label::new(Some(&(id[..1].to_uppercase() + &id[1..])));
            label.set_xalign(0.0);
            label.set_margin_top(8);
            label.set_margin_bottom(8);
            label.set_margin_start(10);
            label.set_margin_end(10);
            row.set_child(Some(&label));
            row.set_activatable(true);
            row.set_selectable(true);
            self.nav.append(&row);
            if selected.is_none() {
                selected = Some(row.clone());
            }

            let scroller = gtk4::ScrolledWindow::new();
            scroller.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
            scroller.set_vexpand(true);
            let page_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
            page_box.set_margin_top(4);
            page_box.set_margin_bottom(14);
            page_box.set_margin_start(8);
            page_box.set_margin_end(8);
            let page_title = gtk4::Label::new(Some(&(id[..1].to_uppercase() + &id[1..])));
            page_title.add_css_class("settings-page-title");
            page_title.set_xalign(0.0);
            page_box.append(&page_title);
            for key in *keys {
                if self.values.borrow().contains_key(*key) {
                    page_box.append(&self.field_row(key));
                }
            }
            scroller.set_child(Some(&page_box));
            self.page.add_named(&scroller, Some(id));
        }
        self.nav.connect_row_selected({
            let page = self.page.clone();
            move |_, row| {
                if let Some(row) = row {
                    if let Some(label) = row.child().and_downcast::<gtk4::Label>() {
                        let name = label.text().to_string().to_lowercase();
                        page.set_visible_child_name(&name);
                    }
                }
            }
        });
        if let Some(row) = selected {
            self.nav.select_row(Some(&row));
        }
    }

    fn field_row(self: &Rc<Self>, key: &str) -> gtk4::Widget {
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        row.add_css_class("settings-field-row");
        row.set_margin_top(4);
        row.set_margin_bottom(4);
        let label = gtk4::Label::new(Some(key));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        row.append(&label);
        let value = self
            .values
            .borrow()
            .get(key)
            .cloned()
            .unwrap_or(Value::Null);
        match value {
            Value::Bool(current) => {
                let control = gtk4::Switch::new();
                control.set_active(current);
                control.connect_active_notify({
                    let values = Rc::clone(&self.values);
                    let status = self.status.clone();
                    let key = key.to_string();
                    move |switch| persist(&values, &status, &key, Value::Bool(switch.is_active()))
                });
                row.append(&control);
            }
            Value::Number(number) if number.is_i64() => {
                let control = gtk4::SpinButton::with_range(-100000.0, 100000.0, 1.0);
                control.set_value(number.as_i64().unwrap_or_default() as f64);
                control.connect_value_changed({
                    let values = Rc::clone(&self.values);
                    let status = self.status.clone();
                    let key = key.to_string();
                    move |spin| persist(&values, &status, &key, Value::from(spin.value_as_int()))
                });
                row.append(&control);
            }
            Value::Number(number) => {
                let control = gtk4::SpinButton::with_range(-100000.0, 100000.0, 0.1);
                control.set_digits(2);
                control.set_value(number.as_f64().unwrap_or_default());
                control.connect_value_changed({
                    let values = Rc::clone(&self.values);
                    let status = self.status.clone();
                    let key = key.to_string();
                    move |spin| {
                        persist(
                            &values,
                            &status,
                            &key,
                            serde_json::Number::from_f64(spin.value())
                                .map(Value::Number)
                                .unwrap_or(Value::Null),
                        )
                    }
                });
                row.append(&control);
            }
            Value::String(current) => {
                let control = gtk4::Entry::new();
                control.set_text(&current);
                control.set_width_chars(24);
                control.connect_changed({
                    let values = Rc::clone(&self.values);
                    let status = self.status.clone();
                    let key = key.to_string();
                    move |entry| {
                        persist(
                            &values,
                            &status,
                            &key,
                            Value::String(entry.text().to_string()),
                        )
                    }
                });
                row.append(&control);
            }
            other => {
                let control = gtk4::Entry::new();
                control.set_width_chars(24);
                control.set_text(&serde_json::to_string(&other).unwrap_or_default());
                control.set_placeholder_text(Some("JSON value"));
                control.connect_activate({
                    let values = Rc::clone(&self.values);
                    let status = self.status.clone();
                    let key = key.to_string();
                    move |entry| {
                        if let Ok(parsed) = serde_json::from_str::<Value>(&entry.text()) {
                            persist(&values, &status, &key, parsed);
                        } else {
                            status.set_text("Invalid JSON; the previous value was kept.");
                        }
                    }
                });
                row.append(&control);
            }
        }
        row.upcast()
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn persist(
    values: &Rc<RefCell<Map<String, Value>>>,
    status: &gtk4::Label,
    key: &str,
    value: Value,
) {
    values.borrow_mut().insert(key.to_string(), value);
    match save(&values.borrow()) {
        Ok(()) => status.set_text("Saved. Restart Velatura to apply changes."),
        Err(error) => status.set_text(&format!("Could not save settings: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_schema_has_all_108_settings_and_sections_are_unique() {
        let values = defaults();
        assert_eq!(values.len(), 108);
        assert_eq!(SECTIONS.len(), 14);
        let keys: std::collections::HashSet<_> = SECTIONS
            .iter()
            .flat_map(|(_, _, keys)| keys.iter().copied())
            .collect();
        assert_eq!(
            keys.len(),
            SECTIONS
                .iter()
                .map(|(_, _, keys)| keys.len())
                .sum::<usize>()
        );
        assert_eq!(keys, values.keys().map(String::as_str).collect());
        assert!(values.contains_key("clipboardHistory"));
        assert!(values.contains_key("wallpaperDir"));
        assert!(values.contains_key("language"));
    }

    #[test]
    fn defaults_parser_preserves_json_values() {
        assert_eq!(parse_default("true"), Value::Bool(true));
        assert_eq!(parse_default("200"), Value::from(200));
        assert_eq!(parse_default("\"recent\""), Value::String("recent".into()));
        assert_eq!(parse_default("({})"), Value::Object(Map::new()));
    }
}
