//! Generate: the generators read from the app, a run of a generator, and its end.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mortise_core::{
    AppSnapshot, Catalog, FileOutcome, MODULE_MAKE, RunEnd, RunnerKind, Version, gives_data,
};

use super::{MortiseApp, Pending, ReadFor, place_of};
use crate::app::workspace::Tab;
use crate::generate::view::{
    CatalogRead, GenerateAsk, GenerateResult, GenerateRun, RunPlace, TakenOut, typed_names,
};
use crate::index::view::Selection;
use crate::runner::view::{self as runner_view, Shown};
use crate::strings;
use crate::ui::dialog::Dialog;
use crate::ui::toast::{Toast, ToastAction};

/// The generators read from a 13.4 app (design D4): in this session, or from the
/// cache in Mortise's data folder.
#[derive(Debug, Default)]
pub(super) struct AppCatalog {
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
    pub(super) fn in_use(&self, package: &Version) -> &Catalog {
        self.read_for(package)
            .map_or_else(|| Catalog::shipped(package), |(catalog, _)| catalog)
    }

    /// What the picker says about the read, on a 13.4 app only.
    pub(super) fn picker(&self, package: &Version, reading: bool) -> Option<CatalogRead<'_>> {
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
    pub(super) fn look_up(&mut self, data_dir: Option<&Path>, root: &Path, package: &Version) {
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

/// A generator that runs, and what its end needs.
pub(super) struct Generating {
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
pub(super) struct SelectNew {
    before: BTreeSet<String>,
    names: Vec<String>,
}

impl SelectNew {
    pub(super) fn find(&self, app: &AppSnapshot) -> Option<String> {
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

impl MortiseApp {
    /// The end of `module:generators --json` (design D4): the catalog replaces the
    /// shipped one for the app and goes into the cache, or the picker says why not.
    /// `package` is the version the app had when the read started. When the app's
    /// version changed meanwhile, the answer may be of either version: it is dropped,
    /// and the picker shows the shipped catalog of the new version.
    pub(super) fn catalog_read(
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
    pub(super) fn dry_run_read(
        &mut self,
        index: usize,
        words: Vec<String>,
        place: RunPlace,
        end: Result<mortise_core::ReadEnd, mortise_core::StartError>,
    ) {
        let Some(app) = self.snapshot(index) else {
            return;
        };
        if place_of(&self.settings.apps[index].php) != place {
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

    // --- Generate ---

    /// Opens the form of `generator` on the Generate tab: in `module`, or for
    /// `module:make` with "All modules" selected (design D7). Nothing runs.
    pub(super) fn open_generate(&mut self, module: Option<String>, generator: &str) {
        let Some(current) = self.current else {
            return;
        };
        // A generator without a module argument opens under "All modules" too.
        let takes_module = self.snapshot(current).is_none_or(|app| {
            self.apps[current]
                .catalog
                .in_use(&app.package)
                .get(generator)
                .is_none_or(crate::generate::view::takes_module)
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
    pub(super) fn generate(&mut self, index: usize, ask: GenerateAsk) {
        let runner = self.settings.apps[index].php.runner.as_ref();
        let line = |args: &[String]| {
            runner_view::line_now(runner, args)
                .unwrap_or_else(|| format!("php artisan {}", args.join(" ")))
        };
        let dialog = match &ask {
            GenerateAsk::NewModule => return,
            GenerateAsk::ReadCatalog => {
                let words = crate::generate::view::read_catalog_words();
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
                let place = place_of(&self.settings.apps[index].php);
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
    pub(super) fn start_generate(&mut self, index: usize, run: GenerateRun) {
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
    pub(super) fn generate_ended(&mut self, index: usize, generating: Generating) {
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
            .map(crate::generate::view::module_folders)
            .unwrap_or_default();
        // The modules the run writes in place its paths first.
        let asked = crate::generate::view::run_modules(
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
