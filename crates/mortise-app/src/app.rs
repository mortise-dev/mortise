//! The app: its state, the keys, and the three columns of the workbench.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Event, Key, KeyboardShortcut, Modifiers, Rect, Theme, Ui, pos2, vec2};
use mortise_core::{
    Action, Ask, Catalog, Environment, FileOutcome, LangReport, LiveRuntime, MODULE_MAKE,
    ModuleState, PackageData, ReadOptions, Rules, RunEnd, RunnerKind, Runs, RuntimeState, Secret,
    Secrets, Target, TestReport, ThemeChoice, Version, gives_data,
};
use mortise_core::{
    AppEntry, AppSnapshot, LiveApp, LiveState, ReadCache, Settings, SettingsError,
    propose_monogram, read_app_cached,
};

use crate::actions_view::{self, Act, ActionsInputs, ActionsState};
use crate::add::{self, AddAction, AddPanel, Checked};
use crate::boundary::{BoundaryAction, BoundaryState};
use crate::chrome::{self, Chrome};
use crate::dialog::{Answer, Dialog, Typed};
use crate::draw::{self, Button, ButtonKind};
use crate::generate_view::{
    CatalogRead, GenerateAsk, GenerateInputs, GenerateResult, GenerateRun, GenerateState, RunPlace,
    TakenOut, typed_names,
};
use crate::index::{self, IndexRuntime, IndexState};
use crate::palette::{self, Command, PaletteAction, PaletteState};
use crate::rail::{self, RailAction, Tile};
use crate::runner_view::{self, AppRuns, DrawerAction, RunnerView, Shown};
use crate::runtime_view::{Brief, RuntimeAction, RuntimeView};
use crate::settings_view::{self, AppInfo, Section, SettingsAction, SettingsState};
use crate::strings;
use crate::summary::{Sort, SummaryState};
use crate::surface_view::SurfaceInputs;
use crate::theme::{self, radius, size};
use crate::toast::{Toast, ToastAction};
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
    /// How long the window waits for a folder to be read before it goes on without
    /// the result. A folder that is read within this time shows in the same frame.
    pub quick_read: std::time::Duration,
    /// Where Mortise keeps the history of runs. `None` keeps no history.
    pub data_dir: Option<PathBuf>,
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
    /// The test of how PHP runs for the app: the one that runs and the last one, in
    /// this run (design D8). It stays when Settings close.
    php_test: crate::php_settings::PhpTest,
    /// When the app was opened, to show how far its first read is only when it takes long.
    since: std::time::Instant,
    /// The command that runs or ran last, and the history of runs.
    runs: AppRuns,
    /// What the Actions tab keeps: the selected action and the options.
    actions: ActionsState,
    /// The action that runs, when it was started from the Actions tab or asked about
    /// again: its end gets a toast (design D6). With it, the modules that were on
    /// when it started, for the toast of a fresh (design D7).
    action_run: Option<(Act, usize)>,
    /// What the Generate tab keeps: the picker, the form and the last result.
    generate: GenerateState,
    /// The generator that runs, with the modules the app had when it started.
    generate_run: Option<Generating>,
    /// After `module:make`: the module to select once a snapshot has it.
    select_new: Option<SelectNew>,
    /// The data the package gave in this session (P4, P8). It goes into every read
    /// of the app, so the snapshot holds it (design D7).
    package: PackageData,
    /// The read of package data that runs, if one does (design D3).
    reading: Option<PackageRead>,
    /// Why the last read of the migration state gave nothing, with its target. The
    /// state box says so; the state stays as it was (design D2).
    migration_error: Option<(Target, String)>,
    /// The generators read from the app (P3, design D4).
    catalog: AppCatalog,
}

impl OpenApp {
    /// The label of the command that runs in the app, a run or a read. While one
    /// runs, every run button of the app is off.
    fn busy(&self) -> Option<&str> {
        busy(&self.runs, self.reading.as_ref())
    }
}

/// [`OpenApp::busy`] from its two fields, for code that borrows the others.
fn busy<'a>(runs: &'a AppRuns, reading: Option<&'a PackageRead>) -> Option<&'a str> {
    runs.running()
        .or_else(|| reading.map(|read| read.label.as_str()))
}

/// The generators read from a 13.4 app (design D4): in this session, or from the
/// cache in Mortise's data folder.
#[derive(Debug, Default)]
struct AppCatalog {
    /// The catalog, when it was read in ms since 1970, and the raw package version
    /// it was read with.
    read: Option<(Catalog, i64, String)>,
    /// Why the last read gave no catalog.
    error: Option<String>,
    /// The package version the cache was last looked up for.
    looked_up: Option<String>,
}

impl AppCatalog {
    /// The read catalog and its time, while the app is a 13.4 app with the package
    /// version it was read with.
    fn read_for(&self, package: &Version) -> Option<(&Catalog, i64)> {
        let (catalog, read_ms, raw) = self.read.as_ref()?;
        (gives_data(package) && *raw == package.raw).then_some((catalog, *read_ms))
    }

    /// The generators every screen uses: the read ones, else the ones shipped for
    /// the app's package version.
    fn in_use(&self, package: &Version) -> &Catalog {
        self.read_for(package)
            .map_or_else(|| Catalog::shipped(package), |(catalog, _)| catalog)
    }

    /// What the picker says about the read, on a 13.4 app only.
    fn picker(&self, package: &Version, reading: bool) -> Option<CatalogRead<'_>> {
        if !gives_data(package) {
            return None;
        }
        Some(if reading {
            CatalogRead::Reading
        } else if let Some((_, read_ms)) = self.read_for(package) {
            CatalogRead::Read(read_ms)
        } else if let Some(error) = &self.error {
            CatalogRead::Failed(error)
        } else {
            CatalogRead::NotRead
        })
    }

    /// Takes the catalog from the cache, once for each package version the app has.
    fn look_up(&mut self, data_dir: Option<&Path>, root: &Path, package: &Version) {
        if self.looked_up.as_ref() == Some(&package.raw) {
            return;
        }
        self.looked_up = Some(package.raw.clone());
        if self.read_for(package).is_some() || !gives_data(package) {
            return;
        }
        if let Some((catalog, read_ms)) =
            data_dir.and_then(|dir| mortise_core::catalog_cache::load(dir, root, package))
        {
            self.read = Some((catalog, read_ms, package.raw.clone()));
        }
    }
}

/// A read of package data: a command that only gives data, whose result goes where
/// the data is shown, not to the drawer (design D3).
struct PackageRead {
    read: mortise_core::Read,
    /// The artisan words, for the tooltip of a busy app.
    label: String,
    purpose: ReadFor,
}

/// What a read is for.
#[allow(dead_code)] // Each variant is started by the task of its data (15.3 to 15.5).
enum ReadFor {
    /// `module:generators --json` (P3, design D4), started while the app had this
    /// package version.
    Catalog { package: Version },
    /// A generator with `--dry-run --json` (P2, design D5): the words of the form,
    /// and the runner and working folder it started with.
    DryRun { words: Vec<String>, place: RunPlace },
    /// `module:migrate-status --json` (P4, design D7), on a connection: its
    /// `--database`, empty for the default.
    Migrations { target: Target, connection: String },
}

/// A generator that runs, and what its end needs.
struct Generating {
    run: GenerateRun,
    /// The modules the app had when it started.
    before: BTreeSet<String>,
    /// The line that runs carries `--force`, in the runner's words or in the run's.
    forced: bool,
    /// The generator and the module of the form it was started from: its result
    /// shows only while that form is still on screen.
    form: Option<(String, Option<String>)>,
}

/// The module `module:make` created, to select once the app is read with it
/// (design D6): the first module that was not there before, else a typed name.
struct SelectNew {
    before: BTreeSet<String>,
    names: Vec<String>,
}

impl SelectNew {
    fn find(&self, app: &AppSnapshot) -> Option<String> {
        // ASCII case ignored, and "-" and "_" left out, as the package makes the name studly.
        let key = |name: &str| {
            name.chars()
                .filter(|c| *c != '-' && *c != '_')
                .map(|c| c.to_ascii_lowercase())
                .collect::<String>()
        };
        app.modules
            .iter()
            .find(|module| !self.before.contains(&module.name))
            .or_else(|| {
                app.modules
                    .iter()
                    .find(|module| self.names.iter().any(|name| key(name) == key(&module.name)))
            })
            .map(|module| module.name.clone())
    }
}

/// A folder that is read for the add panel.
struct Reading {
    result: std::sync::mpsc::Receiver<Checked>,
    progress: Arc<mortise_core::Progress>,
    since: std::time::Instant,
}

/// A progress bar is drawn only after this time, so that quick work does not flash.
const SHOW_PROGRESS_AFTER: std::time::Duration = std::time::Duration::from_millis(300);

pub struct MortiseApp {
    chrome: Chrome,
    settings: Settings,
    settings_path: Option<PathBuf>,
    /// One per entry of `settings.apps`, in the same order.
    apps: Vec<OpenApp>,
    current: Option<usize>,
    /// The add panel, while it is open.
    adding: Option<AddPanel>,
    /// The folder of the add panel, while it is read on another thread.
    reading: Option<Reading>,
    quick_read: std::time::Duration,
    /// What the operating system said about motion, and when it was asked.
    system_motion: Option<(bool, std::time::Instant)>,
    /// The menu bar of macOS, in the real window.
    #[cfg(target_os = "macos")]
    menu: Option<crate::menu::MenuBar>,
    toast: Option<Toast>,
    pick_folder: Box<dyn FnMut() -> Option<PathBuf>>,
    /// True while the keyboard is in use, false once the pointer is. The focus ring
    /// is drawn only while it is true.
    keyboard: bool,
    /// The selection moved by key in this frame, so its row must be scrolled into view.
    reveal: bool,
    /// The selection moved in the workspace: its row is scrolled into view in the next frame.
    reveal_next: bool,
    /// Where the workspace was in the last frame, to tell which region holds focus.
    work_rect: Rect,
    /// Focus is on its way from the index into the workspace: 2 when the tab row
    /// has it, 1 when it moved on to the first control of the tab.
    entering_workspace: u8,
    /// What Ctrl B chose for the index: its strip, or its full width. `None` lets
    /// the width of the window decide, which it does again after a resize.
    index_choice: Option<bool>,
    /// The width of the window in the last frame, to notice a resize.
    window_w: f32,
    /// Settings, while they are open.
    settings_open: Option<SettingsState>,
    /// Esc closes Settings once they are drawn.
    close_settings: bool,
    /// The window has the platform's decorations at this moment.
    decorated: bool,
    /// The filter field takes focus in this frame.
    focus_filter: bool,
    /// The filter field had focus at the end of the last frame.
    filter_was_focused: bool,
    /// A text field of Settings had focus at the end of the last frame.
    settings_field_was_focused: bool,
    /// The command palette, while it is open, and the app it searches.
    palette: Option<PaletteState>,
    palette_scope: usize,
    /// A dialog that asks before the rules file is created, with the rules to write.
    dialog: Option<(Dialog, Pending)>,
    /// The dialog opened in this frame: focus goes to its button.
    dialog_opened: bool,
    secrets: Secrets,
    /// The commands that run, one per app at most, for the whole window.
    runs: Runs,
    /// The runner drawer, the same for every app.
    runner: RunnerView,
    /// Where the history of runs is kept.
    data_dir: Option<PathBuf>,
    /// Where the drawer was in the last frame, to tell which region holds focus.
    drawer_rect: Rect,
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
            .map(|entry| open_app(&cc.egui_ctx, entry, config.data_dir.as_deref()))
            .collect();
        let mut app = Self {
            chrome: config.chrome,
            current: None,
            settings,
            settings_path: config.settings_path,
            apps,
            adding: None,
            reading: None,
            quick_read: config.quick_read,
            system_motion: None,
            #[cfg(target_os = "macos")]
            menu: None,
            toast,
            pick_folder: config.pick_folder,
            // At start, focus is on the selected row of the index.
            keyboard: true,
            reveal: false,
            reveal_next: false,
            work_rect: Rect::NOTHING,
            entering_workspace: 0,
            index_choice: None,
            window_w: 0.0,
            settings_open: None,
            close_settings: false,
            decorated: config.chrome == Chrome::System,
            focus_filter: false,
            filter_was_focused: false,
            settings_field_was_focused: false,
            palette: None,
            palette_scope: 0,
            dialog: None,
            dialog_opened: false,
            secrets: config.secrets,
            runs: Runs::new(),
            runner: RunnerView::default(),
            data_dir: config.data_dir,
            drawer_rect: Rect::NOTHING,
        };
        let first = if app.apps.is_empty() { None } else { Some(0) };
        if let Some(index) = app.settings.current.or(first) {
            app.select_app(index);
        }
        if let Some(folder) = config.folder {
            let folder = std::path::absolute(&folder).unwrap_or(folder);
            // A folder given at start that is an app already is opened, not asked about.
            match app.settings.position(&folder) {
                Some(index) => app.select_app(index),
                None => app.open_folder(&cc.egui_ctx, &folder),
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
            .map(AppEntry::display_name)
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
            // Another app: its Actions tab starts again on Migrate (design D4).
            self.apps[index].actions = ActionsState::default();
        }
        self.apps[index].live.watch(true);
        self.current = Some(index);
        self.adding = None;
        if self.settings.current != Some(index) {
            self.settings.current = Some(index);
            self.save();
        }
    }

