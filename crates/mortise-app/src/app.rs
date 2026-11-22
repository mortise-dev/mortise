//! The app: its state, the keys, and the three columns of the workbench.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Event, Key, KeyboardShortcut, Modifiers, Rect, Theme, Ui, pos2, vec2};
use mortise_core::{
    AppEntry, AppSnapshot, LiveApp, LiveState, Settings, SettingsError, propose_monogram, read_app,
};
use mortise_core::{
    Environment, LiveRuntime, ReadOptions, Rules, RuntimeState, Secret, Secrets, ThemeChoice,
};

use crate::add::{self, AddAction, AddPanel, Checked};
use crate::boundary::{BoundaryAction, BoundaryState};
use crate::chrome::{self, Chrome};
use crate::dialog::{Answer, Dialog};
use crate::draw::{self, Button, ButtonKind};
use crate::index::{self, IndexRuntime, IndexState};
use crate::palette::{self, Command, PaletteAction, PaletteState};
use crate::rail::{self, RailAction, Tile};
use crate::runtime_view::{Brief, RuntimeAction, RuntimeView};
use crate::settings_view::{self, AppInfo, Section, SettingsAction, SettingsState};
use crate::strings;
use crate::summary::{Sort, SummaryState};
use crate::surface_view::SurfaceInputs;
use crate::theme::{self, radius, size};
use crate::toast::Toast;
use crate::workspace::{self, EmptyMark, Tab, WorkState};

/// The row of the index that is selected: the whole app or one module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    All,
    Module(String),
}

/// How the app is started.
pub struct Config {
    pub chrome: Chrome,
    /// Where Mortise keeps its settings. `None` keeps nothing between runs.
    pub settings_path: Option<PathBuf>,
    /// An app folder to open at start, from the command line.
    pub folder: Option<PathBuf>,
    /// Opens the folder dialog of the operating system.
    pub pick_folder: Box<dyn FnMut() -> Option<PathBuf>>,
    /// Where the secrets of the environments are kept.
    pub secrets: Secrets,
}

/// An environment that is asked for its runtime status, and what it was opened with.
struct Connected {
    environment: Environment,
    secret: Secret,
    live: LiveRuntime,
}

/// An app on the rail: its live snapshot and what is selected in it.
struct OpenApp {
    live: LiveApp,
    selection: Selection,
    tab: Tab,
    /// The text of the index's filter field.
    filter: String,
    /// The filter the selection was last matched to.
    filter_seen: String,
    /// The order of the "By module" table.
    sort: Sort,
    /// What the Boundary tab keeps.
    boundary: BoundaryState,
    /// The modules opened last, the latest first, for the palette.
    recent: Vec<String>,
    /// The Findings list is open.
    findings_open: bool,
    /// The row of the Findings list that is selected.
    finding_row: usize,
    /// The snapshot that was looked at last, to notice a newer one.
    seen: Option<Arc<AppSnapshot>>,
    /// The chosen environment, while the app is on screen and has the actuator.
    runtime: Option<Connected>,
    /// The last answer of each environment in this run, by its name.
    last_answers: BTreeMap<String, Brief>,
}

pub struct MortiseApp {
    chrome: Chrome,
    settings: Settings,
    settings_path: Option<PathBuf>,
    /// One per entry of `settings.apps`, in the same order.
    apps: Vec<OpenApp>,
    current: Option<usize>,
    /// The add panel, while it is open.
    adding: Option<AddPanel>,
    toast: Option<Toast>,
    pick_folder: Box<dyn FnMut() -> Option<PathBuf>>,
    /// True while the keyboard is in use, false once the pointer is. The focus ring
    /// is drawn only while it is true.
    keyboard: bool,
    /// The selection moved by key in this frame, so its row must be scrolled into view.
    reveal: bool,
    /// What Ctrl B chose for the index: its strip, or its full width. `None` lets
    /// the width of the window decide, which it does again after a resize.
    index_choice: Option<bool>,
    /// The width of the window in the last frame, to notice a resize.
    window_w: f32,
    /// Settings, while they are open.
    settings_open: Option<SettingsState>,
    /// The window has the platform's decorations at this moment.
    decorated: bool,
    /// The filter field takes focus in this frame.
    focus_filter: bool,
    /// The filter field had focus at the end of the last frame.
    filter_was_focused: bool,
    /// The command palette, while it is open, and the app it searches.
    palette: Option<PaletteState>,
    palette_scope: usize,
    /// A dialog that asks before the rules file is created, with the rules to write.
    dialog: Option<(Dialog, Pending)>,
    /// The dialog opened in this frame: focus goes to its button.
    dialog_opened: bool,
    secrets: Secrets,
}

impl MortiseApp {
    /// Sets up fonts and style, loads the settings and starts reading the apps.
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        crate::fonts::install(&cc.egui_ctx);
        style(&cc.egui_ctx);

        let mut toast = None;
        let settings = match config.settings_path.as_deref().map(Settings::load) {
            None => Settings::default(),
            Some(Ok(settings)) => settings,
            Some(Err(SettingsError::Invalid { path, .. })) => {
                // Keep the file that could not be read, so nothing a person wrote is lost.
                let kept = path.with_extension("invalid.json");
                let _ = std::fs::rename(&path, &kept);
                toast = Some(Toast::failed(strings::settings_invalid(&kept)));
                Settings::default()
            }
            Some(Err(error)) => {
                toast = Some(Toast::failed(format!("{error}.")));
                Settings::default()
            }
        };

