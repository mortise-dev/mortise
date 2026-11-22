//! The app: its state, the keys, and the three columns of the workbench.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Event, Key, KeyboardShortcut, Modifiers, Rect, Theme, Ui, pos2, vec2};
use mortise_core::{
    AppEntry, AppSnapshot, LiveApp, LiveState, Settings, SettingsError, propose_monogram, read_app,
};

use crate::add::{self, AddAction, AddPanel, Checked};
use crate::chrome::{self, Chrome};
use crate::draw::{Button, ButtonKind};
use crate::index::{self, IndexState};
use crate::rail::{self, RailAction, Tile};
use crate::strings;
use crate::theme::{self, radius, size};
use crate::toast::Toast;
use crate::workspace::{self, EmptyMark, Tab};

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
}

/// An app on the rail: its live snapshot and what is selected in it.
struct OpenApp {
    live: LiveApp,
    selection: Selection,
    tab: Tab,
    /// The snapshot that was looked at last, to notice a newer one.
    seen: Option<Arc<AppSnapshot>>,
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

    // --- Apps ---

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
            ..
        }) = self.adding.take()
        else {
            return;
        };
        let index = self.settings.add(&app);
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

        // Add app: the folder dialog opens at once.
        if chord(Modifiers::COMMAND, Key::O) {
            self.pick(ui.ctx());
        }
        // Esc closes the topmost thing: the add panel, then the toast.
        if chord(Modifiers::NONE, Key::Escape) {
            if self.adding.is_some() {
                self.adding = None;
            } else {
                self.toast = None;
            }
        }
        if self.adding.is_some() {
            return;
        }

        // Read the app again.
        if chord(Modifiers::COMMAND, Key::R)
            && let Some(current) = self.current
        {
            self.apps[current].live.refresh();
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
        let Some(open) = self.current.map(|current| &mut self.apps[current]) else {
            return;
        };
        let pressed = |key: Key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));

        // J and K, or Down and Up: the next or previous row. Home and End: the first or last.
        let rows = app.modules.len() + 1;
        let current = match &open.selection {
            Selection::All => 0,
            Selection::Module(name) => app
                .modules
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
                row => Selection::Module(app.modules[row - 1].name.clone()),
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

        let bar = Rect::from_min_size(window.min, vec2(window.width(), size::TITLEBAR_H));
        let app_name = self
            .current
            .map(|current| self.settings.apps[current].name.clone());
        let shown_name = app_name.as_deref().unwrap_or(strings::NO_APP);
        chrome::title_bar(ui, pal, bar, self.chrome, frame, Some(shown_name));

        let below = Rect::from_min_max(pos2(window.left(), bar.bottom()), window.max);
        let rail_rect = Rect::from_min_size(below.min, vec2(size::RAIL_W, below.height()));
        let tiles: Vec<Tile<'_>> = self
            .settings
            .apps
            .iter()
            .map(|app| Tile {
                name: &app.name,
                monogram: &app.monogram,
            })
            .collect();
        let adding = self.adding.is_some();
        let rail_action = rail::rail(ui, pal, rail_rect, &tiles, self.current, adding);

        let rest = Rect::from_min_max(pos2(rail_rect.right(), below.top()), below.max);
        let mut add_action = None;
        let mut first_run_add = false;
        let monogram = self.monogram_preview();
        if let Some(panel) = &mut self.adding {
            add_action = add::add_panel(ui, pal, rest, panel, &monogram, self.keyboard, mac);
        } else if let Some(current) = self.current {
            let open = &mut self.apps[current];
            match open.live.state() {
                LiveState::Ready(app) => {
                    let index_rect =
                        Rect::from_min_size(rest.min, vec2(size::INDEX_W, rest.height()));
                    let work_rect =
                        Rect::from_min_max(pos2(index_rect.right(), rest.top()), rest.max);
                    // The workspace first: the selected row of the index is painted over its edge.
                    workspace::workspace(ui, pal, work_rect, &app, &open.selection, &mut open.tab);
                    let state = IndexState {
                        selection: &mut open.selection,
                        keyboard: self.keyboard,
                        reveal: self.reveal,
                    };
                    index::index(ui, pal, index_rect, &app, state);
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
            && !toast.show(ui, pal, rest)
        {
            self.toast = None;
        }
        chrome::resize_edges(ui, self.chrome);

        // What the pointer asked for, after everything is painted.
        match rail_action {
            Some(RailAction::Select(index)) => self.select_app(index),
            Some(RailAction::Add) => self.adding = Some(AddPanel::empty()),
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
        live: LiveApp::open(entry.root.clone(), entry.codes.clone(), move || {
            ctx.request_repaint();
        }),
        selection: Selection::All,
        tab: Tab::Summary,
        seen: None,
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
            };
            Ok(Box::new(MortiseApp::new(cc, config)))
        }),
    )
}