    /// Reads a folder for the add panel, on another thread. A folder that is read
    /// in a moment gives its result at once; a larger one gives `None`, and the
    /// result arrives in a later frame.
    fn check(&mut self, ctx: &egui::Context, folder: &Path) -> Option<Checked> {
        self.reading = None;
        if let Some(index) = self.settings.position(folder) {
            return Some(Checked::Known {
                index,
                name: self.settings.apps[index].display_name().to_owned(),
            });
        }
        let mut cache = ReadCache::default();
        let progress = cache.progress();
        let (sender, result) = std::sync::mpsc::channel();
        let (root, ctx) = (folder.to_owned(), ctx.clone());
        std::thread::Builder::new()
            .name("mortise-add-reader".to_owned())
            .spawn(move || {
                let read =
                    read_app_cached(&root, &BTreeMap::new(), &ReadOptions::default(), &mut cache);
                let _ = sender.send(match read {
                    Ok(app) => Checked::Found(Box::new(app)),
                    Err(error) => Checked::Failed(error),
                });
                ctx.request_repaint();
            })
            .expect("a thread can be started");
        match result.recv_timeout(self.quick_read) {
            Ok(checked) => Some(checked),
            Err(_) => {
                self.reading = Some(Reading {
                    result,
                    progress,
                    since: std::time::Instant::now(),
                });
                None
            }
        }
    }

    /// Opens the add panel on a folder that was chosen or dropped.
    fn open_folder(&mut self, ctx: &egui::Context, folder: &Path) {
        let path = strings::tilde(folder);
        self.adding = Some(match self.check(ctx, folder) {
            Some(checked) => AddPanel::checked(path, checked),
            None => AddPanel::reading(path),
        });
    }

    /// Takes in the folder of the add panel once it is read.
    fn take_in_folder(&mut self, ctx: &egui::Context) {
        let Some(reading) = &self.reading else {
            return;
        };
        match reading.result.try_recv() {
            Ok(checked) => {
                self.reading = None;
                if let Some(panel) = self.adding.as_mut().filter(|panel| panel.reading) {
                    panel.set_checked(checked);
                }
            }
            // The bar moves while the files are read.
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.reading = None,
        }
        if self.adding.is_none() {
            self.reading = None;
        }
    }