        let apps = settings
            .apps
            .iter()
            .map(|entry| open_app(&cc.egui_ctx, entry))
            .collect();
        let mut app = Self {
            chrome: config.chrome,
            current: None,
            settings,
            settings_path: config.settings_path,
            apps,
            adding: None,
            toast,
            pick_folder: config.pick_folder,
            // At start, focus is on the selected row of the index.
            keyboard: true,
            reveal: false,
            index_choice: None,
            window_w: 0.0,
            settings_open: None,
            decorated: config.chrome == Chrome::System,
            focus_filter: false,
            filter_was_focused: false,
            palette: None,
            palette_scope: 0,
            dialog: None,
            dialog_opened: false,
            secrets: config.secrets,
        };
        let first = if app.apps.is_empty() { None } else { Some(0) };
        if let Some(index) = app.settings.current.or(first) {
            app.select_app(index);
        }
        if let Some(folder) = config.folder {
            let folder = std::path::absolute(&folder).unwrap_or(folder);
            // A folder given at start that is an app already is opened, not asked about.
            match app.check(&folder) {
                Checked::Known { index, .. } => app.select_app(index),
                checked => app.adding = Some(AddPanel::checked(strings::tilde(&folder), checked)),
            }
        }
        app
    }

    // --- What tests and the screens ask ---

    /// The names of the apps, in the order of the rail.
    pub fn app_names(&self) -> Vec<&str> {
        self.settings
            .apps
            .iter()
            .map(|app| app.name.as_str())
            .collect()
    }

    pub fn current(&self) -> Option<usize> {
        self.current
    }

    /// The snapshot of the current app, once it is read.
    pub fn app(&self) -> Option<Arc<AppSnapshot>> {
        match self.apps.get(self.current?)?.live.state() {
            LiveState::Ready(app) => Some(app),
            _ => None,
        }
    }

    /// True once the app on screen is watched for changes.
    pub fn is_watching(&self) -> bool {
        self.current
            .is_some_and(|current| self.apps[current].live.is_watching())
    }

    /// True once every app has been read at least once.
    pub fn is_read(&self) -> bool {
        self.apps
            .iter()
            .all(|app| !matches!(app.live.state(), LiveState::Reading))
    }

    pub fn selection(&self) -> Selection {
        self.current
            .and_then(|current| self.apps.get(current))
            .map_or(Selection::All, |app| app.selection.clone())
    }

    pub fn tab(&self) -> Tab {
        self.current
            .and_then(|current| self.apps.get(current))
            .map_or(Tab::Summary, |app| app.tab)
    }

    pub fn is_adding(&self) -> bool {
        self.adding.is_some()
    }

    /// True while the Findings list is on screen.
    pub fn findings_open(&self) -> bool {
        self.current
            .is_some_and(|current| self.apps[current].findings_open)
    }

    /// What the chosen environment of the current app answered last.
    pub fn runtime(&self) -> RuntimeState {
        self.current
            .and_then(|current| self.apps[current].runtime.as_ref())
            .map(|connected| connected.live.state())
            .unwrap_or_default()
    }

    /// The environments of the current app, as the settings have them.
    pub fn environments(&self) -> &[Environment] {
        self.current
            .map_or(&[], |current| &self.settings.apps[current].environments)
    }

    /// True when a secret is kept for an environment of the current app.
    pub fn has_secret(&self, environment: &str) -> bool {
        self.current.is_some_and(|current| {
            self.secrets
                .has(&self.settings.apps[current].root, environment)
        })
    }

    // --- Apps ---

    /// Keeps the runtime connection in step with the settings: the app on screen is
    /// asked through its chosen environment, and no other app is asked.
    fn sync_runtime(&mut self, ctx: &egui::Context) {
        for (index, (open, entry)) in self.apps.iter_mut().zip(&self.settings.apps).enumerate() {
            let installed =
                matches!(open.live.state(), LiveState::Ready(app) if app.actuator.is_some());
            let wanted = entry
                .chosen_environment()
                .filter(|_| self.current == Some(index) && installed);
            let Some(environment) = wanted else {
                open.runtime = None;
                continue;
            };
            let secret = if environment.credentials.needs_secret() {
                self.secrets.get(&entry.root, &environment.name)
            } else {
                None
            };
            let same = open.runtime.as_ref().is_some_and(|connected| {
                &connected.environment == environment && connected.secret == secret
            });
            if !same {
                let ctx = ctx.clone();
                let live = LiveRuntime::open(environment.clone(), secret.clone(), move || {
                    ctx.request_repaint();
                });
                open.runtime = Some(Connected {
                    environment: environment.clone(),
                    secret,
                    live,
                });
            }
            if let Some(connected) = &open.runtime {
                // The window is visible: this frame is drawn.
                connected.live.seen();
                if let Some(answer) = connected.live.state().answer {
                    open.last_answers
                        .insert(environment.name.clone(), Brief::of_answer(&answer));
                }
            }
        }
    }

    /// What the runtime box of the current app asked for.
    fn runtime_action(&mut self, ctx: &egui::Context, action: RuntimeAction) {
        let Some(current) = self.current else {
            return;
        };
        match action {
            RuntimeAction::Choose(name) => {
                self.settings.apps[current].environment = Some(name);
                self.save();
            }
            RuntimeAction::TryAgain => {
                if let Some(connected) = &self.apps[current].runtime {
                    connected.live.refresh();
                }
            }
            RuntimeAction::AddEnvironment => {
                self.open_settings(Section::Apps);
                if let Some(state) = &mut self.settings_open {
                    state.add_environment = true;
                }
            }
            RuntimeAction::OpenSettings => {
                self.open_settings(Section::Apps);
                let entry = &self.settings.apps[current];
                let chosen = entry.chosen_environment().map(|chosen| &chosen.name);
                let place = entry
                    .environments
                    .iter()
                    .position(|e| Some(&e.name) == chosen);
                if let Some(state) = &mut self.settings_open {
                    state.environment = place.unwrap_or(0);
                }
            }
            RuntimeAction::Copy { text, said } => {
                ctx.copy_text(text.to_owned());
                self.toast = Some(Toast::done(said));
            }
        }
    }

    fn save(&mut self) {
        let Some(path) = &self.settings_path else {
            return;
        };
        if let Err(error) = self.settings.save(path) {
            self.toast = Some(Toast::failed(format!("{error}.")));
        }
    }

    /// Shows an app. Only the app on screen is watched for changes; it is read again
    /// when it comes on screen.
    fn select_app(&mut self, index: usize) {
        if index >= self.apps.len() {
            return;
        }
        if let Some(previous) = self.current.filter(|previous| *previous != index) {
            self.apps[previous].live.watch(false);
            self.apps[index].live.refresh();
        }
        self.apps[index].live.watch(true);
        self.current = Some(index);
        self.adding = None;
        if self.settings.current != Some(index) {
            self.settings.current = Some(index);
            self.save();
        }
    }

    /// Reads a folder for the add panel.
    fn check(&self, folder: &Path) -> Checked {
        if let Some(index) = self.settings.position(folder) {
            return Checked::Known {
                index,
                name: self.settings.apps[index].name.clone(),
            };
        }
        match read_app(folder) {
            Ok(app) => Checked::Found(Box::new(app)),
            Err(error) => Checked::Failed(error),
        }
    }

    /// Opens the add panel on a folder that was chosen or dropped.
    fn open_folder(&mut self, folder: &Path) {
        let checked = self.check(folder);
        self.adding = Some(AddPanel::checked(strings::tilde(folder), checked));
    }

    /// Adds the app the panel found, shows it and says so.
    fn add_found(&mut self, ctx: &egui::Context) {
        let Some(AddPanel {
            checked: Some(Checked::Found(app)),
            runtime,
            ..
        }) = self.adding.take()
        else {
            return;
        };
        let index = self.settings.add(&app);
        // Runtime is optional: without a base URL the app is added without it.
        if let Some(form) = runtime.filter(|form| !form.base_url.trim().is_empty()) {
            let environment = form.environment();
            if environment.credentials.needs_secret() && !form.secret.is_empty() {
                self.secrets.set(&app.root, &environment.name, &form.secret);
            }
            let entry = &mut self.settings.apps[index];
            entry.environment = Some(environment.name.clone());
            entry.environments.push(environment);
        }
        self.apps.push(open_app(ctx, &self.settings.apps[index]));
        self.select_app(index);
        self.save();
        self.toast = Some(Toast::done(strings::added(&app.name, app.modules.len())));
        // Focus moves to the "All modules" row of the new app.
        self.keyboard = true;
    }

    /// The monogram the app in the add panel would get.
    fn monogram_preview(&self) -> String {
        let taken: BTreeSet<String> = self
            .settings
            .apps
            .iter()
            .map(|app| app.monogram.clone())
            .collect();
        match &self.adding {
            Some(AddPanel {
                checked: Some(Checked::Found(app)),
                ..
            }) => propose_monogram(&app.name, &taken),
            _ => String::new(),
        }
    }

    /// Takes in a newer snapshot of the current app: keeps the codes it gave, and
    /// drops a selection whose module is gone.
    fn take_in(&mut self) {
        let Some(current) = self.current else {
            return;
        };
        let open = &mut self.apps[current];
        let LiveState::Ready(app) = open.live.state() else {
            return;
        };
        if open
            .seen
            .as_ref()
            .is_some_and(|seen| Arc::ptr_eq(seen, &app))
        {
            return;
        }
        if let Selection::Module(name) = &open.selection
            && !app.modules.iter().any(|module| &module.name == name)
        {
            open.selection = Selection::All;
        }
        open.seen = Some(Arc::clone(&app));

        let entry = &mut self.settings.apps[current];
        let before = entry.codes.len();
        entry.codes.extend(app.codes());
        if entry.codes.len() != before {
            self.save();
        }
    }

    /// Switches the index of the current app between by name and problems first.
    fn toggle_order(&mut self) {
        if let Some(current) = self.current {
            let entry = &mut self.settings.apps[current];
            entry.problems_first = !entry.problems_first;
            self.save();
        }
    }

    // --- Settings ---

    fn open_settings(&mut self, section: Section) {
        self.adding = None;
        self.palette = None;
        self.settings_open = Some(SettingsState::open(section, self.current.unwrap_or(0)));
    }

    /// Applies what Settings chose for the whole of Mortise, and gives the frame
    /// of the window that follows from it.
    fn apply_general(&mut self, ctx: &egui::Context) -> Chrome {
        let general = &self.settings.general;
        let theme = match general.theme {
            ThemeChoice::System => egui::ThemePreference::System,
            ThemeChoice::Light => egui::ThemePreference::Light,
            ThemeChoice::Dark => egui::ThemePreference::Dark,
        };
        if ctx.options(|options| options.theme_preference) != theme {
            ctx.set_theme(theme);
        }
        // Text size scales every size in the interface.
        let zoom = f32::from(general.text_size) / 100.0;
        if (ctx.zoom_factor() - zoom).abs() > 0.001 {
            ctx.set_zoom_factor(zoom);
        }

        // Where Mortise draws the frame, the user can move the controls or take
        // the platform's own title bar instead.
        let chrome = match self.chrome {
            Chrome::Custom { .. } if general.system_title_bar => Chrome::System,
            Chrome::Custom { .. } => Chrome::Custom {
                controls_left: general.window_controls_left,
            },
            other => other,
        };
        let decorated = chrome == Chrome::System;
        if matches!(self.chrome, Chrome::Custom { .. }) && decorated != self.decorated {
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(decorated));
            self.decorated = decorated;
        }
        chrome
    }

    /// Forgets an app. Its folder and its files stay as they are.
    fn remove_app(&mut self, index: usize) {
        if index >= self.apps.len() {
            return;
        }
        let was_current = self.current == Some(index);
        // Mortise forgets the app, its environments and its saved credentials.
        let removed = self.settings.remove(index);
        for environment in &removed.environments {
            self.secrets.remove(&removed.root, &environment.name);
        }
        self.apps.remove(index);
        self.current = match self.current {
            Some(current) if current > index => Some(current - 1),
            Some(current) if current == index => None,
            other => other,
        };
        if was_current && !self.apps.is_empty() {
            self.select_app(0);
        }
        self.settings.current = self.current;
        self.save();
        if let Some(state) = &mut self.settings_open {
            state.app = 0;
        }
    }

    /// Opens a file or a folder in the editor the user chose.
    fn open_in_editor(&mut self, path: &Path, line: u32) {
        let command = self.settings.general.editor_command().to_owned();
        if let Err(error) = mortise_core::open_in_editor(&command, path, line) {
            self.toast = Some(Toast::failed(format!(
                "Could not open the editor: \"{error}\". Choose the editor command in Settings."
            )));
        }
    }

    // --- Palette ---

    fn open_palette(&mut self) {
        self.palette = Some(PaletteState::default());
        self.palette_scope = self.current.unwrap_or(0);
    }

    /// Runs a row of the palette, in the app the palette searched.
    fn run(&mut self, command: Command) {
        self.palette = None;
        let scope = self.palette_scope;
        // Commands about modules and findings act in the app the palette searched.
        if matches!(
            command,
            Command::Module(_)
                | Command::Surface(_)
                | Command::AllModules
                | Command::Findings
                | Command::Finding { .. }
        ) {
            if self.current != Some(scope) {
                self.select_app(scope);
            }
            self.adding = None;
        }
        match command {
            Command::Module(name) => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::Module(name);
                    self.reveal = true;
                }
            }
            Command::Surface(name) => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::Module(name);
                    open.tab = Tab::Boundary;
                    open.boundary.surface_view = true;
                    self.reveal = true;
                }
            }
            Command::AllModules => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::All;
                    open.tab = Tab::Summary;
                    open.findings_open = false;
                }
            }
            Command::Findings => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::All;
                    open.tab = Tab::Summary;
                    open.findings_open = true;
                    open.finding_row = 0;
                }
            }
            Command::Finding { module, other } => {
                if let Some(open) = self.apps.get_mut(scope) {
                    open.selection = Selection::Module(module);
                    match other {
                        Some(other) => {
                            open.tab = Tab::Boundary;
                            open.boundary.other = Some(other);
                            open.boundary.file = 0;
                        }
                        None => open.tab = Tab::Summary,
                    }
                    self.reveal = true;
                }
            }
            Command::App(index) => self.select_app(index),
            Command::AddApp => self.adding = Some(AddPanel::empty()),
            Command::Settings => self.open_settings(Section::Apps),
            Command::Theme(theme) => {
                self.settings.general.theme = theme;
                self.save();
            }
        }
    }

    // --- Rules ---

    /// Changes the rules of the current app. The first change asks before
    /// `mortise.json` is created.
    fn set_rules(&mut self, rules: Rules) {
        let Some(app) = self.app() else {
            return;
        };
        if app.root.join(mortise_core::RULES_FILE).exists() {
            self.save_rules(&rules);
        } else {
            let dialog = Dialog {
                title: strings::create_rules_title(&app.name),
                body: strings::create_rules_body(&app.name),
                confirm: strings::CREATE_RULES_FILE.to_owned(),
            };
            self.dialog = Some((dialog, Pending::CreateRules(rules)));
            self.dialog_opened = true;
        }
    }

    /// Writes the rules to `mortise.json` and reads the app again.
    fn save_rules(&mut self, rules: &Rules) {
        let (Some(app), Some(current)) = (self.app(), self.current) else {
            return;
        };
        match rules.save(&app.root) {
            Ok(()) => self.apps[current].live.refresh(),
            Err(error) => self.toast = Some(Toast::failed(strings::rules_not_saved(&error))),
        }
    }

    // --- Keys ---

    fn keys(&mut self, ui: &Ui) {
        // The focus ring shows while the keyboard is in use and goes when the pointer is.
        let (key_pressed, pointer_pressed) = ui.input(|input| {
            let key = input
                .events
                .iter()
                .any(|event| matches!(event, Event::Key { pressed: true, .. }));
            (key, input.pointer.any_pressed())
        });
        if key_pressed {
            self.keyboard = true;
        }
        if pointer_pressed {
            self.keyboard = false;
        }

        let chord = |modifiers: Modifiers, key: Key| {
            ui.input_mut(|input| input.consume_shortcut(&KeyboardShortcut::new(modifiers, key)))
        };

        // The command palette, from anywhere. The same key closes it.
        if chord(Modifiers::COMMAND, Key::K) {
            if self.palette.is_some() {
                self.palette = None;
            } else if self.dialog.is_none() {
                self.open_palette();
            }
        }
        // Add app: the folder dialog opens at once.
        if self.palette.is_none() && chord(Modifiers::COMMAND, Key::O) {
            self.pick(ui.ctx());
        }
        // Esc closes the topmost thing: the add panel, then the toast, then the text
        // of the filter, then the filter field itself.
        let filter_id = index::filter_id(ui);
        let filter_focused = ui.memory(|memory| memory.has_focus(filter_id));
        if chord(Modifiers::NONE, Key::Escape) {
            let open = self.current.map(|current| &mut self.apps[current]);
            if self.dialog.is_some() {
                self.dialog = None;
            } else if self.palette.is_some() {
                self.palette = None;
            } else if self.adding.is_some() {
                self.adding = None;
            } else if self.toast.is_some() {
                self.toast = None;
            } else if let Some(open) = open {
                // A scan of the public surface on screen: Esc stops it.
                let scanning = match &open.selection {
                    Selection::Module(name) if open.tab == Tab::Boundary => open
                        .boundary
                        .surfaces
                        .get_mut(name)
                        .filter(|surface| open.boundary.surface_view && surface.is_scanning()),
                    _ => None,
                };
                if let Some(surface) = scanning {
                    surface.stop();
                } else if !open.filter.is_empty() {
                    // Esc clears the text first and leaves the field second. egui has
                    // already taken focus from the field for this Esc: give it back.
                    open.filter.clear();
                    self.focus_filter = self.filter_was_focused;
                } else if !self.filter_was_focused && open.findings_open {
                    // Back from a second view.
                    open.findings_open = false;
                } else if !self.filter_was_focused {
                    self.settings_open = None;
                }
            } else {
                self.settings_open = None;
            }
        }
        // A dialog, the palette and the add panel hold the keys until they close.
        if self.adding.is_some() || self.dialog.is_some() || self.palette.is_some() {
            return;
        }

        // Settings, and the shortcuts, which are a section of them.
        let typing = ui.ctx().text_edit_focused();
        if chord(Modifiers::COMMAND, Key::Comma) {
            self.open_settings(Section::Apps);
        }
        if !typing && chord(Modifiers::NONE, Key::Questionmark) {
            self.open_settings(Section::Keyboard);
        }
        // Settings take the place of the workbench: its keys rest until Esc.
        if self.settings_open.is_some() {
            return;
        }

        // Collapse or open the module index. Mortise then stops deciding its width
        // until the window is resized.
        if chord(Modifiers::COMMAND, Key::B) {
            let (_, collapsed) = index_width(self.window_w, self.index_choice);
            self.index_choice = Some(!collapsed);
        }
        // Filter modules: / when not typing, or the chord for keyboards where / needs Shift.
        if chord(Modifiers::COMMAND, Key::F) || (!typing && chord(Modifiers::NONE, Key::Slash)) {
            self.index_choice = Some(false);
            self.focus_filter = true;
        }
        // Enter in the filter keeps the first match and gives the keys back to the list.
        if filter_focused && chord(Modifiers::NONE, Key::Enter) {
            ui.memory_mut(|memory| memory.surrender_focus(filter_id));
        }

        // Read the app again.
        if chord(Modifiers::COMMAND, Key::R)
            && let Some(current) = self.current
        {
            // From disk and from the actuator.
            self.apps[current].live.refresh();
            if let Some(connected) = &self.apps[current].runtime {
                connected.live.refresh();
            }
        }
        // Select app 1 to 9 on the rail.
        let digits = [
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Num5,
            Key::Num6,
            Key::Num7,
            Key::Num8,
            Key::Num9,
        ];
        for (index, key) in digits.into_iter().enumerate() {
            if chord(Modifiers::COMMAND | Modifiers::ALT, key) {
                self.select_app(index);
            }
        }

        // Single keys never act while a text field has focus.
        if ui.ctx().text_edit_focused() {
            return;
        }
        let Some(app) = self.app() else {
            return;
        };
        let Some(current_app) = self.current else {
            return;
        };
        let pressed = |key: Key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));

        // S: change the order of the index. The choice is kept per app.
        if pressed(Key::S) {
            self.toggle_order();
        }
        let problems_first = self.settings.apps[current_app].problems_first;
        let open = &mut self.apps[current_app];
        // E: the selected module in the editor.
        if pressed(Key::E)
            && let Selection::Module(name) = &open.selection
            && let Some(module) = app.module(name)
        {
            let folder = module.dir.clone();
            self.open_in_editor(&folder, 1);
            return;
        }
        let open = &mut self.apps[current_app];
        // F: the findings list of the app.
        if pressed(Key::F) {
            open.selection = Selection::All;
            open.tab = Tab::Summary;
            open.findings_open = true;
            open.finding_row = 0;
        }

        // J and K, or Down and Up: the next or previous row. Home and End: the first or last.
        // Only the rows the filter leaves.
        let modules = index::ordered(&app, &open.filter, problems_first);
        let rows = modules.len() + 1;
        let current = match &open.selection {
            Selection::All => 0,
            Selection::Module(name) => modules
                .iter()
                .position(|module| &module.name == name)
                .map_or(0, |i| i + 1),
        };
        let mut target = current;
        if pressed(Key::J) || pressed(Key::ArrowDown) {
            target = (current + 1).min(rows - 1);
        }
        if pressed(Key::K) || pressed(Key::ArrowUp) {
            target = current.saturating_sub(1);
        }
        if pressed(Key::Home) {
            target = 0;
        }
        if pressed(Key::End) {
            target = rows - 1;
        }
        if target != current {
            open.selection = match target {
                0 => Selection::All,
                row => Selection::Module(modules[row - 1].name.clone()),
            };
            self.reveal = true;
        }

        // 1 to 5: the tabs. The choice of tab is kept while moving through modules.
        for (tab, key) in Tab::ALL.into_iter().zip(digits) {
            if pressed(key) {
                open.tab = tab;
            }
        }
    }

    /// Opens the folder dialog and, if a folder is chosen, the add panel on it.
    fn pick(&mut self, ctx: &egui::Context) {
        if let Some(folder) = (self.pick_folder)() {
            self.open_folder(&folder);
            ctx.request_repaint();
        }
    }

    fn add_action(&mut self, ctx: &egui::Context, action: AddAction) {
        match action {
            AddAction::Back => self.adding = None,
            AddAction::Pick => self.pick(ctx),
            AddAction::Check => {
                let Some(path) = self.adding.as_ref().map(|panel| expand(panel.path.trim())) else {
                    return;
                };
                let checked = self.check(&path);
                if let Some(panel) = &mut self.adding {
                    panel.set_checked(checked);
                }
            }
            AddAction::Add => self.add_found(ctx),
            AddAction::Open(index) => self.select_app(index),
            AddAction::CopyCommand => {
                ctx.copy_text(strings::REQUIRE_PACKAGE.to_owned());
                self.toast = Some(Toast::done(strings::COPIED_COMMAND));
            }
            AddAction::Test => {
                if let Some(form) = self
                    .adding
                    .as_mut()
                    .and_then(|panel| panel.runtime.as_mut())
                {
                    let secret = (!form.secret.is_empty()).then(|| form.secret.clone());
                    form.start_test(secret, ctx);
                }
            }
        }
    }
}

