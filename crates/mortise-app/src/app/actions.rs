//! Actions: act on a module, the dialogs that ask first, refusals, and the end of an action.

use mortise_core::{Action, Ask, LangReport, ModuleState, RunEnd, Target, TestReport};

use super::{MortiseApp, Pending, words_target};
use crate::actions::view::{self as actions_view, Act, ActionsState};
use crate::app::workspace::Tab;
use crate::index::view::Selection;
use crate::runner::view::{self as runner_view, Shown};
use crate::strings;
use crate::ui::dialog::{Dialog, Typed};
use crate::ui::toast::{Toast, ToastAction};

impl MortiseApp {
    // --- Actions ---

    /// Selects `module` in the index, or "All modules" for `None`, opens the Actions
    /// tab and selects `action` there. Nothing runs: the line shows first (deviation S40).
    pub(super) fn open_action(&mut self, module: Option<String>, action: Action) {
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
    pub(super) fn act(&mut self, index: usize, act: Act) {
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
    pub(super) fn refused(&self, index: usize, action: Action) -> bool {
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
    pub(super) fn show_refused(
        &mut self,
        index: usize,
        action: Action,
        target: Target,
        args: &[String],
    ) {
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
    pub(super) fn check_dry_run(&mut self, index: usize, act: &Act) {
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

    /// Starts the words of an action and keeps what it was, for the toast at its end.
    pub(super) fn start_action(&mut self, index: usize, act: Act) {
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
    pub(super) fn action_ended(&mut self, index: usize, act: &Act, modules_on: usize) {
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
    /// that toast only when the drawer does not show it. A translation report that
    /// cannot be read gets a failed toast with the reason: Findings keep the last one.
    pub(super) fn documents_ended(&mut self, index: usize, args: &[String]) {
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
        let Some(report) = self.apps[index].runs.ended().map(|over| {
            mortise_core::whole_document::<LangReport>(over.stdout, over.stdout_cut)
                .map_err(|error| strings::no_run_document(&error, over.end))
        }) else {
            return;
        };
        match report {
            Ok(report) => self.set_package(index, |data| data.merge_translations(report)),
            Err(reason) => self.failed_toast(index, strings::translations_not_read(&reason)),
        }
    }
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

/// True when artisan words run `action` with `--json`: its answer is a document.
fn is_json_run(args: &[String], action: Action) -> bool {
    args.first().and_then(|word| Action::from_artisan(word)) == Some(action)
        && mortise_core::has_option(args, "--json")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

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