    /// Adds the app the panel found, shows it and says so.
    fn add_found(&mut self, ctx: &egui::Context) {
        let Some(AddPanel {
            checked: Some(Checked::Found(app)),
            runtime,
            php,
            php_test,
            ..
        }) = self.adding.take()
        else {
            return;
        };
        let index = self.settings.add(&app);
        // Add saves the runner the field shows (design D8), read from its text in
        // case the field still has focus.
        if let Some(form) = &php {
            self.settings.apps[index].php.runner = Some(RunnerKind::from_line(&form.line));
        }
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
        let mut open = open_app(ctx, &self.settings.apps[index], self.data_dir.as_deref());
        // The test is kept in memory, as Settings shows it. One that runs still lands.
        open.php_test = php_test;
        self.apps.push(open);
        self.settings_open = None;
        self.select_app(index);
        self.save();
        self.toast = Some(Toast::done(strings::added(
            self.settings.apps[index].display_name(),
            app.modules.len(),
        )));
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

    /// Gives the app the menu bar of macOS. Only the real window has one.
    #[cfg(target_os = "macos")]
    pub fn with_menu_bar(mut self, ctx: &egui::Context) -> Self {
        self.menu = crate::menu::MenuBar::install(ctx);
        self
    }

    /// What was chosen in the menu bar. A Settings field in use is left first, so
    /// that its edit applies in this frame, before the choice is done at its end.
    #[cfg(target_os = "macos")]
    fn menu_chosen(&self, ctx: &egui::Context) -> Vec<crate::menu::MenuAction> {
        let chosen = self
            .menu
            .as_ref()
            .map(|menu| menu.chosen())
            .unwrap_or_default();
        if !chosen.is_empty() && self.settings_open.is_some() {
            ctx.memory_mut(egui::Memory::stop_text_input);
        }
        chosen
    }

    /// Does what was chosen in the menu bar: the same commands as the palette.
    #[cfg(target_os = "macos")]
    fn menu_actions(&mut self, ctx: &egui::Context, chosen: Vec<crate::menu::MenuAction>) {
        use crate::menu::MenuAction;
        // The choices run after this frame was painted: the next frame shows them.
        if !chosen.is_empty() {
            ctx.request_repaint();
        }
        for action in chosen {
            self.palette_scope = self.current.unwrap_or(0);
            match action {
                MenuAction::Settings => self.open_settings(Section::Apps),
                MenuAction::Shortcuts => self.open_settings(Section::Keyboard),
                // A dialog holds the app until it closes, as for the key.
                MenuAction::AddApp if self.dialog.is_none() => self.pick(ctx),
                MenuAction::AddApp => {}
                MenuAction::Palette => self.open_palette(),
                MenuAction::AllModules => self.run(Command::AllModules),
                MenuAction::Findings => self.run(Command::Findings),
                MenuAction::Theme(theme) => self.run(Command::Theme(theme)),
                MenuAction::ToggleIndex => {
                    let set = f32::from(self.settings.general.index_width);
                    let (_, collapsed) = index_width(self.window_w, self.index_choice, set);
                    self.index_choice = Some(!collapsed);
                }
                MenuAction::ReadAgain => {
                    if let Some(current) = self.current {
                        self.apps[current].live.refresh();
                        if let Some(connected) = &self.apps[current].runtime {
                            connected.live.refresh();
                        }
                    }
                }
            }
        }
    }

    /// What the operating system says about motion, asked at most once a second.
    fn system_reduces_motion(&mut self) -> bool {
        let fresh = |(_, at): &(bool, std::time::Instant)| at.elapsed().as_secs() < 1;
        match self.system_motion.filter(fresh) {
            Some((reduced, _)) => reduced,
            None => {
                let reduced = system_reduces_motion();
                self.system_motion = Some((reduced, std::time::Instant::now()));
                reduced
            }
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

    /// Moves an app to another place on the rail. The app on screen stays on screen.
    fn move_app(&mut self, from: usize, to: usize) {
        if from >= self.apps.len() || to >= self.apps.len() || from == to {
            return;
        }
        let entry = self.settings.apps.remove(from);
        self.settings.apps.insert(to, entry);
        let open = self.apps.remove(from);
        self.apps.insert(to, open);
        // Where a place that was held before the move is now.
        let moved = |place: usize| {
            if place == from {
                to
            } else if from < place && place <= to {
                place - 1
            } else if to <= place && place < from {
                place + 1
            } else {
                place
            }
        };
        self.current = self.current.map(moved);
        self.palette_scope = moved(self.palette_scope);
        self.settings.current = self.current;
        if let Some(state) = &mut self.settings_open {
            state.app = moved(state.app);
        }
        self.save();
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
        // A command that goes somewhere leaves Settings.
        if !matches!(
            command,
            Command::Settings | Command::PhpSettings | Command::Theme(_) | Command::Runner
        ) {
            self.settings_open = None;
        }
        let scope = self.palette_scope;
        // Commands about modules and findings act in the app the palette searched.
        if matches!(
            command,
            Command::Module(_)
                | Command::Surface(_)
                | Command::AllModules
                | Command::Findings
                | Command::Finding { .. }
                | Command::Action { .. }
                | Command::Generate { .. }
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
                            open.boundary.surface_view = false;
                            open.boundary.other = Some(other);
                            open.boundary.file = 0;
                        }
                        None => open.tab = Tab::Summary,
                    }
                    self.reveal = true;
                }
            }
            Command::Action { module, action } => {
                if self.current != Some(scope) {
                    self.select_app(scope);
                }
                self.open_action(module, action);
            }
            Command::Generate { module, generator } => self.open_generate(module, &generator),
            Command::App(index) => self.select_app(index),
            Command::AddApp => self.adding = Some(AddPanel::empty()),
            Command::Settings => self.open_settings(Section::Apps),
            Command::PhpSettings => self.open_settings(Section::Php),
            Command::Runner => self.runner.toggle(),
            Command::Theme(theme) => {
                self.settings.general.theme = theme;
                self.save();
            }
        }
    }

    /// Changes the package data of the app at `index` and reads the app again, so
    /// that its snapshot holds the new data (design D7).
    fn set_package(&mut self, index: usize, change: impl FnOnce(&mut PackageData)) {
        let (Some(open), Some(entry)) = (self.apps.get_mut(index), self.settings.apps.get(index))
        else {
            return;
        };
        change(&mut open.package);
        open.live.set_options(read_options(entry, &open.package));
    }

    /// Starts a read of package data in the app at `index` (design D3). It does not
    /// open the drawer. A read that cannot start ends at once with its error.
    fn start_read(&mut self, index: usize, args: Vec<String>, purpose: ReadFor) {
        let (Some(open), Some(entry)) = (self.apps.get(index), self.settings.apps.get(index))
        else {
            return;
        };
        if open.busy().is_some() {
            return;
        }
        match mortise_core::Read::start(&self.runs, &entry.root, &entry.php, &args) {
            Ok(read) => {
                self.apps[index].reading = Some(PackageRead {
                    read,
                    label: args.join(" "),
                    purpose,
                });
            }
            Err(error) => self.read_ended(index, purpose, Err(error)),
        }
    }

    /// What a read gave, or why it did not start (design D2, D3).
    fn read_ended(
        &mut self,
        index: usize,
        purpose: ReadFor,
        end: Result<mortise_core::ReadEnd, mortise_core::StartError>,
    ) {
        match purpose {
            ReadFor::Catalog { package } => self.catalog_read(index, &package, end),
            ReadFor::DryRun { words, place } => self.dry_run_read(index, words, place, end),
            ReadFor::Migrations { target, connection } => {
                self.migrations_read(index, target, &connection, end)
            }
        }
    }

    /// Starts a read of the migration state of `target` with `words`, on the
    /// connection their `--database` names (design D7).
    fn read_migrations(&mut self, index: usize, target: Target, words: Vec<String>) {
        let connection = mortise_core::Options::from_args(&words).database;
        self.start_read(index, words, ReadFor::Migrations { target, connection });
    }

    /// Takes in a read of the migration state of `target` on `connection` (design
    /// D7): its document merges into the app's package data. Otherwise the state box
    /// says why, and the state stays as it was (design D2).
    fn migrations_read(
        &mut self,
        index: usize,
        target: Target,
        connection: &str,
        end: Result<mortise_core::ReadEnd, mortise_core::StartError>,
    ) {
        let status = match end {
            Ok(end) => end
                .document::<mortise_core::MigrationStatus>()
                .map_err(|error| strings::no_document(&end, &error)),
            Err(error) => Err(strings::start_reason(&error)),
        };
        let modules_on: Vec<String> = self.snapshot(index).map_or_else(Vec::new, |app| {
            app.modules
                .iter()
                .filter(|module| module.state == ModuleState::On)
                .map(|module| module.name.clone())
                .collect()
        });
        let Some(open) = self.apps.get_mut(index) else {
            return;
        };
        let status = match status {
            Ok(status) => status,
            Err(reason) => {
                open.migration_error = Some((target, reason));
                return;
            }
        };
        open.migration_error = None;
        let modules_on: Vec<&str> = modules_on.iter().map(String::as_str).collect();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_millis() as i64);
        self.set_package(index, |package| {
            package.merge_migrations(status, &target, connection, &modules_on, now_ms);
        });
    }

    /// After a run of the app at `index` ended, with these words, from wherever it
    /// started (design D7). A migrate, roll back, reset, refresh or fresh, whatever its
    /// exit code, forgets the migration state it may have changed: the box then reads
    /// "Not read yet." again. A delete that exited with 0 forgets its module.
    fn after_run(&mut self, index: usize, args: &[String]) {
        self.documents_ended(index, args);
        let Some(open) = self.apps.get(index) else {
            return;
        };
        let forget = match (destructive_words(args), open.runs.ended()) {
            (
                Some((Action::DeleteModule, Target::Module(name))),
                Some(runner_view::RunOver {
                    end: RunEnd::Exited { code: Some(0), .. },
                    ..
                }),
            ) => Some(Target::Module(name)),
            _ => mortise_core::migrations_changed_by(args),
        };
        let Some(target) = forget else {
            return;
        };
        // The app is read again only when something goes.
        if open.package.clone().forget_migrations(&target) {
            self.set_package(index, |package| {
                package.forget_migrations(&target);
            });
        }
    }

    /// The end of `module:generators --json` (design D4): the catalog replaces the
    /// shipped one for the app and goes into the cache, or the picker says why not.
    /// `package` is the version the app had when the read started. When the app's
    /// version changed meanwhile, the answer may be of either version: it is dropped,
    /// and the picker shows the shipped catalog of the new version.
    fn catalog_read(
        &mut self,
        index: usize,
        package: &Version,
        end: Result<mortise_core::ReadEnd, mortise_core::StartError>,
    ) {
        let Some(app) = self.snapshot(index) else {
            return;
        };
        if app.package.raw != package.raw {
            return;
        }
        let root = self.settings.apps[index].root.clone();
        let read = match &end {
            Ok(end) => end
                .document::<Catalog>()
                .map_err(|error| strings::no_document(end, &error)),
            Err(error) => Err(strings::start_reason(error)),
        };
        let catalog = &mut self.apps[index].catalog;
        match read {
            Ok(read) => {
                let read_ms = jiff::Timestamp::now().as_millisecond();
                if let Some(dir) = &self.data_dir {
                    // A cache that cannot be written leaves the read for this session.
                    let _ = mortise_core::catalog_cache::store(dir, &root, package, &read, read_ms);
                }
                catalog.read = Some((read, read_ms, package.raw.clone()));
                catalog.error = None;
            }
            Err(reason) => {
                catalog.read = None;
                catalog.error = Some(reason);
            }
        }
    }

    /// The end of a dry run of a form (design D5): the file list for its words, or
    /// why it gave none. The exit code does not matter: a kept file exits 1 with a
    /// whole document. A dry run whose runner or working folder changed meanwhile
    /// listed the files of another app: its answer changes nothing.
    fn dry_run_read(
        &mut self,
        index: usize,
        words: Vec<String>,
        place: RunPlace,
        end: Result<mortise_core::ReadEnd, mortise_core::StartError>,
    ) {
        let Some(app) = self.snapshot(index) else {
            return;
        };
        let php = &self.settings.apps[index].php;
        if php.runner != place.runner || php.working_folder != place.folder {
            return;
        }
        let listed = match &end {
            Ok(end) => end
                .document::<mortise_core::GeneratorReport>()
                .map_err(|error| strings::no_document(end, &error)),
            Err(error) => Err(strings::start_reason(error)),
        };
        let open = &mut self.apps[index];
        let catalog = open.catalog.in_use(&app.package);
        if let Some(generator) = words.first().and_then(|name| catalog.get(name)) {
            open.generate.take_files(generator, words, place, listed);
        }
    }

    // --- Runs ---

    /// Starts artisan with `args` in the app at `index`, through its current runner.
    /// The drawer opens on it; it does not take focus.
    fn start_run(&mut self, index: usize, args: Vec<String>) {
        let Some(entry) = self.settings.apps.get(index) else {
            return;
        };
        let (root, php) = (entry.root.clone(), entry.php.clone());
        self.apps[index].runs.start(&self.runs, &root, &php, args);
        self.runner.shown = Shown::Open;
        self.runner.history_open = false;
    }

    /// Stops the command of the app at `index`. A migration, a rollback or a seed asks first.
    fn ask_stop(&mut self, index: usize) {
        let Some(open) = self.apps.get(index) else {
            return;
        };
        match open.runs.stop_asks_first() {
            Some(label) => {
                let dialog = Dialog {
                    cancel: strings::KEEP_RUNNING.to_owned(),
                    start_on_cancel: true,
                    ..Dialog::new(
                        strings::stop_title(&label),
                        strings::STOP_BODY.to_owned(),
                        strings::STOP_COMMAND.to_owned(),
                    )
                };
                self.dialog = Some((dialog, Pending::StopRun(index)));
                self.dialog_opened = true;
            }
            None => open.runs.stop(),
        }
    }

    /// Runs the command on screen again, unless the app runs one. It runs only when
    /// its exact line is on screen; otherwise the drawer shows that line first.
    /// A line Mortise asked about before it ran asks again (design D2).
    fn run_again(&mut self, index: usize) {
        if self.apps[index].busy().is_some() {
            return;
        }
        let runner = self.settings.apps[index].php.runner.as_ref();
        let runs = &mut self.apps[index].runs;
        let Some(args) = runs.again(&mut self.runner, runner) else {
            return;
        };
        // Reset, refresh, fresh and delete that Mortise refuses (production, or the
        // modules folder assumed for delete) open the Actions tab with the line to
        // copy. Otherwise they ask again in their own dialog, for these exact words; a
        // dry run of reset runs at once (design D7).
        if let Some((action, target)) = destructive_words(&args) {
            if !Action::destructive(&target).contains(&action) {
                return;
            }
            if self.refused(index, action) {
                self.show_refused(index, action, target, &args);
                return;
            }
            if mortise_core::asks_again(&args) {
                let ask = match action {
                    Action::FreshMigrate => Ask::TypeAppName,
                    Action::DeleteModule => Ask::TypeModuleName,
                    _ => Ask::NamingTables,
                };
                let act = Act {
                    action,
                    target,
                    args,
                    ask: Some(ask),
                };
                self.act(index, act);
                return;
            }
        }
        if !mortise_core::asks_again(&args) {
            self.start_run(index, args);
            return;
        }
        let dialog = Dialog {
            danger: true,
            start_on_cancel: true,
            ..Dialog::new(
                strings::run_again_title(&args.join(" ")),
                strings::RUN_AGAIN_BODY.to_owned(),
                strings::RUN_AGAIN.to_owned(),
            )
        };
        self.dialog = Some((dialog, Pending::RunAgain { index, args }));
        self.dialog_opened = true;
    }

    /// What the drawer asked for.
    fn drawer_action(&mut self, ctx: &egui::Context, index: usize, action: DrawerAction) {
        match action {
            DrawerAction::Stop => self.ask_stop(index),
            DrawerAction::RunAgain => self.run_again(index),
            DrawerAction::CopyOutput => {
                if let Some(text) = self.apps[index].runs.output_text(&self.runner) {
                    ctx.copy_text(text);
                }
            }
            DrawerAction::ToggleHistory => self.runner.toggle_history(),
            DrawerAction::Hide => self.runner.shown = Shown::Collapsed,
            DrawerAction::Show => self.runner.shown = Shown::Open,
            // Settings open on the PHP section with this app selected.
            DrawerAction::OpenSettings => {
                self.open_settings(Section::Php);
                if let Some(state) = &mut self.settings_open {
                    state.app = index;
                }
            }
            DrawerAction::OpenMigration(path) => self.open_in_editor(&path, 1),
            // The Migrate panel with the dry run checked; nothing runs (deviation S38).
            // The database of the failed run comes along; seeding stays off.
            DrawerAction::DryRun { module, database } => {
                if self.current != Some(index) {
                    self.select_app(index);
                }
                self.open_action(module, Action::Migrate);
                let options = &mut self.apps[index].actions.options;
                options.pretend = true;
                options.database = database.unwrap_or_default();
            }
        }
    }

    // --- Actions ---

    /// Selects `module` in the index, or "All modules" for `None`, opens the Actions
    /// tab and selects `action` there. Nothing runs: the line shows first (deviation S40).
    fn open_action(&mut self, module: Option<String>, action: Action) {
        let Some(current) = self.current else {
            return;
        };
        let open = &mut self.apps[current];
        let target = module.clone().map_or(Target::All, Target::Module);
        open.selection = module.map_or(Selection::All, Selection::Module);
        open.tab = Tab::Actions;
        open.findings_open = false;
        open.actions = ActionsState::on(target, action);
        self.reveal_next = true;
    }

    /// Starts an action of the Actions tab of the app at `index`, or asks first.
    fn act(&mut self, index: usize, act: Act) {
        let Some(ask) = act.ask else {
            self.start_action(index, act);
            return;
        };
        let name = self.settings.apps[index].display_name().to_owned();
        let object = match &act.target {
            Target::Module(module) => module.clone(),
            Target::All => strings::ALL_MODULES_OBJECT.to_owned(),
        };
        let production = self.app().is_some_and(|app| actions_view::production(&app));
        let dialog = match ask {
            Ask::TurnOff => {
                let users = self
                    .app()
                    .map(|app| actions_view::users_on(&app, &object))
                    .unwrap_or_default();
                let users: Vec<&str> = users.iter().map(String::as_str).collect();
                Dialog::new(
                    strings::turn_off_title(&object),
                    strings::turn_off_body(&users, &object),
                    strings::turn_off_confirm(&object),
                )
            }
            Ask::RollBack => {
                let mut body = strings::roll_back_body(&object);
                if production {
                    body = format!("{body} {}", strings::production_body(&name));
                }
                // With the migration state the dialog lists the last batch and
                // counts it (S10.3); without it, only the dry run (deviation S41).
                let connection = mortise_core::Options::from_args(&act.args).database;
                let last_batch: Vec<String> = self
                    .snapshot(index)
                    .and_then(|app| {
                        let migrations = app.package_data.migrations_for(&connection)?;
                        migrations.get(&object).map(|state| {
                            state
                                .last_batch()
                                .into_iter()
                                .map(|ran| ran.name.clone())
                                .collect()
                        })
                    })
                    .unwrap_or_default();
                let confirm = match last_batch.len() {
                    0 => strings::action_button(Action::RollBack).to_owned(),
                    n => strings::roll_back_counted(n),
                };
                Dialog {
                    danger: true,
                    left: Some(strings::DRY_RUN.to_owned()),
                    code: (!last_batch.is_empty()).then(|| bounded(&last_batch, "\n")),
                    ..Dialog::new(strings::roll_back_title(&object), body, confirm)
                }
            }
            Ask::Production => Dialog {
                danger: true,
                ..Dialog::new(
                    strings::production_title(act.action, &object),
                    strings::production_body(&name),
                    strings::production_confirm(act.action).to_owned(),
                )
            },
            Ask::NamingTables | Ask::TypeAppName | Ask::TypeModuleName => {
                match self.destructive_dialog(index, ask, &act) {
                    Some(dialog) => dialog,
                    None => return,
                }
            }
        };
        // Focus starts on Cancel, so Enter never confirms an action (S10.3).
        let dialog = Dialog {
            start_on_cancel: true,
            ..dialog
        };
        self.dialog = Some((dialog, Pending::Action { index, act }));
        self.dialog_opened = true;
    }

    /// The strongest dialog of a destructive action (S10.5, design D6): the title
    /// names the action and the object, the body names what is lost, and fresh and
    /// delete ask for a name to be typed. Without the migration state the tables are
    /// not named and the buttons carry no count (deviation S42).
    fn destructive_dialog(&self, index: usize, ask: Ask, act: &Act) -> Option<Dialog> {
        let app = self.snapshot(index)?;
        let entry = &self.settings.apps[index];
        let app_name = entry.display_name();
        let module = match &act.target {
            Target::Module(name) => name.as_str(),
            Target::All => "",
        };
        let title = strings::destructive_question(act.action, module, app_name);
        let confirm = strings::action_button(act.action).to_owned();
        let typed = ask.name_to_type(app_name, &act.target).map(|name| Typed {
            name: name.to_owned(),
            text: String::new(),
        });
        // With the migration state of the connection the words run on, reset and
        // refresh name the tables and count the migrations (S10.5 a and b); without
        // it they do not (deviation S42).
        let connection = mortise_core::Options::from_args(&act.args).database;
        let state = app
            .package_data
            .migrations_for(&connection)
            .and_then(|migrations| migrations.get(module));
        let ran = state.map_or(0, |state| state.ran.len());
        let tables = state.map_or(&[][..], |state| state.tables.as_slice());
        let code = (!tables.is_empty()).then(|| bounded(tables, ", "));
        let dialog = match act.action {
            Action::ResetMigrations => Dialog {
                left: Some(strings::DRY_RUN.to_owned()),
                code,
                lines: vec![match tables.len() {
                    0 => strings::nothing_outside(module),
                    n => strings::tables_untouched(n, module),
                }],
                ..Dialog::new(
                    title,
                    strings::reset_body(module),
                    match ran {
                        0 => confirm,
                        n => strings::reset_counted(n),
                    },
                )
            },
            Action::RefreshMigrations => Dialog {
                code,
                lines: vec![match tables.len() {
                    0 => strings::NO_DRY_RUN.to_owned(),
                    n => strings::tables_no_dry_run(n),
                }],
                ..Dialog::new(
                    title,
                    strings::refresh_body(module),
                    match ran {
                        0 => confirm,
                        n => strings::refresh_counted(n),
                    },
                )
            },
            Action::FreshMigrate => {
                let on = app
                    .modules
                    .iter()
                    .filter(|module| module.state == ModuleState::On)
                    .count();
                // The line that runs, as the panel shows it (deviation S37).
                let line = runner_view::line_now(entry.php.runner.as_ref(), &act.args)
                    .unwrap_or_else(|| format!("php artisan {}", act.args.join(" ")));
                Dialog {
                    code: Some(line),
                    ..Dialog::new(title, strings::fresh_body(on), confirm)
                }
            }
            Action::DeleteModule => {
                let found = app.module(module)?;
                let users = actions_view::users_on(&app, module);
                let users: Vec<&str> = users.iter().map(String::as_str).collect();
                let statuses = actions_view::statuses_file(&app);
                let body = strings::delete_body(
                    &app.path_shown(&found.dir),
                    found.files,
                    statuses.as_deref(),
                    &users,
                );
                // What git cannot bring back: left out when there is nothing.
                let alert = match found.uncommitted {
                    Some(0) => None,
                    Some(n) => Some(strings::uncommitted(n)),
                    None => Some(strings::CANNOT_TELL_GIT.to_owned()),
                };
                Dialog {
                    alert,
                    ..Dialog::new(title, body, confirm)
                }
            }
            _ => return None,
        };
        Some(Dialog {
            danger: true,
            typed,
            ..dialog
        })
    }

    /// True when Mortise does not run `action` in the app at `index` now. The
    /// snapshot can be a moment old, so `APP_ENV` and, for a delete, the modules
    /// folder are read again from the app's files. Without a snapshot nothing runs.
    fn refused(&self, index: usize, action: Action) -> bool {
        let entry = &self.settings.apps[index];
        let Some(app) = self.snapshot(index) else {
            return true;
        };
        if actions_view::refusal(&app, entry.display_name(), action).is_some() {
            return true;
        }
        if !action.is_destructive() {
            return false;
        }
        let layout = mortise_core::read_layout(&entry.root);
        let context = mortise_core::Context {
            production: mortise_core::read_app_env(&entry.root).as_deref() == Some("production"),
            users_on: &[],
            // A folder that moved since the dialog showed it is not the one it named.
            modules_folder_assumed: layout
                .assumed
                .contains(&mortise_core::LayoutKey::ModulesFolder)
                || layout.modules_dir != app.layout.modules_dir,
            package_data: mortise_core::gives_data(&app.package),
        };
        mortise_core::refuses(action, &context)
    }

    /// Opens the Actions tab of the app at `index` on `action` with the options of
    /// `args`, where the refusal is said and the line can be copied. Nothing runs.
    fn show_refused(&mut self, index: usize, action: Action, target: Target, args: &[String]) {
        if self.current != Some(index) {
            self.select_app(index);
        }
        let module = match target {
            Target::Module(name) => Some(name),
            Target::All => None,
        };
        self.open_action(module, action);
        self.apps[index].actions.options = mortise_core::Options::from_args(args);
    }

    /// "Dry run" of a dialog: the panel of the action, with the options of the
    /// words the dialog asked about and the dry run checked. Nothing runs
    /// (deviation S38).
    fn check_dry_run(&mut self, index: usize, act: &Act) {
        let shown = self.apps.get(index).is_some_and(|open| {
            open.tab == Tab::Actions && open.actions.shows(&act.target, act.action)
        });
        if !shown && self.current == Some(index) {
            let module = match &act.target {
                Target::Module(name) => Some(name.clone()),
                Target::All => None,
            };
            self.open_action(module, act.action);
        }
        if let Some(open) = self.apps.get_mut(index) {
            open.actions.options = mortise_core::Options::from_args(&act.args);
            open.actions.options.pretend = true;
        }
    }

    /// The snapshot of the app at `index`, once it is read.
    fn snapshot(&self, index: usize) -> Option<Arc<AppSnapshot>> {
        match self.apps.get(index)?.live.state() {
            LiveState::Ready(app) => Some(app),
            _ => None,
        }
    }

    /// Starts the words of an action and keeps what it was, for the toast at its end.
    fn start_action(&mut self, index: usize, act: Act) {
        if self.apps[index].busy().is_some() {
            return;
        }
        let modules_on = self.snapshot(index).map_or(0, |app| {
            app.modules
                .iter()
                .filter(|module| module.state == ModuleState::On)
                .count()
        });
        self.start_run(index, act.args.clone());
        let open = &mut self.apps[index];
        open.action_run = open.runs.running().is_some().then_some((act, modules_on));
    }

    /// The toast after an action of the app at `index` ended (design D6). A dry run
    /// and Check translations get none when they succeed. A failure gets one only when
    /// the drawer does not show it. On 13.4 a run of tests whose document counts its
    /// tests gets the toast of [`Self::documents_ended`] instead (design D8).
    fn action_ended(&mut self, index: usize, act: &Act, modules_on: usize) {
        if self.tests_ended(index, &act.args).is_some() {
            return;
        }
        let Some(runner_view::RunOver { end, lines, .. }) = self.apps[index].runs.ended() else {
            return;
        };
        let count = mortise_core::migration_count(lines);
        let object = match &act.target {
            Target::Module(module) => module.as_str(),
            Target::All => strings::ALL_MODULES_OBJECT,
        };
        let label = act.args.join(" ");
        if let RunEnd::Exited { code: Some(0), .. } = end {
            if mortise_core::has_option(&act.args, "--pretend") {
                return;
            }
            let text = match act.action {
                Action::TurnOff => strings::turned_off(object),
                Action::TurnOn => strings::turned_on(object),
                Action::Migrate => strings::migrated(object, count.ran),
                Action::RollBack => strings::rolled_back(object, count.rolled_back),
                Action::Seed => strings::seeded(object),
                Action::ResetMigrations => strings::reset_done(object, count.rolled_back),
                Action::RefreshMigrations => strings::refreshed(object, count.ran),
                Action::FreshMigrate => {
                    strings::dropped_all(self.settings.apps[index].display_name(), modules_on)
                }
                // The module is gone: the index selects "All modules"; the tab stays.
                Action::DeleteModule => {
                    self.apps[index].selection = Selection::All;
                    strings::deleted_module(object)
                }
                _ => return,
            };
            self.toast = Some(Toast::done(text));
            return;
        }
        if self.output_in_view(index) {
            return;
        }
        let text = match end {
            RunEnd::Exited { code, .. } => strings::stopped_with(&label, code),
            RunEnd::Stopped { .. } => strings::was_stopped(&label),
        };
        self.failed_toast(index, text);
    }

    /// True when the drawer shows the output of the app at `index`: a failure there
    /// needs no toast.
    fn output_in_view(&self, index: usize) -> bool {
        self.runner.shown == Shown::Open
            && self.current == Some(index)
            && self.settings_open.is_none()
            && self.adding.is_none()
    }

    /// The toast of a failed run of the app at `index`, with "Show output".
    fn failed_toast(&mut self, index: usize, text: String) {
        let root = self.settings.apps[index].root.clone();
        let toast =
            Toast::failed(text).with_action(strings::SHOW_OUTPUT, ToastAction::ShowOutput(root));
        self.toast = Some(toast);
    }

    /// The toast of a run of `module:test --json` of the app at `index` that ended,
    /// counted from its document (design D8). `None` for other words, or when the
    /// document does not count the tests.
    fn tests_ended(&self, index: usize, args: &[String]) -> Option<String> {
        if !is_json_run(args, Action::RunTests) {
            return None;
        }
        let over = self.apps.get(index)?.runs.ended()?;
        let report =
            mortise_core::whole_document::<TestReport>(over.stdout, over.stdout_cut).ok()?;
        tests_toast(&report, &words_target(args)?)
    }

    /// The documents of a run of the app at `index` that ended, from wherever it
    /// started: the Actions tab, Run again or the history (design D8, D9). The report
    /// of `module:lang --json` goes into the package data, and `module:test --json`
    /// gets the toast that counts its tests, whatever the exit code. A failure gets
    /// that toast only when the drawer does not show it.
    fn documents_ended(&mut self, index: usize, args: &[String]) {
        if let Some(text) = self.tests_ended(index, args) {
            let passed = matches!(
                self.apps[index].runs.ended(),
                Some(runner_view::RunOver {
                    end: RunEnd::Exited { code: Some(0), .. },
                    ..
                })
            );
            if passed {
                self.toast = Some(Toast::done(text));
            } else if !self.output_in_view(index) {
                self.failed_toast(index, text);
            }
            return;
        }
        if !is_json_run(args, Action::CheckTranslations) {
            return;
        }
        let report = self.apps[index].runs.ended().and_then(|over| {
            mortise_core::whole_document::<LangReport>(over.stdout, over.stdout_cut).ok()
        });
        if let Some(report) = report {
            self.set_package(index, |data| data.merge_translations(report));
        }
    }

    /// What the button of a toast asked for.
    fn toast_action(&mut self, action: ToastAction) {
        match action {
            ToastAction::ShowOutput(root) => {
                self.toast = None;
                // The app may be gone from the rail since the toast came.
                let Some(index) = self.settings.apps.iter().position(|app| app.root == root) else {
                    return;
                };
                self.settings_open = None;
                self.adding = None;
                self.select_app(index);
                self.runner.shown = Shown::Open;
                self.runner.history_open = false;
            }
        }
    }

    // --- Generate ---

    /// Opens the form of `generator` on the Generate tab: in `module`, or for
    /// `module:make` with "All modules" selected (design D7). Nothing runs.
    fn open_generate(&mut self, module: Option<String>, generator: &str) {
        let Some(current) = self.current else {
            return;
        };
        // A generator without a module argument opens under "All modules" too.
        let takes_module = self.snapshot(current).is_none_or(|app| {
            self.apps[current]
                .catalog
                .in_use(&app.package)
                .get(generator)
                .is_none_or(crate::generate_view::takes_module)
        });
        let open = &mut self.apps[current];
        open.selection = match module {
            Some(name) if generator != MODULE_MAKE => Selection::Module(name),
            _ => Selection::All,
        };
        open.tab = Tab::Generate;
        open.findings_open = false;
        open.generate.open_form(generator, takes_module);
        self.adding = None;
        self.settings_open = None;
        self.reveal_next = true;
    }

    /// What the Generate tab of the app at `index` asked for: a run, or a question
    /// first. Replace the module and Replace a file show the line they run (D6).
    fn generate(&mut self, index: usize, ask: GenerateAsk) {
        let runner = self.settings.apps[index].php.runner.as_ref();
        let line = |args: &[String]| {
            runner_view::line_now(runner, args)
                .unwrap_or_else(|| format!("php artisan {}", args.join(" ")))
        };
        let dialog = match &ask {
            GenerateAsk::NewModule => return,
            GenerateAsk::ReadCatalog => {
                let words = crate::generate_view::read_catalog_words();
                let Some(app) = self.snapshot(index) else {
                    return;
                };
                let package = app.package.clone();
                self.start_read(index, words, ReadFor::Catalog { package });
                return;
            }
            GenerateAsk::ListFiles { words } => {
                // Only a generator the package ships honours `--dry-run`: an app's
                // own may declare it and write anyway.
                let generator = words.first().map_or("", String::as_str);
                let ships = self
                    .snapshot(index)
                    .is_some_and(|app| mortise_core::generator_gives_data(&app.package, generator));
                if !ships {
                    return;
                }
                let args = mortise_core::with_dry_run(words.clone());
                let words = words.clone();
                let php = &self.settings.apps[index].php;
                let place = RunPlace {
                    runner: php.runner.clone(),
                    folder: php.working_folder.clone(),
                };
                self.start_read(index, args, ReadFor::DryRun { words, place });
                return;
            }
            GenerateAsk::Run(run) if run.generator != MODULE_MAKE || !forced(runner, &run.args) => {
                None
            }
            GenerateAsk::Run(run) => {
                let names = typed_names(&run.args);
                let names: Vec<&str> = names.iter().map(String::as_str).collect();
                let app = self.snapshot(index);
                let existing = app.as_ref().and_then(|app| {
                    let module = app.modules.iter().find(|module| {
                        names
                            .iter()
                            .any(|name| name.eq_ignore_ascii_case(&module.name))
                    })?;
                    Some((app.path_shown(&module.dir), module.files))
                });
                let body = strings::replace_module_body(
                    existing
                        .as_ref()
                        .map(|(folder, files)| (folder.as_str(), *files)),
                );
                Some(Dialog {
                    code: Some(line(&run.args)),
                    ..Dialog::new(
                        strings::replace_module_title(&names),
                        body,
                        strings::REPLACE_MODULE.to_owned(),
                    )
                })
            }
            GenerateAsk::Replace { file, run } => Some(Dialog {
                code: Some(line(&run.args)),
                ..Dialog::new(
                    strings::replace_file_title(file),
                    strings::REPLACE_FILE_BODY.to_owned(),
                    strings::REPLACE_FILE.to_owned(),
                )
            }),
        };
        let run = match ask {
            GenerateAsk::Run(run) | GenerateAsk::Replace { run, .. } => run,
            GenerateAsk::NewModule | GenerateAsk::ReadCatalog | GenerateAsk::ListFiles { .. } => {
                return;
            }
        };
        let Some(dialog) = dialog else {
            self.start_generate(index, run);
            return;
        };
        // Something is written over: the red button, and focus starts on Cancel.
        let dialog = Dialog {
            danger: true,
            start_on_cancel: true,
            ..dialog
        };
        self.dialog = Some((dialog, Pending::Generate { index, run }));
        self.dialog_opened = true;
    }

    /// Starts a generator and keeps what it was, for its result at its end.
    fn start_generate(&mut self, index: usize, run: GenerateRun) {
        if self.apps[index].busy().is_some() {
            return;
        }
        let before: BTreeSet<String> = self.snapshot(index).map_or_else(BTreeSet::new, |app| {
            app.modules
                .iter()
                .map(|module| module.name.clone())
                .collect()
        });
        let forced = forced(self.settings.apps[index].php.runner.as_ref(), &run.args);
        self.start_run(index, run.args.clone());
        let open = &mut self.apps[index];
        open.select_new = None;
        open.generate.drop_files();
        let form = open.generate.form();
        open.generate_run = open.runs.running().is_some().then_some(Generating {
            run,
            before,
            forced,
            form,
        });
        if open.generate_run.is_none() {
            open.generate.run_ended();
        }
    }

    /// The result and the toast after a generator of the app at `index` ended
    /// (design D6). The result comes from the output lines, not from the exit code.
    fn generate_ended(&mut self, index: usize, generating: Generating) {
        let Generating {
            run,
            before,
            forced,
            form,
        } = generating;
        self.apps[index].generate.run_ended();
        let root = self.settings.apps[index].root.clone();
        let Some(runner_view::RunOver {
            end,
            lines,
            stdout,
            stdout_cut,
        }) = self.apps[index].runs.ended()
        else {
            return;
        };
        let done = matches!(end, RunEnd::Exited { code: Some(0), .. });
        // A run with `--json` gives its files in its answer (P5); without one they are
        // read from the lines, and the result says why (design D6).
        let json = mortise_core::has_option(&run.args, "--json");
        let answer = json.then(|| {
            mortise_core::whole_document::<mortise_core::GeneratorReport>(stdout, stdout_cut)
        });
        // The modules folder of the app's layout and its modules place a path of a
        // container (Sail).
        let app = self.snapshot(index);
        let modules = app.as_ref().map_or_else(
            || "Modules".to_owned(),
            |app| app.path_shown(&app.layout.modules_dir),
        );
        let folders = app
            .as_deref()
            .map(crate::generate_view::module_folders)
            .unwrap_or_default();
        // The modules the run writes in place its paths first.
        let asked = crate::generate_view::run_modules(
            app.as_deref(),
            &run.generator,
            run.module.as_deref(),
            &run.args,
        );
        let asked: Vec<&str> = asked.iter().map(String::as_str).collect();
        let (mut files, autoload_failed, fallback) = match answer {
            Some(Ok(report)) => (
                mortise_core::reported_files(&report, &root, &modules, &asked, &folders),
                report.autoload_failed(),
                None,
            ),
            answer => (
                mortise_core::generated_files(lines, forced),
                mortise_core::autoload_failed(lines),
                answer.and_then(|answer| answer.err().map(|e| strings::no_run_document(&e, end))),
            ),
        };
        // The files S9.3 took out of the command: the run left them, so its result
        // lists them as left, each with its own Replace (design D5).
        let taken_out: Vec<TakenOut> = run
            .taken_out
            .iter()
            .map(|file| TakenOut {
                path: mortise_core::app_relative(&root, &modules, &asked, &folders, &file.path)
                    .unwrap_or_else(|| file.path.clone()),
                replace: file.replace.clone(),
            })
            .collect();
        if !files.is_empty() {
            files.extend(taken_out.iter().map(|file| mortise_core::GeneratedFile {
                path: file.path.clone(),
                outcome: FileOutcome::Left,
            }));
        }
        let label = run.args.join(" ");
        let in_view = self.runner.shown == Shown::Open
            && self.current == Some(index)
            && self.settings_open.is_none()
            && self.adding.is_none();
        // The result belongs to the form the run started from (design D6).
        let attached = form.is_some() && self.apps[index].generate.form() == form;
        let on_tab = in_view && attached && self.apps[index].tab == Tab::Generate;
        let failed_toast = |text: String, root: PathBuf| {
            Toast::failed(text).with_action(strings::SHOW_OUTPUT, ToastAction::ShowOutput(root))
        };
        let open = &mut self.apps[index];
        // A Replace of a left row of the result: that row changes, the others stay.
        let row_taken = attached
            && run
                .row
                .as_deref()
                .is_some_and(|row| open.generate.replace_row(row, &files, fallback.clone()));
        // A run with `--json` that read no file, such as one that failed at boot: the
        // form keeps its line and says why the answer was not read (design D2).
        if attached && !row_taken && files.is_empty() && fallback.is_some() {
            let result =
                GenerateResult::new(run.module.clone(), run.args.clone(), Vec::new(), false);
            open.generate
                .show_result(result.read_from_output(fallback.clone()));
        }

        if run.generator == MODULE_MAKE {
            let names = typed_names(&run.args);
            // A run that wrote files made a module, whatever its exit code: it is
            // selected once the app is read with it. When composer dump-autoload
            // failed, the form keeps its alert on screen instead (design D6).
            let wrote = files
                .iter()
                .any(|file| matches!(file.outcome, FileOutcome::Created | FileOutcome::Replaced));
            open.live.refresh();
            if autoload_failed {
                if attached {
                    // Read from the lines when the answer was missing or cut: why.
                    let result = GenerateResult::new(None, run.args, files, true);
                    open.generate.show_result(result.read_from_output(fallback));
                }
            } else if wrote {
                open.select_new = Some(SelectNew {
                    before,
                    names: names.clone(),
                });
            }
            if done && !autoload_failed {
                open.generate.clear_name();
                let names: Vec<&str> = names.iter().map(String::as_str).collect();
                self.toast = Some(Toast::done(strings::created_modules(&names)));
                return;
            }
            // No toast while the drawer shows the failure.
            if !in_view {
                let text = match end {
                    RunEnd::Exited { code, .. } => strings::stopped_with(&label, code),
                    RunEnd::Stopped { .. } => strings::was_stopped(&label),
                };
                self.toast = Some(failed_toast(text, root));
            }
            return;
        }

        // No file read: no result; the drawer says what happened, and the form why
        // the answer was not read.
        if files.is_empty() {
            if !done && !in_view {
                let text = match end {
                    RunEnd::Exited { code, .. } => strings::stopped_with(&label, code),
                    RunEnd::Stopped { .. } => strings::was_stopped(&label),
                };
                self.toast = Some(failed_toast(text, root));
            }
            return;
        }
        let place = run.module.clone().unwrap_or_default();
        let total = files.len();
        let written: Vec<&mortise_core::GeneratedFile> = files
            .iter()
            .filter(|file| matches!(file.outcome, FileOutcome::Created | FileOutcome::Replaced))
            .collect();
        let replaced = written
            .iter()
            .all(|file| file.outcome == FileOutcome::Replaced);
        let toast = if written.len() == total {
            Some(Toast::done(strings::created_files(total, &place, replaced)))
        } else if !on_tab {
            // A failure that is not on screen: the sentence of the result (not in the spec).
            let name = |outcome| {
                files
                    .iter()
                    .filter(|file| file.outcome == outcome)
                    .map(|file| file.path.rsplit(['/', '\\']).next().unwrap_or(&file.path))
                    .collect::<Vec<_>>()
            };
            let text = strings::left_alert(
                written.len(),
                total,
                &place,
                &name(FileOutcome::Left),
                &name(FileOutcome::Failed),
            );
            Some(failed_toast(text, root))
        } else {
            None
        };
        if let Some(toast) = toast {
            self.toast = Some(toast);
        }
        if attached && !row_taken {
            let result = GenerateResult::new(run.module, run.args, files, false);
            open.generate
                .show_result(result.read_from_output(fallback).with_taken_out(taken_out));
        }
    }

    // --- Rules ---

    /// Changes the rules of the current app. The first change asks before
    /// `mortise.json` is created.
    fn set_rules(&mut self, rules: Rules) {
        let (Some(app), Some(current)) = (self.app(), self.current) else {
            return;
        };
        if app.root.join(mortise_core::RULES_FILE).exists() {
            self.save_rules(&app.root, &rules);
        } else {
            let name = self.settings.apps[current].display_name();
            let dialog = Dialog::new(
                strings::create_rules_title(name),
                strings::create_rules_body(name),
                strings::CREATE_RULES_FILE.to_owned(),
            );
            let root = app.root.clone();
            self.dialog = Some((dialog, Pending::CreateRules { root, rules }));
            self.dialog_opened = true;
        }
    }

    /// Writes the rules to `mortise.json` of the app at `root` and reads the app again.
    /// Nothing is written when that app is no longer the one on screen.
    fn save_rules(&mut self, root: &Path, rules: &Rules) {
        let (Some(app), Some(current)) = (self.app(), self.current) else {
            return;
        };
        if app.root != root {
            return;
        }
        match rules.save(root) {
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
        // Which region holds focus (K1). Focus is on the index when its selected row
        // holds it, and also when nothing holds it.
        let in_list = draw::take_list_focus(ui.ctx());
        let on_row = index::take_row_focus(ui.ctx());
        let focused = ui.memory(|memory| memory.focused());
        let on_index = focused.is_none() || on_row;
        let focused_at = focused
            .filter(|_| !on_row)
            .and_then(|id| ui.ctx().read_response(id))
            .map(|response| response.rect.center());
        let in_workspace = focused_at.is_some_and(|at| self.work_rect.contains(at));

        // Focus on its way into the workspace: from the tab row to the first control
        // of the tab. A tab without a control gives focus back to the index.
        match self.entering_workspace {
            2 => {
                ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::Next));
                self.entering_workspace = 1;
            }
            1 => {
                if let Some(id) = focused.filter(|_| !in_workspace) {
                    ui.memory_mut(|memory| memory.surrender_focus(id));
                }
                self.entering_workspace = 0;
            }
            _ => {}
        }

        // The command palette, from anywhere. The same key closes it.
        if chord(Modifiers::COMMAND, Key::K) {
            if self.palette.is_some() {
                self.palette = None;
            } else if self.dialog.is_none() {
                self.open_palette();
            }
        }
        // Copy the output of the drawer: before Add app, which would take Cmd Shift O too.
        let free = self.palette.is_none() && self.dialog.is_none() && self.adding.is_none();
        let copy_output = free && chord(Modifiers::COMMAND | Modifiers::SHIFT, Key::O);
        // Add app: the folder dialog opens at once. A dialog holds the keys.
        if self.palette.is_none() && self.dialog.is_none() && chord(Modifiers::COMMAND, Key::O) {
            self.pick(ui.ctx());
        }
        // Esc closes the topmost thing: the add panel, then the toast, then Settings,
        // then the text of the filter, then the filter field itself.
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
            } else if self.settings_open.is_some() {
                // Settings take the place of the workbench, so they are above it.
                // Esc leaves a field first, and egui has already taken its focus.
                // The next Esc closes Settings.
                if !self.settings_field_was_focused {
                    self.close_settings = true;
                }
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
                let picker = open.tab == Tab::Generate;
                if let Some(surface) = scanning {
                    surface.stop();
                } else if picker
                    && (open.generate.filter_was_focused() || open.filter.is_empty())
                    && open.generate.escape()
                {
                    // Esc clears the filter of the picker (S9.1).
                } else if !open.filter.is_empty() {
                    // Esc clears the text first and leaves the field second. egui has
                    // already taken focus from the field for this Esc: give it back.
                    open.filter.clear();
                    self.focus_filter = self.filter_was_focused;
                } else if !self.filter_was_focused && open.findings_open {
                    // Back from a second view.
                    open.findings_open = false;
                }
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

        // F6 and Shift F6: to the next or the previous region. Rail, index, workspace,
        // and the runner drawer while it is open.
        let backwards = chord(Modifiers::SHIFT, Key::F6);
        if (backwards || chord(Modifiers::NONE, Key::F6))
            && let Some(current) = self.current
        {
            let regions = if self.runner.shown == Shown::Open {
                4
            } else {
                3
            };
            let in_drawer = focused_at.is_some_and(|at| self.drawer_rect.contains(at));
            let region = if on_index {
                1
            } else if in_drawer {
                3
            } else if in_workspace {
                2
            } else {
                0
            };
            let target = if backwards {
                (region + regions - 1) % regions
            } else {
                (region + 1) % regions
            };
            let id = match target {
                0 => rail::current_tile(ui.ctx()),
                2 => Some(workspace::tab_id(ui, self.apps[current].tab)),
                3 => Some(runner_view::handle_id()),
                _ => None,
            };
            ui.memory_mut(|memory| match (id, focused) {
                (Some(id), _) => memory.request_focus(id),
                // The index: its selected row has the keys when nothing else holds focus.
                (None, Some(focused)) => memory.surrender_focus(focused),
                (None, None) => {}
            });
            self.keyboard = true;
        }

        // Collapse or open the module index. Mortise then stops deciding its width
        // until the window is resized.
        if chord(Modifiers::COMMAND, Key::B) {
            let set = f32::from(self.settings.general.index_width);
            let (_, collapsed) = index_width(self.window_w, self.index_choice, set);
            self.index_choice = Some(!collapsed);
        }
        // The new module form, anywhere an app is open (design D7). Nothing runs.
        if chord(Modifiers::COMMAND, Key::N) && self.app().is_some() {
            self.open_generate(None, MODULE_MAKE);
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

        // The runner drawer, where not typing. Cmd Shift R comes before Cmd R.
        if !typing && let Some(current) = self.current {
            if chord(Modifiers::CTRL, Key::Backtick) || chord(Modifiers::COMMAND, Key::J) {
                self.runner.toggle();
            }
            if chord(Modifiers::COMMAND | Modifiers::SHIFT, Key::R) {
                self.run_again(current);
            }
            if chord(Modifiers::COMMAND | Modifiers::SHIFT, Key::H) {
                self.runner.toggle_history();
            }
            if copy_output {
                self.drawer_action(ui.ctx(), current, DrawerAction::CopyOutput);
            }
            if chord(Modifiers::COMMAND, Key::Period) {
                self.ask_stop(current);
            }
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
        // G: the Generate tab with the filter of the picker focused (S9.1).
        if pressed(Key::G) {
            open.tab = Tab::Generate;
            open.findings_open = false;
            open.generate.open_picker(true);
        }
        // F: the findings list of the app.
        if pressed(Key::F) {
            open.selection = Selection::All;
            open.tab = Tab::Summary;
            open.findings_open = true;
            open.finding_row = 0;
        }

        // Enter or Right: from the index into the workspace. Esc brings focus back.
        if on_index && (pressed(Key::Enter) || pressed(Key::ArrowRight)) {
            ui.memory_mut(|memory| {
                // The arrow is not also a move of focus to the right.
                memory.move_focus(egui::FocusDirection::None);
                memory.request_focus(workspace::tab_id(ui, open.tab));
            });
            self.entering_workspace = 2;
        }

        // J and K, or Down and Up: the next or previous row. Home and End: the first or last.
        // Only the rows the filter leaves. A list or table that has focus keeps these
        // keys for itself, and the arrows move focus when another control has it.
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
        if !in_list && (pressed(Key::J) || (on_index && pressed(Key::ArrowDown))) {
            target = (current + 1).min(rows - 1);
        }
        if !in_list && (pressed(Key::K) || (on_index && pressed(Key::ArrowUp))) {
            target = current.saturating_sub(1);
        }
        if on_index && pressed(Key::Home) {
            target = 0;
        }
        if on_index && pressed(Key::End) {
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
            self.open_folder(ctx, &folder);
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
                if let Some(panel) = &mut self.adding {
                    panel.start_reading();
                }
                let checked = self.check(ctx, &path);
                if let (Some(panel), Some(checked)) = (&mut self.adding, checked) {
                    panel.set_checked(checked);
                }
            }
            // What was found is used only while it belongs to the path in the field.
            AddAction::Add | AddAction::Open(_) | AddAction::Test | AddAction::TestPhp
                if !self.adding.as_ref().is_some_and(AddPanel::is_current) => {}
            AddAction::Add => self.add_found(ctx),
            AddAction::Open(index) => {
                self.settings_open = None;
                self.select_app(index);
            }
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
            // The add panel has no working folder: the test starts in the app folder.
            AddAction::TestPhp => {
                if let Some(AddPanel {
                    checked: Some(Checked::Found(app)),
                    php: Some(form),
                    php_test,
                    ..
                }) = &mut self.adding
                {
                    php_test.start(&form.runner, app.root.clone(), ctx);
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
        self.reveal = std::mem::take(&mut self.reveal_next);
        draw::focus_what_was_asked(&ctx);
        #[cfg(target_os = "macos")]
        let menu_chosen = self.menu_chosen(&ctx);
        self.keys(ui);
        self.take_in();
        self.sync_runtime(&ctx);
        // Lines and the elapsed time show without a callback from the core.
        let mut running = false;
        let mut ended = Vec::new();
        let mut generated = Vec::new();
        let mut finished = Vec::new();
        for (index, open) in self.apps.iter_mut().enumerate() {
            let was = open.runs.running().is_some();
            running |= open.runs.poll();
            if was && open.runs.running().is_none() {
                let action_run = open.action_run.take();
                if let Some(args) = open.runs.args() {
                    finished.push((index, args.to_vec()));
                }
                if let Some((act, modules_on)) = action_run {
                    ended.push((index, act, modules_on));
                }
                if let Some(generating) = open.generate_run.take() {
                    generated.push((index, generating));
                }
            }
        }
        let mut read = Vec::new();
        for (index, open) in self.apps.iter_mut().enumerate() {
            if let (LiveState::Ready(app), Some(entry)) =
                (open.live.state(), self.settings.apps.get(index))
            {
                open.catalog
                    .look_up(self.data_dir.as_deref(), &entry.root, &app.package);
            }
            let Some(reading) = &mut open.reading else {
                continue;
            };
            running = true;
            if let Some(end) = reading.read.poll()
                && let Some(reading) = open.reading.take()
            {
                read.push((index, reading.purpose, Ok(end)));
            }
        }
        for (index, act, modules_on) in ended {
            self.action_ended(index, &act, modules_on);
        }
        for (index, purpose, end) in read {
            self.read_ended(index, purpose, end);
        }
        for (index, generating) in generated {
            self.generate_ended(index, generating);
        }
        for (index, args) in finished {
            self.after_run(index, &args);
        }
        if running {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // A folder dropped on the window opens the add panel on it.
        let dropped = ui.input(|input| {
            input
                .raw
                .dropped_files
                .first()
                .map(|file| file.path().to_owned())
        });
        if let Some(folder) = dropped {
            self.open_folder(&ctx, &folder);
        }
        self.take_in_folder(&ctx);

        let window = ui.max_rect();
        ui.painter().rect_filled(window, 0, pal.frame);
        // A resize hands the width of the index back to Mortise.
        if (window.width() - self.window_w).abs() > 0.5 {
            self.window_w = window.width();
            self.index_choice = None;
        }
        let chrome = self.apply_general(&ctx);
        let reduced_motion = match self.settings.general.motion {
            mortise_core::Motion::Reduced => true,
            mortise_core::Motion::Full => false,
            mortise_core::Motion::System => self.system_reduces_motion(),
        };

        let bar = Rect::from_min_size(window.min, vec2(window.width(), size::TITLEBAR_H));
        let app_name = self
            .current
            .map(|current| self.settings.apps[current].display_name().to_owned());
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
                root: &app.root,
                name: app.display_name(),
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
        let set_index_w = f32::from(self.settings.general.index_width);
        let (index_w, index_collapsed) =
            index_width(window.width(), self.index_choice, set_index_w);
        let mut new_index_w = None;
        let mut settings_actions = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs() as i64);
        let mut add_action = None;
        let mut work_action = None;
        let mut toggle_order = false;
        let mut new_module = false;
        let problems_first = self
            .current
            .is_some_and(|current| self.settings.apps[current].problems_first);
        let mut first_run_add = false;
        let mut drawer_rect = Rect::NOTHING;
        let mut drawer_actions = Vec::new();
        let monogram = self.monogram_preview();
        if let Some(panel) = &mut self.adding {
            // The bar is drawn only after 300 ms, so quick work does not flash, and only
            // once the files are counted, so it never reads "0 of 0 files".
            let progress = self
                .reading
                .as_ref()
                .filter(|reading| reading.since.elapsed() >= SHOW_PROGRESS_AFTER)
                .map(|reading| reading.progress.get())
                .filter(|&(_, total)| total > 0);
            add_action = add::add_panel(
                ui,
                pal,
                rest,
                panel,
                &monogram,
                progress,
                self.keyboard,
                mac,
            );
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
                        php_test: &mut open.php_test,
                        running: busy(&open.runs, open.reading.as_ref()).map(strings::is_running),
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
            // A click on "Back" can also leave an edited field: the edit comes first.
            settings_actions = in_section;
            settings_actions.extend(in_list);
        } else if let Some(current) = self.current {
            let open = &mut self.apps[current];
            match open.live.state() {
                LiveState::Ready(app) => {
                    let rows = self.settings.general.index_rows;
                    let index_w = if index_collapsed {
                        index::strip_width(&app, &open.filter, rows, rest.height())
                    } else {
                        index_w
                    };
                    let index_rect = Rect::from_min_size(rest.min, vec2(index_w, rest.height()));
                    // The runner drawer takes the bottom of the workspace column.
                    let column = Rect::from_min_max(pos2(index_rect.right(), rest.top()), rest.max);
                    let drawer_h = self.runner.taken(window.height(), column.height());
                    let work_rect = Rect::from_min_max(
                        column.min,
                        pos2(column.right(), column.bottom() - drawer_h),
                    );
                    drawer_rect =
                        Rect::from_min_max(pos2(column.left(), work_rect.bottom()), column.max);
                    // The Findings list belongs to the Summary of "All modules" only.
                    if open.selection != Selection::All || open.tab != Tab::Summary {
                        open.findings_open = false;
                    }
                    let entry = &self.settings.apps[current];
                    let held = self.dialog.is_some() || self.palette.is_some();
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
                    let state = IndexState {
                        selection: &mut open.selection,
                        filter: &mut open.filter,
                        focus_filter: self.focus_filter,
                        keyboard: self.keyboard,
                        reveal: self.reveal,
                        collapsed: index_collapsed,
                        problems_first,
                        rows,
                        mac,
                        name: entry.display_name(),
                        runtime: runtime.brief().map(|brief| IndexRuntime {
                            brief: brief.for_header(),
                            environment: runtime.environment(),
                            checked: runtime.checked(),
                        }),
                    };
                    let index_asked = index::index(ui, pal, index_rect, &app, state);
                    toggle_order = index_asked.toggle_order;
                    new_module = index_asked.new_module;
                    // Where the window has room, the user sets the width of the index.
                    if !index_collapsed && window.width() >= 1076.0 {
                        new_index_w = index_edge(ui, index_rect, self.keyboard, pal);
                    }
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

                    // The options of the Actions tab belong to the selection, wherever it
                    // changed in this frame.
                    let target = match &open.selection {
                        Selection::All => Target::All,
                        Selection::Module(name) => Target::Module(name.clone()),
                    };
                    open.actions.follow(target, open.tab == Tab::Actions);
                    // After module:make, the new module once the app is read with it.
                    if open.select_new.is_some() && open.selection != Selection::All {
                        open.select_new = None;
                    }
                    if let Some(name) = open.select_new.as_ref().and_then(|wait| wait.find(&app)) {
                        open.selection = Selection::Module(name);
                        open.tab = Tab::Summary;
                        open.select_new = None;
                        self.reveal_next = true;
                        ctx.request_repaint();
                    }
                    let module = match &open.selection {
                        Selection::Module(name) => Some(name.as_str()),
                        Selection::All => None,
                    };
                    open.generate.follow(module, open.tab == Tab::Generate);

                    // The workspace, after the index: Tab goes from the rail to the index
                    // and then to the workspace.
                    let state = WorkState {
                        name: entry.display_name(),
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
                        actions: &mut open.actions,
                        actions_inputs: ActionsInputs {
                            app: entry.display_name(),
                            runner: entry.php.runner.as_ref(),
                            busy: busy(&open.runs, open.reading.as_ref()),
                            held,
                            keyboard: self.keyboard,
                            mac,
                            reading_state: open.reading.as_ref().and_then(|reading| match &reading
                                .purpose
                            {
                                ReadFor::Migrations { target, .. } => Some(target),
                                _ => None,
                            }),
                            state_error: open.migration_error.as_ref(),
                        },
                        generate: &mut open.generate,
                        generate_inputs: GenerateInputs {
                            app: entry.display_name(),
                            catalog: open.catalog.in_use(&app.package),
                            catalog_read: open.catalog.picker(
                                &app.package,
                                matches!(
                                    open.reading,
                                    Some(PackageRead {
                                        purpose: ReadFor::Catalog { .. },
                                        ..
                                    })
                                ),
                            ),
                            runner: entry.php.runner.as_ref(),
                            folder: entry.php.working_folder.as_deref(),
                            busy: busy(&open.runs, open.reading.as_ref()),
                            listing: matches!(
                                open.reading,
                                Some(PackageRead {
                                    purpose: ReadFor::DryRun { .. },
                                    ..
                                })
                            ),
                            held,
                            keyboard: self.keyboard,
                            mac,
                            reduced_motion,
                        },
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
                    // A module chosen in the workspace shows in the index in the next frame.
                    if work_action.is_some() {
                        ctx.request_repaint();
                    }
                    match &work_action {
                        Some(BoundaryAction::Select(module)) => {
                            open.selection = Selection::Module(module.clone());
                            self.reveal_next = true;
                        }
                        Some(BoundaryAction::ShowFindings) => {
                            open.findings_open = true;
                            open.finding_row = 0;
                        }
                        Some(BoundaryAction::ShowLink { module, other }) => {
                            open.selection = Selection::Module(module.clone());
                            open.tab = Tab::Boundary;
                            open.boundary.surface_view = false;
                            open.boundary.other = Some(other.clone());
                            open.boundary.file = 0;
                            self.reveal_next = true;
                        }
                        _ => {}
                    }
                    self.work_rect = work_rect;

                    // The drawer, after the workspace in the Tab order. A read keeps
                    // the app busy too (design D3).
                    let busy = open.busy().map(str::to_owned);
                    let inputs = runner_view::Inputs {
                        app: entry.display_name(),
                        runner: entry.php.runner.as_ref(),
                        snapshot: &app,
                        busy: busy.as_deref(),
                        keyboard: self.keyboard,
                        mac,
                    };
                    // Each action carries its app, as the selection may change later in the frame.
                    drawer_actions = runner_view::drawer(
                        ui,
                        pal,
                        drawer_rect,
                        &mut self.runner,
                        &mut open.runs,
                        &inputs,
                    )
                    .into_iter()
                    .map(|action| (current, entry.root.clone(), action))
                    .collect();
                }
                LiveState::Reading => {
                    ui.painter().rect_filled(rest, 0, pal.surface);
                    // The first read of a large app: say how far it is.
                    let (read, total) = open.live.progress();
                    if open.since.elapsed() >= SHOW_PROGRESS_AFTER && total > 0 {
                        let label = strings::reading_files(read, total);
                        let share = read as f32 / total as f32;
                        let at = rest.min + vec2(16.0, 14.0);
                        draw::progress(
                            ui,
                            pal,
                            at,
                            320.0_f32.min(rest.width() - 32.0),
                            &label,
                            share,
                        );
                    }
                    ctx.request_repaint_after(std::time::Duration::from_millis(50));
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

        self.drawer_rect = drawer_rect;
        // The toast stands above the drawer.
        let drawer_h = if drawer_rect.is_positive() {
            drawer_rect.height()
        } else {
            0.0
        };
        let toast_area = Rect::from_min_max(rest.min, pos2(rest.right(), rest.bottom() - drawer_h));
        let mut toast_action = None;
        if let Some(toast) = &mut self.toast {
            let (open, asked) = toast.show(ui, pal, toast_area, reduced_motion);
            toast_action = asked;
            if !open {
                self.toast = None;
            }
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
                    (entry.display_name().to_owned(), problems)
                })
                .collect();
            let recent = self
                .apps
                .get(scope)
                .map_or(&[][..], |open| open.recent.as_slice());
            let history = self
                .apps
                .get(scope)
                .map(|open| open.runs.history())
                .unwrap_or_default();
            let catalog = match (self.apps.get(scope), snapshot.as_deref()) {
                (Some(open), Some(app)) => open.catalog.in_use(&app.package),
                _ => Catalog::v13_3(),
            };
            let source = palette::Source {
                scope: snapshot.as_deref().map(|app| (scope, app)),
                apps: &names,
                recent,
                history: &history,
                catalog,
                now: jiff::Timestamp::now().as_second(),
                selected: self.apps.get(scope).and_then(|open| match &open.selection {
                    Selection::Module(name) => Some(name.as_str()),
                    Selection::All => None,
                }),
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
        if let Some((dialog, _)) = &mut self.dialog {
            let answer = dialog.show(ui, pal, self.dialog_opened, self.keyboard);
            self.dialog_opened = false;
            match (answer, self.dialog.take_if(|_| answer.is_some())) {
                (Some(Answer::Confirm), Some((_, Pending::CreateRules { root, rules }))) => {
                    self.save_rules(&root, &rules)
                }
                (Some(Answer::Confirm), Some((_, Pending::RemoveApp(index)))) => {
                    self.remove_app(index)
                }
                (Some(Answer::Confirm), Some((_, Pending::StopRun(index)))) => {
                    if let Some(open) = self.apps.get(index) {
                        open.runs.stop();
                    }
                }
                (Some(Answer::Confirm), Some((_, Pending::RunAgain { index, args }))) => {
                    if self
                        .apps
                        .get(index)
                        .is_some_and(|open| open.busy().is_none())
                    {
                        self.start_run(index, args);
                    }
                }
                // The app may have changed while the dialog was open, such as its
                // `.env` to production: a refused action shows its line instead.
                (Some(Answer::Confirm), Some((_, Pending::Action { index, act }))) => {
                    if act.action.is_destructive() && self.refused(index, act.action) {
                        self.show_refused(index, act.action, act.target.clone(), &act.args);
                    } else {
                        self.start_action(index, act)
                    }
                }
                (Some(Answer::Confirm), Some((_, Pending::Generate { index, run }))) => {
                    self.start_generate(index, run)
                }
                // "Dry run" checks the option on the panel and runs nothing (deviation S38).
                (Some(Answer::Left), Some((_, Pending::Action { index, act }))) => {
                    self.check_dry_run(index, &act)
                }
                _ => {}
            }
        }
        if toggle_order {
            self.toggle_order();
        }
        if let Some((width, keep)) = new_index_w {
            self.settings.general.index_width = width as u16;
            if keep {
                self.save();
            }
            ctx.request_repaint();
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
            Some(BoundaryAction::OpenAction(action)) => {
                let module = match self.selection() {
                    Selection::Module(name) => Some(name),
                    Selection::All => None,
                };
                self.open_action(module, action);
            }
            Some(BoundaryAction::OpenModuleAction { module, action }) => {
                self.open_action(Some(module), action);
            }
            Some(BoundaryAction::Act(act)) => {
                if let Some(current) = self.current {
                    self.act(current, act);
                }
            }
            Some(BoundaryAction::ReadMigrations { target, args }) => {
                if let Some(current) = self.current {
                    self.read_migrations(current, target, args);
                }
            }
            Some(BoundaryAction::Generate(GenerateAsk::NewModule)) => {
                self.open_generate(None, MODULE_MAKE)
            }
            Some(BoundaryAction::Generate(ask)) => {
                if let Some(current) = self.current {
                    self.generate(current, ask);
                }
            }
            _ => {}
        }
        if let Some(action) = toast_action {
            self.toast_action(action);
        }

        // Focus is always visible while the keyboard is in use.
        if self.keyboard {
            draw::focus_ring_for_the_rest(ui, pal);
        }
        chrome::resize_edges(ui, chrome);
        self.focus_filter = false;
        let filter_id = index::filter_id(ui);
        self.filter_was_focused = ui.memory(|memory| memory.has_focus(filter_id));
        self.settings_field_was_focused = self.settings_open.is_some() && ctx.text_edit_focused();

        // What the pointer asked for, after everything is painted.
        match rail_action {
            Some(RailAction::Select(index)) => {
                self.settings_open = None;
                self.select_app(index);
            }
            Some(RailAction::Move(from, to)) => self.move_app(from, to),
            Some(RailAction::Add) => self.adding = Some(AddPanel::empty()),
            Some(RailAction::Settings) if self.settings_open.is_some() => self.settings_open = None,
            Some(RailAction::Settings) => self.open_settings(Section::Apps),
            None => {}
        }
        if std::mem::take(&mut self.close_settings) {
            self.settings_open = None;
        }
        for action in settings_actions {
            match action {
                SettingsAction::Close => self.settings_open = None,
                SettingsAction::Changed { reread } => {
                    self.save();
                    if let Some(index) = reread {
                        let open = &self.apps[index];
                        open.live
                            .set_options(read_options(&self.settings.apps[index], &open.package));
                    }
                }
                // An app that runs a command stays: Remove is off until it ends.
                SettingsAction::RemoveApp(index) if self.apps[index].busy().is_some() => {}
                SettingsAction::RemoveApp(index) => {
                    let name = self.settings.apps[index].display_name().to_owned();
                    let dialog = Dialog::new(
                        format!("Remove {name} from Mortise?"),
                        "The folder and its files stay as they are. Mortise forgets the \
                         app, its environments and its saved credentials."
                            .to_owned(),
                        "Remove app".to_owned(),
                    );
                    self.dialog = Some((dialog, Pending::RemoveApp(index)));
                    self.dialog_opened = true;
                }
            }
        }
        if first_run_add {
            self.adding = Some(AddPanel::empty());
        }
        if new_module {
            self.open_generate(None, MODULE_MAKE);
        }
        // An action of the drawer applies to the app it was drawn for, if that app
        // is still at its place on the rail.
        for (index, root, action) in drawer_actions {
            if self
                .settings
                .apps
                .get(index)
                .is_some_and(|entry| entry.root == root)
            {
                self.drawer_action(&ctx, index, action);
            }
        }
        if let Some(action) = add_action {
            self.add_action(&ctx, action);
        }
        #[cfg(target_os = "macos")]
        self.menu_actions(&ctx, menu_chosen);
    }

    /// Closing Mortise stops the run of each app, as Stop does, and waits for the
    /// ends, so no command goes on after the window (design D6 of phase-2-hardening).
    /// No dialog asks. eframe calls this on every way out, Cmd Q on macOS too, and
    /// the process ends only after it returns.
    fn on_exit(&mut self) {
        for open in &self.apps {
            open.runs.stop();
            if let Some(reading) = &open.reading {
                reading.read.stop();
            }
        }
        let deadline = std::time::Instant::now() + mortise_core::STOP_WAIT;
        for open in &mut self.apps {
            open.runs.wait(deadline);
            if let Some(reading) = &mut open.reading {
                reading.read.wait(deadline);
            }
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
fn open_app(ctx: &egui::Context, entry: &AppEntry, data_dir: Option<&Path>) -> OpenApp {
    let ctx = ctx.clone();
    OpenApp {
        live: LiveApp::open(
            entry.root.clone(),
            entry.codes.clone(),
            read_options(entry, &PackageData::default()),
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
        php_test: crate::php_settings::PhpTest::default(),
        since: std::time::Instant::now(),
        runs: AppRuns::open(data_dir, &entry.root),
        actions: ActionsState::default(),
        action_run: None,
        generate: GenerateState::default(),
        generate_run: None,
        select_new: None,
        package: PackageData::default(),
        reading: None,
        migration_error: None,
        catalog: AppCatalog::default(),
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

/// True when the operating system is set to reduce motion.
///
/// macOS is asked. On Windows and Linux neither egui nor winit gives the setting,
/// and reading it needs code this crate does not allow: there "System" moves as "Full".
#[cfg(target_os = "macos")]
fn system_reduces_motion() -> bool {
    objc2_app_kit::NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
}

#[cfg(not(target_os = "macos"))]
fn system_reduces_motion() -> bool {
    false
}

/// The toast of Run tests from the document of `module:test --json` on `target`
/// (design D8), or `None` when the document has no counts. A module that ran
/// without counts, such as when PHPUnit stopped before its report, leaves the
/// totals short: then there is no count either. Nor is there when no test failed
/// and the exit code says the run failed (a warning under `failOnWarning`): the
/// count cannot say what failed.
fn tests_toast(report: &TestReport, target: &Target) -> Option<String> {
    let object = match target {
        Target::Module(name) => {
            let listed = report
                .modules
                .iter()
                .find(|module| module.name.eq_ignore_ascii_case(name));
            if listed.is_some_and(|module| !module.ran) {
                return Some(strings::no_tests_folder(name));
            }
            name.clone()
        }
        Target::All => match report.modules.iter().filter(|module| module.ran).count() {
            0 => return None,
            n => strings::modules(n),
        },
    };
    if report
        .modules
        .iter()
        .any(|module| module.ran && (module.passed.is_none() || module.failed.is_none()))
    {
        return None;
    }
    let (passed, failed) = (report.passed?, report.failed?);
    if failed == 0 {
        if report.exit_code.is_some_and(|code| code != 0) {
            return None;
        }
        return Some(strings::tests_passed(passed, &object));
    }
    let total = passed + failed + report.skipped.unwrap_or(0);
    Some(strings::tests_failed(failed, total, &object))
}

/// The names a dialog lists at most: the migrations of a roll back, the tables of
/// a reset or a refresh. With the footer, the dialog fits in a window 600 px high.
const DIALOG_NAMES: usize = 6;

/// At most [`DIALOG_NAMES`] of `names` joined by `separator`, then "and 24 more"
/// (not in the spec): on a line of its own after a line break, else after the last name.
fn bounded(names: &[String], separator: &str) -> String {
    let mut text = names
        .iter()
        .take(DIALOG_NAMES)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(separator);
    if names.len() > DIALOG_NAMES {
        let more = strings::and_more(names.len() - DIALOG_NAMES);
        if separator == "\n" {
            text.push('\n');
            text.push_str(more.trim_start());
        } else {
            text.push_str(&more);
        }
    }
    text
}

/// The action and the target of artisan words of reset, refresh, fresh or delete.
fn destructive_words(args: &[String]) -> Option<(Action, Target)> {
    let action = Action::from_artisan(args.first()?).filter(|action| action.is_destructive())?;
    Some((action, words_target(args)?))
}

/// The target of artisan words: `--all`, or the module, which is the second word
/// or the word after `--`.
fn words_target(args: &[String]) -> Option<Target> {
    if args
        .iter()
        .take_while(|word| *word != "--")
        .any(|word| word == "--all")
    {
        Some(Target::All)
    } else {
        runner_view::module_named(args).map(Target::Module)
    }
}

/// True when artisan words run `action` with `--json`: its answer is a document.
fn is_json_run(args: &[String], action: Action) -> bool {
    args.first().and_then(|word| Action::from_artisan(word)) == Some(action)
        && mortise_core::has_option(args, "--json")
}

/// True when the line that runs `args` carries `--force` before a `--`: in the
/// words of the runner, such as a Custom line `php artisan --force`, or in the run's.
fn forced(runner: Option<&RunnerKind>, args: &[String]) -> bool {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let line = runner
        .and_then(|runner| mortise_core::CommandLine::artisan(runner, &words))
        .map_or_else(|| args.to_vec(), |line| line.args);
    line.iter()
        .take_while(|word| *word != "--")
        .any(|word| word == "--force")
}

/// What a dialog waits to do.
enum Pending {
    /// Write the rules to a `mortise.json` that does not exist yet, in the app at `root`.
    CreateRules { root: PathBuf, rules: Rules },
    /// Forget the app at this place on the rail.
    RemoveApp(usize),
    /// Stop the command that runs in the app at this place on the rail.
    StopRun(usize),
    /// Start an action in the app at this place on the rail.
    Action { index: usize, act: Act },
    /// Run these words again in the app at this place on the rail.
    RunAgain { index: usize, args: Vec<String> },
    /// Start a generator in the app at this place on the rail.
    Generate { index: usize, run: GenerateRun },
}

/// How wide the module index is for a window of this width (section 5.14), and
/// whether it is its strip. The workspace is what the user came for, so the index
/// gives up width before the workspace does.
/// `set` is the width the user gave the index, which it has when there is room.
fn index_width(window_w: f32, choice: Option<bool>, set: f32) -> (f32, bool) {
    let narrow = window_w < 1032.0;
    let set = set.clamp(size::INDEX_W_MIN, size::INDEX_W_MAX);
    match choice.unwrap_or(narrow) {
        true => (size::INDEX_STRIP_W, true),
        // Opened by hand in a narrow window: the workspace gives way.
        false if narrow => (size::INDEX_W_MIN, false),
        false => {
            let left = window_w - size::RAIL_W - 760.0;
            (left.clamp(size::INDEX_W_MIN, set), false)
        }
    }
}

/// The right edge of the index as a handle: dragging it, or Left and Right while it
/// has focus, sets the width of the index between 220 and 320 px. Returns the new
/// width while it changes, and true when the change is to be kept.
fn index_edge(
    ui: &Ui,
    index_rect: Rect,
    keyboard: bool,
    pal: &theme::Palette,
) -> Option<(f32, bool)> {
    let zone = Rect::from_min_max(
        pos2(index_rect.right() - 3.0, index_rect.top()),
        pos2(index_rect.right() + 3.0, index_rect.bottom()),
    );
    let id = ui.scope_id().with("index-edge");
    let response = ui.interact(zone, id, egui::Sense::click_and_drag());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::Splitter, true, strings::INDEX_WIDTH)
    });
    if response.hovered() || response.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    let clamp = |width: f32| width.clamp(size::INDEX_W_MIN, size::INDEX_W_MAX).round();
    let mut change = None;
    if response.dragged()
        && let Some(pointer) = response.interact_pointer_pos()
    {
        change = Some((clamp(pointer.x - index_rect.left()), false));
    }
    if response.drag_stopped() {
        change = Some((clamp(index_rect.width()), true));
    }
    if response.has_focus() {
        // Nothing is reachable only by dragging: the arrows move the edge too.
        draw::keep_arrows(ui, id, false, true);
        let (left, right) = ui.input_mut(|input| {
            (
                input.consume_key(Modifiers::NONE, Key::ArrowLeft),
                input.consume_key(Modifiers::NONE, Key::ArrowRight),
            )
        });
        let step = match (left, right) {
            (true, _) => -8.0,
            (_, true) => 8.0,
            _ => 0.0,
        };
        if step != 0.0 {
            change = Some((clamp(index_rect.width() + step), true));
        }
        if keyboard {
            draw::focus_ring(ui, pal, zone.shrink2(vec2(2.0, 0.0)), 0);
        }
    }
    change
}

/// What the user set for an app, as the reader takes it.
fn read_options(entry: &AppEntry, package: &PackageData) -> ReadOptions {
    ReadOptions {
        coverage_limit: entry.coverage_limit,
        coverage_report: entry.coverage_report.as_ref().map(PathBuf::from),
        package: package.clone(),
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

/// The icon of the window and the taskbar, and of the Dock on macOS: eframe sets it
/// even in a bundle, and shows the egui logo without it. macOS takes the drawing on
/// its own grid. `tools/app-icon.mjs` makes both files.
fn app_icon() -> egui::IconData {
    #[cfg(target_os = "macos")]
    let png = include_bytes!("../../../assets/icon/macos/mortise-512.png");
    #[cfg(not(target_os = "macos"))]
    let png = include_bytes!("../../../assets/icon/linux/hicolor/256x256/apps/mortise.png");
    eframe::icon_data::from_png_bytes(png).expect("the app icon is a valid PNG")
}

/// Starts the app. `folder` is an app folder to open, if any.
pub fn run(chrome: Chrome, folder: Option<PathBuf>) -> eframe::Result {
    let viewport = chrome.viewport(
        egui::ViewportBuilder::default()
            .with_title(strings::BRAND)
            .with_app_id("mortise")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([size::WINDOW_MIN_W, size::WINDOW_MIN_H])
            .with_drag_and_drop(true)
            .with_icon(app_icon()),
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
                quick_read: std::time::Duration::from_millis(60),
                data_dir: mortise_core::history::data_dir(),
            };
            let app = MortiseApp::new(cc, config);
            // The system menu bar: Mortise, File, View, Window and Help.
            #[cfg(target_os = "macos")]
            let app = app.with_menu_bar(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The document of a recording in `fixtures/output`.
    fn recorded(name: &str) -> TestReport {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/output")
            .join(format!("{name}.json"));
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let stdout: Vec<String> = json["stdout"]
            .as_str()
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();
        mortise_core::document(&stdout).unwrap()
    }

    #[test]
    fn the_toast_of_run_tests_counts_from_the_document() {
        let checkout = Target::Module("Checkout".to_owned());
        assert_eq!(
            tests_toast(&recorded("module-test-fail"), &checkout).as_deref(),
            Some("1 of 1 test failed in Checkout.")
        );
        let mut all = recorded("module-test-all");
        assert_eq!(
            tests_toast(&all, &Target::All).as_deref(),
            Some("1 of 2 tests failed in 3 modules.")
        );
        // No test failed and the run failed: the count cannot say why.
        all.failed = Some(0);
        assert_eq!(tests_toast(&all, &Target::All), None);
        // A module that ran without counts leaves the totals short: no count.
        all.exit_code = Some(0);
        all.modules[1].passed = None;
        assert_eq!(tests_toast(&all, &Target::All), None);
        let mut report = recorded("module-test-pass");
        report.modules[0].ran = false;
        assert_eq!(
            tests_toast(&report, &checkout).as_deref(),
            Some("Checkout has no tests folder.")
        );
        // Without counts the toasts of an action without data apply.
        report.modules[0].ran = true;
        report.passed = None;
        assert_eq!(tests_toast(&report, &checkout), None);
    }

    #[test]
    fn a_dialog_lists_six_names_then_counts_the_rest() {
        let names: Vec<String> = (1..=30).map(|n| format!("t{n}")).collect();
        assert_eq!(bounded(&names, "\n"), "t1\nt2\nt3\nt4\nt5\nt6\nand 24 more");
        assert_eq!(
            bounded(&names[..7], ", "),
            "t1, t2, t3, t4, t5, t6 and 1 more"
        );
        assert_eq!(bounded(&names[..2], ", "), "t1, t2");
    }
}