impl eframe::App for MortiseApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let pal = theme::palette(ui.ctx().theme());
        let ctx = ui.ctx().clone();
        let mac = ctx.os() == egui::os::OperatingSystem::Mac;
        self.reveal = false;
        self.keys(ui);
        self.take_in();
        self.sync_runtime(&ctx);

        // A folder dropped on the window opens the add panel on it.
        let dropped = ui.input(|input| {
            input
                .raw
                .dropped_files
                .first()
                .map(|file| file.path().to_owned())
        });
        if let Some(folder) = dropped {
            self.open_folder(&folder);
        }

        let window = ui.max_rect();
        ui.painter().rect_filled(window, 0, pal.frame);
        // A resize hands the width of the index back to Mortise.
        if (window.width() - self.window_w).abs() > 0.5 {
            self.window_w = window.width();
            self.index_choice = None;
        }
        let chrome = self.apply_general(&ctx);
        // "System" cannot be read from egui yet, so it moves as "Full" does.
        let reduced_motion = self.settings.general.motion == mortise_core::Motion::Reduced;

        let bar = Rect::from_min_size(window.min, vec2(window.width(), size::TITLEBAR_H));
        let app_name = self
            .current
            .map(|current| self.settings.apps[current].name.clone());
        let shown_name = app_name.as_deref().unwrap_or(strings::NO_APP);
        if chrome::title_bar(ui, pal, bar, chrome, frame, Some(shown_name)) {
            self.open_palette();
        }

        let below = Rect::from_min_max(pos2(window.left(), bar.bottom()), window.max);
        let rail_rect = Rect::from_min_size(below.min, vec2(size::RAIL_W, below.height()));
        let tiles: Vec<Tile<'_>> = self
            .settings
            .apps
            .iter()
            .zip(&self.apps)
            .map(|(app, open)| Tile {
                name: &app.name,
                monogram: &app.monogram,
                problems: match open.live.state() {
                    LiveState::Ready(app) => app.findings.problems(),
                    _ => 0,
                },
            })
            .collect();
        let adding = self.adding.is_some();
        let settings_open = self.settings_open.is_some();
        let on_rail = if settings_open { None } else { self.current };
        let rail_action = rail::rail(ui, pal, rail_rect, &tiles, on_rail, adding, settings_open);

        let rest = Rect::from_min_max(pos2(rail_rect.right(), below.top()), below.max);
        let (index_w, index_collapsed) = index_width(window.width(), self.index_choice);
        let mut settings_action = None;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs() as i64);
        let mut add_action = None;
        let mut work_action = None;
        let mut toggle_order = false;
        let problems_first = self
            .current
            .is_some_and(|current| self.settings.apps[current].problems_first);
        let mut first_run_add = false;
        let monogram = self.monogram_preview();
        if let Some(panel) = &mut self.adding {
            add_action = add::add_panel(ui, pal, rest, panel, &monogram, self.keyboard, mac);
        } else if let Some(state) = &mut self.settings_open {
            // Settings take the place of the index and the workspace.
            let index_rect = Rect::from_min_size(rest.min, vec2(size::INDEX_W, rest.height()));
            let work_rect = Rect::from_min_max(pos2(index_rect.right(), rest.top()), rest.max);
            let mut infos: Vec<AppInfo<'_>> = self
                .apps
                .iter_mut()
                .map(|open| {
                    let app = match open.live.state() {
                        LiveState::Ready(app) => Some(app),
                        _ => None,
                    };
                    AppInfo {
                        versions: app.as_ref().map_or(String::new(), |app| {
                            strings::versions(app.laravel.as_ref(), &app.package)
                        }),
                        actuator: app.as_ref().is_some_and(|app| app.actuator.is_some()),
                        app_url: app.as_ref().and_then(|app| app.app_url.clone()),
                        last: &mut open.last_answers,
                    }
                })
                .collect();
            let custom_frame = matches!(self.chrome, Chrome::Custom { .. });
            let in_section = settings_view::section(
                ui,
                pal,
                work_rect,
                state,
                &mut self.settings,
                &mut infos,
                &self.secrets,
                custom_frame,
                self.keyboard,
                mac,
            );
            let in_list =
                settings_view::sections(ui, pal, index_rect, state, shown_name, self.keyboard);
            settings_action = in_section.or(in_list);
        } else if let Some(current) = self.current {
            let open = &mut self.apps[current];
            match open.live.state() {
                LiveState::Ready(app) => {
                    let index_rect = Rect::from_min_size(rest.min, vec2(index_w, rest.height()));
                    let work_rect =
                        Rect::from_min_max(pos2(index_rect.right(), rest.top()), rest.max);
                    // The workspace first: the selected row of the index is painted over its edge.
                    // The Findings list belongs to the Summary of "All modules" only.
                    if open.selection != Selection::All || open.tab != Tab::Summary {
                        open.findings_open = false;
                    }
                    let entry = &self.settings.apps[current];
                    let runtime = RuntimeView {
                        installed: app.actuator.is_some(),
                        environments: &entry.environments,
                        chosen: entry.chosen_environment(),
                        state: open
                            .runtime
                            .as_ref()
                            .map(|connected| connected.live.state())
                            .unwrap_or_default(),
                    };
                    // "checked 8 s ago" counts on while the box is on screen.
                    if runtime.brief().is_some() {
                        ctx.request_repaint_after(std::time::Duration::from_secs(1));
                    }
                    let state = WorkState {
                        summary: SummaryState {
                            sort: &mut open.sort,
                            now,
                            runtime: &runtime,
                            keyboard: self.keyboard,
                        },
                        boundary: &mut open.boundary,
                        findings_open: &mut open.findings_open,
                        finding_row: &mut open.finding_row,
                        keyboard: self.keyboard,
                        surface: surface_inputs(&app, &open.selection, &runtime),
                        mac,
                    };
                    work_action = workspace::workspace(
                        ui,
                        pal,
                        work_rect,
                        &app,
                        &open.selection,
                        &mut open.tab,
                        state,
                    );
                    // A module chosen in the workspace is selected before the index is
                    // painted, so that the joint is in the right place in this frame.
                    match &work_action {
                        Some(BoundaryAction::Select(module)) => {
                            open.selection = Selection::Module(module.clone());
                            self.reveal = true;
                        }
                        Some(BoundaryAction::ShowFindings) => {
                            open.findings_open = true;
                            open.finding_row = 0;
                        }
                        Some(BoundaryAction::ShowLink { module, other }) => {
                            open.selection = Selection::Module(module.clone());
                            open.tab = Tab::Boundary;
                            open.boundary.other = Some(other.clone());
                            open.boundary.file = 0;
                            self.reveal = true;
                        }
                        _ => {}
                    }
                    let state = IndexState {
                        selection: &mut open.selection,
                        filter: &mut open.filter,
                        focus_filter: self.focus_filter,
                        keyboard: self.keyboard,
                        reveal: self.reveal,
                        collapsed: index_collapsed,
                        problems_first,
                        rows: self.settings.general.index_rows,
                        mac,
                        runtime: runtime.brief().map(|brief| IndexRuntime {
                            brief: brief.for_header(),
                            environment: runtime.environment(),
                            checked: runtime.checked(),
                        }),
                    };
                    toggle_order = index::index(ui, pal, index_rect, &app, state);
                    // The palette offers the modules that were opened last.
                    if let Selection::Module(name) = &open.selection
                        && open.recent.first() != Some(name)
                    {
                        open.recent.retain(|recent| recent != name);
                        open.recent.insert(0, name.clone());
                        open.recent.truncate(3);
                    }
                    // A change of the filter selects its first match, so Enter opens it.
                    if open.filter != open.filter_seen {
                        open.filter_seen = open.filter.clone();
                        if !open.filter.trim().is_empty() {
                            let first = index::ordered(&app, &open.filter, problems_first)
                                .first()
                                .map(|m| m.name.clone());
                            open.selection = first.map_or(Selection::All, Selection::Module);
                        }
                    }
                }
                LiveState::Reading => {
                    ui.painter().rect_filled(rest, 0, pal.surface);
                }
                LiveState::Failed(error) => {
                    ui.painter().rect_filled(rest, 0, pal.surface);
                    let title = strings::cannot_read_title(shown_name);
                    let text = strings::read_error(&error);
                    workspace::empty_state(ui, pal, rest, EmptyMark::None, &title, &[&text], 0.0);
                }
            }
        } else {
            // First run: no apps yet.
            ui.painter().rect_filled(rest, 0, pal.surface);
            let at = workspace::empty_state(
                ui,
                pal,
                rest,
                EmptyMark::Brand,
                strings::NO_APPS_TITLE,
                &[strings::NO_APPS_BODY],
                size::CONTROL_H,
            );
            let keys = format!("{} O", strings::command_key(mac));
            let button = Button {
                label: strings::ADD_APP_FOLDER,
                kind: ButtonKind::Primary,
                keys: Some(&keys),
                ..Default::default()
            };
            first_run_add = button.show(ui, pal, at, self.keyboard).clicked();
        }

        if let Some(toast) = &mut self.toast
            && !toast.show(ui, pal, rest, reduced_motion)
        {
            self.toast = None;
        }
        // The palette, over the workbench.
        let mut palette_action = None;
        if let Some(palette) = &mut self.palette {
            let scope = self.palette_scope.min(self.apps.len().saturating_sub(1));
            let snapshot = self
                .apps
                .get(scope)
                .and_then(|open| match open.live.state() {
                    LiveState::Ready(app) => Some(app),
                    _ => None,
                });
            let names: Vec<(String, usize)> = self
                .settings
                .apps
                .iter()
                .zip(&self.apps)
                .map(|(entry, open)| {
                    let problems = match open.live.state() {
                        LiveState::Ready(app) => app.findings.problems(),
                        _ => 0,
                    };
                    (entry.name.clone(), problems)
                })
                .collect();
            let recent = self
                .apps
                .get(scope)
                .map_or(&[][..], |open| open.recent.as_slice());
            let source = palette::Source {
                scope: snapshot.as_deref().map(|app| (scope, app)),
                apps: &names,
                recent,
                mac,
            };
            let scope_name = names
                .get(scope)
                .map_or(strings::NO_APP, |(name, _)| name.as_str());
            palette_action = palette::palette(ui, pal, palette, &source, scope_name);
        }
        match palette_action {
            Some(PaletteAction::Close) => self.palette = None,
            Some(PaletteAction::NextApp) if !self.apps.is_empty() => {
                self.palette_scope = (self.palette_scope + 1) % self.apps.len();
                if let Some(palette) = &mut self.palette {
                    palette.row = 0;
                }
            }
            Some(PaletteAction::Run(command)) => self.run(command),
            _ => {}
        }

        // A dialog, over everything else.
        if let Some((dialog, _)) = &self.dialog {
            let answer = dialog.show(ui, pal, self.dialog_opened, self.keyboard);
            self.dialog_opened = false;
            match (answer, self.dialog.take_if(|_| answer.is_some())) {
                (Some(Answer::Confirm), Some((_, Pending::CreateRules(rules)))) => {
                    self.save_rules(&rules)
                }
                (Some(Answer::Confirm), Some((_, Pending::RemoveApp(index)))) => {
                    self.remove_app(index)
                }
                _ => {}
            }
        }
        if toggle_order {
            self.toggle_order();
        }
        match work_action {
            Some(BoundaryAction::SetRules(rules)) => self.set_rules(rules),
            Some(BoundaryAction::CopyPath(path)) => {
                ctx.copy_text(path);
                self.toast = Some(Toast::done(strings::COPIED_PATH));
            }
            Some(BoundaryAction::OpenInEditor { path, line }) => self.open_in_editor(&path, line),
            Some(BoundaryAction::Runtime(action)) => self.runtime_action(&ctx, action),
            Some(BoundaryAction::Copy { text, said }) => {
                ctx.copy_text(text);
                self.toast = Some(Toast::done(said));
            }
            _ => {}
        }

        // Focus is always visible while the keyboard is in use.
        if self.keyboard {
            draw::focus_ring_for_the_rest(ui, pal);
        }
        chrome::resize_edges(ui, chrome);
        self.focus_filter = false;
        let filter_id = index::filter_id(ui);
        self.filter_was_focused = ui.memory(|memory| memory.has_focus(filter_id));

        // What the pointer asked for, after everything is painted.
        match rail_action {
            Some(RailAction::Select(index)) => {
                self.settings_open = None;
                self.select_app(index);
            }
            Some(RailAction::Add) => self.adding = Some(AddPanel::empty()),
            Some(RailAction::Settings) if self.settings_open.is_some() => self.settings_open = None,
            Some(RailAction::Settings) => self.open_settings(Section::Apps),
            None => {}
        }
        match settings_action {
            Some(SettingsAction::Close) => self.settings_open = None,
            Some(SettingsAction::Changed { reread }) => {
                self.save();
                if let Some(index) = reread {
                    self.apps[index]
                        .live
                        .set_options(read_options(&self.settings.apps[index]));
                }
            }
            Some(SettingsAction::RemoveApp(index)) => {
                let name = self.settings.apps[index].name.clone();
                let dialog = Dialog {
                    title: format!("Remove {name} from Mortise?"),
                    body:
                        "The folder and its files stay as they are. Mortise forgets the app, its \
                           environments and its saved credentials."
                            .to_owned(),
                    confirm: "Remove app".to_owned(),
                };
                self.dialog = Some((dialog, Pending::RemoveApp(index)));
                self.dialog_opened = true;
            }
            None => {}
        }
        if first_run_add {
            self.adding = Some(AddPanel::empty());
        }
        if let Some(action) = add_action {
            self.add_action(&ctx, action);
        }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        let theme = if visuals.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        };
        egui::Rgba::from(theme::palette(theme).frame).to_array()
    }
}

