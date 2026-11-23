//! The app: its state, the keys, and the three columns of the workbench.

pub mod chrome;
#[cfg(target_os = "macos")]
pub mod menu;
pub mod rail;
pub mod workspace;

mod actions;
mod apps;
mod generate;
mod keys;
mod palette;
mod runs;
mod runtime;
mod settings;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, Key, Modifiers, Rect, Theme, Ui, pos2, vec2};
use mortise_core::{AppEntry, AppSnapshot, LiveApp, LiveState, Settings, SettingsError};
use mortise_core::{
    Catalog, Environment, LiveRuntime, MODULE_MAKE, PackageData, ReadOptions, Rules, Runs,
    RuntimeState, Secret, Secrets, Target, Version,
};

use crate::actions::view::{self as actions_view, Act, ActionsInputs, ActionsState};
use crate::app::chrome::Chrome;
use crate::app::generate::{AppCatalog, Generating, SelectNew};
use crate::app::rail::{RailAction, Tile};
use crate::app::workspace::{Tab, WorkAction, WorkState};
use crate::apps::view::{self as add, AddPanel, Checked};
use crate::boundary::surface::SurfaceInputs;
use crate::boundary::view::BoundaryState;
use crate::generate::view::{GenerateAsk, GenerateInputs, GenerateRun, GenerateState, RunPlace};
use crate::index::view::{self as index, IndexRuntime, IndexState, Selection};
use crate::palette::view::{self as palette_view, PaletteAction, PaletteState};
use crate::runner::view::{self as runner_view, AppRuns, RunnerView};
use crate::runtime::view::{Brief, RuntimeView};
use crate::settings::view::{
    self as settings_view, AppInfo, Section, SettingsAction, SettingsState,
};
use crate::strings;
use crate::summary::view::{Sort, SummaryState};
use crate::ui::dialog::{Answer, Dialog};
use crate::ui::draw::EmptyMark;
use crate::ui::draw::{self, Button, ButtonKind};
use crate::ui::theme::{self, radius, size};
use crate::ui::toast::Toast;

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
    php_test: crate::runner::php_settings::PhpTest,
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
    /// Why the last read of the migration state gave nothing, with its target and
    /// connection. The state box says so; the state stays as it was (design D2).
    migration_error: Option<actions_view::StateError>,
    /// The runner and working folder of the last read of the migration state. Another
    /// one may reach another database: the state is forgotten when they change (design D7).
    state_place: Option<RunPlace>,
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
    /// `--database`, empty for the default. With the runner and working folder it
    /// started with.
    Migrations {
        target: Target,
        connection: String,
        place: RunPlace,
    },
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
    menu: Option<crate::app::menu::MenuBar>,
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
        crate::ui::fonts::install(&cc.egui_ctx);
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

    fn save(&mut self) {
        let Some(path) = &self.settings_path else {
            return;
        };
        if let Err(error) = self.settings.save(path) {
            self.toast = Some(Toast::failed(format!("{error}.")));
        }
    }

    /// The snapshot of the app at `index`, once it is read.
    fn snapshot(&self, index: usize) -> Option<Arc<AppSnapshot>> {
        match self.apps.get(index)?.live.state() {
            LiveState::Ready(app) => Some(app),
            _ => None,
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
        self.forget_state_of_another_place();
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
                        Some(WorkAction::Select(module)) => {
                            open.selection = Selection::Module(module.clone());
                            self.reveal_next = true;
                        }
                        Some(WorkAction::ShowFindings) => {
                            open.findings_open = true;
                            open.finding_row = 0;
                        }
                        Some(WorkAction::ShowLink { module, other }) => {
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
                    draw::empty_state(ui, pal, rest, EmptyMark::None, &title, &[&text], 0.0);
                }
            }
        } else {
            // First run: no apps yet.
            ui.painter().rect_filled(rest, 0, pal.surface);
            let at = draw::empty_state(
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
            let source = palette_view::Source {
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
            palette_action = palette_view::palette(ui, pal, palette, &source, scope_name);
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
            Some(WorkAction::SetRules(rules)) => self.set_rules(rules),
            Some(WorkAction::CopyPath(path)) => {
                ctx.copy_text(path);
                self.toast = Some(Toast::done(strings::COPIED_PATH));
            }
            Some(WorkAction::OpenInEditor { path, line }) => self.open_in_editor(&path, line),
            Some(WorkAction::Runtime(action)) => self.runtime_action(&ctx, action),
            Some(WorkAction::Copy { text, said }) => {
                ctx.copy_text(text);
                self.toast = Some(Toast::done(said));
            }
            Some(WorkAction::OpenAction(action)) => {
                let module = match self.selection() {
                    Selection::Module(name) => Some(name),
                    Selection::All => None,
                };
                self.open_action(module, action);
            }
            Some(WorkAction::OpenModuleAction { module, action }) => {
                self.open_action(Some(module), action);
            }
            Some(WorkAction::Act(act)) => {
                if let Some(current) = self.current {
                    self.act(current, act);
                }
            }
            Some(WorkAction::ReadMigrations { target, args }) => {
                if let Some(current) = self.current {
                    self.read_migrations(current, target, args);
                }
            }
            Some(WorkAction::Generate(GenerateAsk::NewModule)) => {
                self.open_generate(None, MODULE_MAKE)
            }
            Some(WorkAction::Generate(ask)) => {
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
        php_test: crate::runner::php_settings::PhpTest::default(),
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
        state_place: None,
        catalog: AppCatalog::default(),
    }
}

/// Where the commands of an app with these PHP settings run.
fn place_of(php: &mortise_core::PhpSettings) -> RunPlace {
    RunPlace {
        runner: php.runner.clone(),
        folder: php.working_folder.clone(),
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
    let png = include_bytes!("../../../../assets/icon/macos/mortise-512.png");
    #[cfg(not(target_os = "macos"))]
    let png = include_bytes!("../../../../assets/icon/linux/hicolor/256x256/apps/mortise.png");
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
