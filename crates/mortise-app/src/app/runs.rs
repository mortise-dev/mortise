//! Runs: start, stop and run again, the reads of package data, and what the drawer asks for.

use eframe::egui;
use mortise_core::{Action, Ask, ModuleState, PackageData, RunEnd, Target};

use super::{MortiseApp, PackageRead, Pending, ReadFor, place_of, read_options, words_target};
use crate::actions::view::{self as actions_view, Act};
use crate::generate::view::RunPlace;
use crate::runner::strings;
use crate::runner::view::{self as runner_view, DrawerAction, Shown};
use crate::settings::view::Section;
use crate::ui::dialog::Dialog;
use crate::ui::toast::ToastAction;

impl MortiseApp {
    /// Changes the package data of the app at `index` and reads the app again, so
    /// that its snapshot holds the new data (design D7).
    pub(super) fn set_package(&mut self, index: usize, change: impl FnOnce(&mut PackageData)) {
        let (Some(open), Some(entry)) = (self.apps.get_mut(index), self.settings.apps.get(index))
        else {
            return;
        };
        change(&mut open.package);
        open.live.set_options(read_options(entry, &open.package));
    }

    /// Starts a read of package data in the app at `index` (design D3). It does not
    /// open the drawer. A read that cannot start ends at once with its error.
    pub(super) fn start_read(
        &mut self,
        index: usize,
        args: Vec<String>,
        input: Option<Vec<u8>>,
        purpose: ReadFor,
    ) {
        let (Some(open), Some(entry)) = (self.apps.get(index), self.settings.apps.get(index))
        else {
            return;
        };
        if open.busy().is_some() {
            return;
        }
        match mortise_core::Read::start_with_input(
            &self.runs,
            &entry.root,
            &entry.php,
            &args,
            input,
        ) {
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
    pub(super) fn read_ended(
        &mut self,
        index: usize,
        purpose: ReadFor,
        end: Result<mortise_core::ReadEnd, mortise_core::StartError>,
    ) {
        match purpose {
            ReadFor::Catalog { package } => self.catalog_read(index, &package, end),
            ReadFor::DryRun { words, place } => self.dry_run_read(index, words, place, end),
            ReadFor::Migrations {
                target,
                connection,
                place,
            } => self.migrations_read(index, target, &connection, place, end),
            ReadFor::Surface { module, surface } => {
                self.surface_read(index, &module, &surface, end)
            }
        }
    }

    /// Starts a read of the migration state of `target` with `words`, on the
    /// connection their `--database` names (design D7).
    pub(super) fn read_migrations(&mut self, index: usize, target: Target, words: Vec<String>) {
        let connection = mortise_core::Options::from_args(&words).database;
        let Some(entry) = self.settings.apps.get(index) else {
            return;
        };
        let place = place_of(&entry.php);
        let purpose = ReadFor::Migrations {
            target,
            connection,
            place,
        };
        self.start_read(index, words, None, purpose);
    }

    /// Takes in a read of the migration state of `target` on `connection` (design
    /// D7): its document merges into the app's package data. Otherwise the state box
    /// says why, and the state stays as it was (design D2). A read that started with
    /// another runner or working folder may have read another database: its answer,
    /// or its error, is dropped.
    fn migrations_read(
        &mut self,
        index: usize,
        target: Target,
        connection: &str,
        place: RunPlace,
        end: Result<mortise_core::ReadEnd, mortise_core::StartError>,
    ) {
        if self
            .settings
            .apps
            .get(index)
            .map(|entry| place_of(&entry.php))
            .as_ref()
            != Some(&place)
        {
            return;
        }
        let status = match end {
            Ok(end) => end
                .document::<mortise_core::MigrationStatus>()
                .map_err(|error| crate::strings::no_document(&end, &error)),
            Err(error) => Err(crate::strings::start_reason(&error)),
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
        open.state_place = Some(place);
        let status = match status {
            Ok(status) => status,
            Err(reason) => {
                open.migration_error = Some(actions_view::StateError {
                    target,
                    connection: connection.trim().to_owned(),
                    reason,
                });
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

    /// Forgets the whole migration state of each app whose runner or working folder
    /// changed since its last state read, and why that read failed: another runner or
    /// folder may reach another database (design D7).
    pub(super) fn forget_state_of_another_place(&mut self) {
        for index in 0..self.apps.len() {
            let place = self
                .settings
                .apps
                .get(index)
                .map(|entry| place_of(&entry.php));
            let open = &mut self.apps[index];
            if open.state_place.is_none() || open.state_place == place {
                continue;
            }
            open.state_place = None;
            open.migration_error = None;
            self.set_package(index, |package| {
                package.forget_migrations(&Target::All);
            });
        }
    }

    /// After a run of the app at `index` ended, with these words, from wherever it
    /// started (design D7). A migrate, roll back, reset, refresh or fresh, whatever its
    /// exit code, forgets the migration state it may have changed: the box then reads
    /// "Not read yet." again. A delete that exited with 0 forgets its module.
    pub(super) fn after_run(&mut self, index: usize, args: &[String]) {
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

    // --- Runs ---

    /// Starts artisan with `args` in the app at `index`, through its current runner.
    /// The drawer opens on it; it does not take focus.
    pub(super) fn start_run(&mut self, index: usize, args: Vec<String>) {
        let Some(entry) = self.settings.apps.get(index) else {
            return;
        };
        let (root, php) = (entry.root.clone(), entry.php.clone());
        self.apps[index].runs.start(&self.runs, &root, &php, args);
        self.runner.shown = Shown::Open;
        self.runner.history_open = false;
    }

    /// Stops the command of the app at `index`. A migration, a rollback or a seed asks first.
    pub(super) fn ask_stop(&mut self, index: usize) {
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
    pub(super) fn run_again(&mut self, index: usize) {
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
    pub(super) fn drawer_action(
        &mut self,
        ctx: &egui::Context,
        index: usize,
        action: DrawerAction,
    ) {
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

    /// What the button of a toast asked for.
    pub(super) fn toast_action(&mut self, ctx: &eframe::egui::Context, action: ToastAction) {
        match action {
            ToastAction::InstallUpdate => self.install_update(ctx),
            ToastAction::OpenDownloadPage => self.open_download_page(ctx),
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
}

/// The action and the target of artisan words of reset, refresh, fresh or delete.
fn destructive_words(args: &[String]) -> Option<(Action, Target)> {
    let action = Action::from_artisan(args.first()?).filter(|action| action.is_destructive())?;
    Some((action, words_target(args)?))
}