/// Starts reading an app. The interface is asked to repaint after every read.
fn open_app(ctx: &egui::Context, entry: &AppEntry) -> OpenApp {
    let ctx = ctx.clone();
    OpenApp {
        live: LiveApp::open(
            entry.root.clone(),
            entry.codes.clone(),
            read_options(entry),
            move || {
                ctx.request_repaint();
            },
        ),
        selection: Selection::All,
        tab: Tab::Summary,
        filter: String::new(),
        filter_seen: String::new(),
        sort: Sort::default(),
        boundary: BoundaryState::default(),
        recent: Vec::new(),
        findings_open: false,
        finding_row: 0,
        seen: None,
        runtime: None,
        last_answers: BTreeMap::new(),
    }
}

/// What a scan of the public surface takes from the running app: the routes of the
/// selected module and the scheduled tasks, when the chosen environment gave them.
fn surface_inputs<'a>(
    app: &AppSnapshot,
    selection: &Selection,
    runtime: &'a RuntimeView<'a>,
) -> SurfaceInputs<'a> {
    let answered = runtime.runtime();
    let routes = match (
        selection,
        answered.and_then(|answered| answered.routes.as_deref()),
    ) {
        (Selection::Module(name), Some(routes)) => Some(
            mortise_core::routes_of(app, name, routes)
                .into_iter()
                .map(|route| route.uri.clone())
                .collect(),
        ),
        _ => None,
    };
    let scheduled = answered
        .and_then(|answered| answered.tasks.as_deref())
        .unwrap_or_default()
        .iter()
        .map(|task| {
            let schedule = match (&task.expression, task.interval_ms) {
                (Some(expression), _) => expression.clone(),
                (None, Some(interval)) => strings::every(interval),
                (None, None) => String::new(),
            };
            (task.target.clone(), schedule)
        })
        .collect();
    SurfaceInputs {
        routes,
        scheduled,
        environment: runtime.environment(),
    }
}

/// What a dialog waits to do.
enum Pending {
    /// Write the rules to a `mortise.json` that does not exist yet.
    CreateRules(Rules),
    /// Forget the app at this place on the rail.
    RemoveApp(usize),
}

/// How wide the module index is for a window of this width (section 5.14), and
/// whether it is its strip. The workspace is what the user came for, so the index
/// gives up width before the workspace does.
fn index_width(window_w: f32, choice: Option<bool>) -> (f32, bool) {
    let narrow = window_w < 1032.0;
    match choice.unwrap_or(narrow) {
        true => (size::INDEX_STRIP_W, true),
        // Opened by hand in a narrow window: the workspace gives way.
        false if narrow => (size::INDEX_W_MIN, false),
        false => {
            let left = window_w - size::RAIL_W - 760.0;
            (left.clamp(size::INDEX_W_MIN, size::INDEX_W), false)
        }
    }
}

/// What the user set for an app, as the reader takes it.
fn read_options(entry: &AppEntry) -> ReadOptions {
    ReadOptions {
        coverage_limit: entry.coverage_limit,
        coverage_report: entry.coverage_report.as_ref().map(PathBuf::from),
    }
}

/// A path as typed: `~` at its start is the home folder.
fn expand(path: &str) -> PathBuf {
    match (path.strip_prefix('~'), std::env::home_dir()) {
        (Some(rest), Some(home)) if rest.is_empty() || rest.starts_with(['/', '\\']) => {
            home.join(rest.trim_start_matches(['/', '\\']))
        }
        _ => PathBuf::from(path),
    }
}

/// What egui draws by itself (tooltips, text fields, scrollbars) takes the Slate colours too.
fn style(ctx: &egui::Context) {
    // Text size is a setting of Mortise, not a zoom key.
    ctx.options_mut(|options| options.zoom_with_keyboard = false);
    for theme in [Theme::Light, Theme::Dark] {
        let pal = theme::palette(theme);
        ctx.style_mut_of(theme, |style| {
            // Nothing moves unless the user or a command caused it.
            style.animation_time = 0.0;
            style.visuals.panel_fill = pal.surface;
            // A tooltip: key fill, key-fg text, no outline, no shadow.
            style.visuals.window_fill = pal.key;
            style.visuals.window_stroke = egui::Stroke::NONE;
            style.visuals.window_corner_radius = radius::CONTROL.into();
            style.visuals.popup_shadow = egui::Shadow::NONE;
            style.visuals.widgets.noninteractive.fg_stroke.color = pal.ink;
            // A placeholder.
            style.visuals.weak_text_color = Some(pal.ink_3);
            // A text field: ink caret, a line colour behind selected text.
            style.visuals.extreme_bg_color = pal.surface;
            style.visuals.text_cursor.stroke = egui::Stroke::new(1.5, pal.ink);
            style.visuals.selection.bg_fill = pal.line;
            style.visuals.selection.stroke = egui::Stroke::new(1.0, pal.ink);
            // A scrollbar (C16): a 6 px thumb that floats over the content and is
            // always there while the content is longer than its area.
            let scroll = &mut style.spacing.scroll;
            *scroll = egui::style::ScrollStyle::floating();
            scroll.bar_width = size::SCROLLBAR_THUMB_W;
            scroll.floating_width = size::SCROLLBAR_THUMB_W;
            scroll.floating_allocated_width = 0.0;
            scroll.handle_min_length = size::SCROLLBAR_THUMB_MIN;
            scroll.bar_outer_margin = 2.0;
            scroll.foreground_color = true;
            scroll.dormant_background_opacity = 0.0;
            scroll.active_background_opacity = 0.0;
            scroll.interact_background_opacity = 0.0;
            scroll.dormant_handle_opacity = 1.0;
            scroll.active_handle_opacity = 1.0;
            scroll.interact_handle_opacity = 1.0;
            style.visuals.widgets.inactive.fg_stroke.color = pal.line_2;
            style.visuals.widgets.hovered.fg_stroke.color = pal.ink_2;
            style.visuals.widgets.active.fg_stroke.color = pal.ink;
        });
    }
}

/// Starts the app. `folder` is an app folder to open, if any.
pub fn run(chrome: Chrome, folder: Option<PathBuf>) -> eframe::Result {
    let viewport = chrome.viewport(
        egui::ViewportBuilder::default()
            .with_title(strings::BRAND)
            .with_app_id("mortise")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([size::WINDOW_MIN_W, size::WINDOW_MIN_H])
            .with_drag_and_drop(true),
    );
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        strings::BRAND,
        options,
        Box::new(move |cc| {
            let config = Config {
                chrome,
                settings_path: Settings::default_path(),
                folder,
                // The one native dialog in the app.
                pick_folder: Box::new(|| rfd::FileDialog::new().pick_folder()),
                // For work on Mortise itself: `MORTISE_SECRETS=memory` keeps secrets
                // out of the credential store of the computer.
                secrets: match std::env::var("MORTISE_SECRETS").as_deref() {
                    Ok("memory") => Secrets::memory(),
                    _ => Secrets::os(),
                },
            };
            Ok(Box::new(MortiseApp::new(cc, config)))
        }),
    )
}
