//! The Generate tab (S9.1 to S9.6): the picker of the package's generators, the
//! form of one generator with the exact line its button starts, the file list of
//! its dry run on a 13.4 app, and the result of a run, read from its answer or its
//! output.
//!
//! Nothing here starts a process. The tab hands the app a [`GenerateAsk`] with the
//! words on screen; the app asks first where the rules say so, then starts them.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{
    self, Align2, Color32, FontId, Key, KeyboardShortcut, Modifiers, Pos2, Rect, RichText, Role,
    ScrollArea, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2, vec2,
};
use mortise_core::{
    AppSnapshot, Catalog, FileAction, FileOutcome, Filled, GeneratedFile, Generator,
    GeneratorReport, Group, MODULE_MAKE, Module, NameRule, ReportedFile, RunnerKind, Severity,
    app_relative, generator_gives_data, model_flag_of, replace_words, shipped_mark, with_dry_run,
    with_json,
};

use crate::boundary::BoundaryAction;
use crate::draw::{self, Button, ButtonKind, Icon};
use crate::runner_view::line_now;
use crate::strings;
use crate::summary::alert_box;
use crate::theme::{Face, Palette, Type, radius, size, ty};

const MAKE_MODEL: &str = "module:make-model";
/// The flag of `module:make-model` that S9.3 never takes out: the model's stub reads it.
const FACTORY: &str = "factory";
/// The files of `module:make-model` under "Also create", in the order of S9.2.
const MODEL_CALLS: [&str; 7] = [
    "controller",
    "factory",
    "migration",
    "request",
    "resource",
    "policy",
    "seed",
];
/// The options a form does not offer: Replace is the way for `--force` (S9.6), and
/// Mortise adds `--dry-run` and `--json` itself (design D4).
const NOT_OFFERED: [&str; 3] = ["force", "dry-run", "json"];
/// The gap between the columns of the picker and of a form.
const GAP: f32 = 14.0;
/// A row of the picker: 26 px and a hairline under it.
const ITEM_H: f32 = 26.0;
/// The heading of a group of the picker, with its hairline.
const HEADING_H: f32 = 26.0;
/// The filter field of the picker is 300 px wide (S9.1).
const FILTER_W: f32 = 300.0;
/// A row of a result is at least 30 px high.
const RESULT_ROW_H: f32 = 30.0;
/// A new row of a result carries the well fill this long, then fades (S9.5).
const NEW_FILL_S: f64 = 0.6;
const NEW_FADE_S: f64 = 0.3;

/// The type of a hint and a sentence under a control.
const HINT: Type = Type {
    line: 16.0,
    ..ty::SECONDARY
};
/// A path in a row of a result.
const PATH: Type = Type {
    size: 11.5,
    line: 15.0,
    ..ty::CODE
};

/// What the tab shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Picker,
    Form,
}

/// A generator run to start: the words of the line on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateRun {
    /// The artisan name of the generator, the first word.
    pub generator: String,
    /// The module it writes in. `None` for `module:make`.
    pub module: Option<String>,
    /// The artisan words, exactly as the line on screen shows them.
    pub args: Vec<String>,
    /// The files whose flags S9.3 took out of the command: the run leaves them.
    pub taken_out: Vec<TakenOut>,
    /// The path of the left row of the result on screen that this run replaces. Its
    /// end updates that row, and the other rows keep their Replace.
    pub row: Option<String>,
}

/// A file that S9.3 took out of the command because it exists (design D5). The run
/// leaves it as it is, and its result lists it with its own Replace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TakenOut {
    /// The path as the dry run printed it, or relative to the app once in a result.
    pub path: String,
    /// The words that write it with `--force`: `replace_words` on the words of the
    /// form before its flag was taken out.
    pub replace: Option<Vec<String>>,
}

/// What the tab asks the app for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateAsk {
    /// Start a generator, or ask first (Replace the module).
    Run(GenerateRun),
    /// Ask before a file that a run left is written again.
    Replace { file: String, run: GenerateRun },
    /// Select "All modules" and open the new module form.
    NewModule,
    /// Read the generators of the app: `module:generators --json` (design D4).
    ReadCatalog,
    /// The dry run of the form: these words with `--dry-run --json` (design D5).
    ListFiles { words: Vec<String> },
}

/// The artisan words of the read of the app's generators (P3).
pub fn read_catalog_words() -> Vec<String> {
    vec!["module:generators".to_owned(), "--json".to_owned()]
}

/// What the picker of a 13.4 app says about the read of its generators (design D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogRead<'a> {
    /// Not read in this session, and no cache for the app's version.
    NotRead,
    /// The read runs.
    Reading,
    /// The list is the one read at this time, in ms since 1970.
    Read(i64),
    /// The last read gave no list: why.
    Failed(&'a str),
}

/// The result of a generator run, read from its output (S9.5, S9.6).
#[derive(Debug, Clone)]
pub struct GenerateResult {
    pub module: Option<String>,
    /// The words that ran.
    pub args: Vec<String>,
    pub files: Vec<GeneratedFile>,
    /// `module:make` says that `composer dump-autoload` failed.
    pub autoload_failed: bool,
    /// Why the run gave no answer, when the files were read from its output (design D6).
    fallback: Option<String>,
    /// The files S9.3 took out of the command, listed in `files` as left.
    taken_out: Vec<TakenOut>,
    /// When it was first painted, in seconds of egui's clock.
    shown: Option<f64>,
}

impl GenerateResult {
    pub fn new(
        module: Option<String>,
        args: Vec<String>,
        files: Vec<GeneratedFile>,
        autoload_failed: bool,
    ) -> Self {
        Self {
            module,
            args,
            files,
            autoload_failed,
            fallback: None,
            taken_out: Vec::new(),
            shown: None,
        }
    }

    /// The files S9.3 took out of the run's command, which `files` lists as left. Each
    /// Replace of them runs its own words.
    pub fn with_taken_out(mut self, taken_out: Vec<TakenOut>) -> Self {
        self.taken_out = taken_out;
        self
    }

    /// The result read from the output lines, because the run gave no answer: why.
    pub fn read_from_output(mut self, reason: Option<String>) -> Self {
        self.fallback = reason;
        self
    }

    fn written(&self) -> impl Iterator<Item = &GeneratedFile> {
        self.files
            .iter()
            .filter(|file| matches!(file.outcome, FileOutcome::Created | FileOutcome::Replaced))
    }
}

/// Where a command of the app runs: its PHP runner and its working folder. A dry run
/// lists the files of the app it ran in, so its list belongs to its place (design D5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPlace {
    pub runner: Option<RunnerKind>,
    pub folder: Option<PathBuf>,
}

/// The file list of a dry run (P2, design D5), or why it gave none. It belongs to
/// the words of the form it ran with: another line drops it. It belongs to the
/// place it ran in too: another runner or working folder drops it.
#[derive(Debug, Clone)]
struct FileList {
    /// The words of the form, without `--dry-run --json`.
    words: Vec<String>,
    /// Where the dry run ran.
    place: RunPlace,
    /// The words the dry run ran with, before S9.3 took flags out.
    before: Vec<String>,
    listed: Result<GeneratorReport, String>,
    /// The flags of the model form taken out because their file exists (S9.3).
    taken_out: Vec<&'static str>,
    /// S9.3 took `all` out of the form, for the other six flags.
    all_taken_out: bool,
}

impl FileList {
    /// The flags of the model form whose files exist: those taken out, and the
    /// factory's, which stays in the command (S9.3).
    fn existing(&self, model: &str) -> Vec<&'static str> {
        let mut flags = self.taken_out.clone();
        let factory = self.listed.as_ref().is_ok_and(|report| {
            report.files.iter().any(|file| {
                file.action == FileAction::Keep && model_flag_of(model, &file.path) == Some(FACTORY)
            })
        });
        if factory {
            flags.push(FACTORY);
        }
        flags
    }
}

/// What the Generate tab keeps for an open app, in memory.
#[derive(Debug, Default)]
pub struct GenerateState {
    pub view: View,
    /// The text of the picker's filter field.
    pub filter: String,
    /// The current row of the picker, by artisan name. `None` is the first file
    /// generator listed.
    current: Option<String>,
    /// The generator of the form, by artisan name.
    pub generator: String,
    /// The generator takes a module argument. One without (an app's own) writes no
    /// file through a module word, so its form may stay under "All modules".
    takes_module: bool,
    /// What the form holds. It stays while the module changes.
    pub filled: Filled,
    /// The words the form showed in the last frame it was drawn. Ctrl Enter runs
    /// only these, so a key never runs a line that was not on screen.
    last_shown: Option<Vec<String>>,
    /// The words of the read ("List files" or the read of the generators) whose line
    /// was whole on screen in the last frame. Its button starts only these.
    read_shown: Option<Vec<String>>,
    /// The result of the last run, in place of the line and its buttons.
    pub result: Option<GenerateResult>,
    /// The file list of the last dry run, or why it gave none.
    files: Option<FileList>,
    /// The module the tab was drawn for last: `None` until it is first drawn.
    module: Option<Option<String>>,
    focus_filter: bool,
    focus_name: bool,
    /// The result appeared: its first button takes focus, so Enter opens the file.
    focus_result: bool,
    filter_was_focused: bool,
}

impl GenerateState {
    /// The form of `generator`, with focus in its name field. Another generator
    /// starts with an empty form. `takes_module` is false for a generator without a
    /// module argument.
    pub fn open_form(&mut self, generator: &str, takes_module: bool) {
        if self.generator != generator {
            self.generator = generator.to_owned();
            self.filled = Filled::default();
        }
        self.takes_module = takes_module;
        self.view = View::Form;
        self.result = None;
        self.last_shown = None;
        self.read_shown = None;
        self.focus_name = true;
    }

    /// The picker, with focus in its filter field when `focus_filter`.
    pub fn open_picker(&mut self, focus_filter: bool) {
        self.view = View::Picker;
        self.focus_filter |= focus_filter;
    }

    /// Called every frame for the app on screen. Another module drops the result,
    /// which names its module; the form and its values stay (design D5). A form
    /// needs its scope: a file generator a module, `module:make` "All modules". A
    /// generator without a module argument opens under either.
    /// `shown` is false while the tab is not on screen, and then no line counts as shown.
    pub fn follow(&mut self, module: Option<&str>, shown: bool) {
        if self.module.as_ref().map(Option::as_deref) != Some(module) {
            if self.module.is_some() {
                self.result = None;
            }
            self.module = Some(module.map(str::to_owned));
        }
        let out_of_scope = if self.generator == MODULE_MAKE {
            module.is_some()
        } else {
            module.is_none() && self.takes_module
        };
        if self.view == View::Form && out_of_scope {
            self.view = View::Picker;
        }
        if !shown {
            self.last_shown = None;
            self.read_shown = None;
        }
    }

    /// Takes the end of the dry run of `words`, the words of the form of `generator`
    /// (design D5). On the model form a file that exists and that a flag writes takes
    /// that flag out of the form, and `--all` becomes the other six (S9.3); the list
    /// then belongs to the words without it. The factory is the exception: its flag
    /// stays, and the run leaves the file. A form that changed meanwhile drops it.
    /// `place` is where the dry run started.
    pub fn take_files(
        &mut self,
        generator: &Generator,
        words: Vec<String>,
        place: RunPlace,
        listed: Result<GeneratorReport, String>,
    ) {
        let module = self.module.clone().flatten();
        if self.generator != generator.name
            || generator.words(&self.filled, module.as_deref()) != words
        {
            return;
        }
        let mut taken_out = Vec::new();
        let mut all_taken_out = false;
        if let Ok(report) = &listed
            && generator.name == MAKE_MODEL
        {
            let model = self
                .filled
                .arguments
                .get("model")
                .cloned()
                .unwrap_or_default();
            let flags = &mut self.filled.flags;
            for file in report.files.iter().filter(|f| f.action == FileAction::Keep) {
                // The model's stub renders `HasFactory` from `--factory`, so that flag
                // stays. The factory call then fails on its file, and the model goes on
                // with its other calls and exits 1 after them (`ModelMakeCommand::handle`
                // and `callGenerator` at 13.4.0).
                let Some(flag) =
                    model_flag_of(model.trim(), &file.path).filter(|flag| *flag != FACTORY)
                else {
                    continue;
                };
                let all = flags.remove("all");
                all_taken_out |= all;
                if all || MODEL_CALLS.iter().all(|call| flags.contains(*call)) {
                    flags.extend(MODEL_CALLS.iter().map(|call| call.to_string()));
                }
                if flags.remove(flag) {
                    taken_out.push(flag);
                }
            }
        }
        self.files = Some(FileList {
            words: generator.words(&self.filled, module.as_deref()),
            place,
            before: words,
            listed,
            taken_out,
            all_taken_out,
        });
    }

    /// Drops a file list that ran in another place than `place`: it lists the files
    /// of another app.
    fn drop_files_of_another_place(&mut self, place: &RunPlace) {
        if self.files.as_ref().is_some_and(|list| list.place != *place) {
            self.drop_list();
        }
    }

    /// Drops the file list of the form, whose words or place changed. The flags S9.3
    /// took out for it are the list's, not the user's: they come back into the form,
    /// with `all` when S9.3 took it.
    fn drop_list(&mut self) {
        let Some(list) = self.files.take() else {
            return;
        };
        if list.before.first() != Some(&self.generator) {
            return;
        }
        let flags = &mut self.filled.flags;
        flags.extend(list.taken_out.iter().map(|flag| flag.to_string()));
        if list.all_taken_out && MODEL_CALLS.iter().all(|call| flags.contains(*call)) {
            for call in MODEL_CALLS {
                flags.remove(call);
            }
            flags.insert("all".to_owned());
        }
    }

    /// The files of the list for `words` whose flags S9.3 took out, with the words
    /// that replace each (design D5).
    fn taken_out_files(
        &self,
        words: &[String],
        inputs: &GenerateInputs<'_>,
        package: &mortise_core::Version,
    ) -> Vec<TakenOut> {
        let Some(list) = self.files_for(words) else {
            return Vec::new();
        };
        let Ok(report) = &list.listed else {
            return Vec::new();
        };
        let model = self
            .filled
            .arguments
            .get("model")
            .map_or("", |model| model.trim());
        report
            .files
            .iter()
            .filter(|file| file.action == FileAction::Keep)
            .filter(|file| {
                model_flag_of(model, &file.path).is_some_and(|flag| list.taken_out.contains(&flag))
            })
            .map(|file| TakenOut {
                path: file.path.clone(),
                replace: replace_for(inputs, &list.before, &file.path, package),
            })
            .collect()
    }

    /// Drops the file list: a run starts that writes files.
    pub fn drop_files(&mut self) {
        self.files = None;
    }

    /// The file list of the form whose words are `words`, if one is kept for them.
    fn files_for(&self, words: &[String]) -> Option<&FileList> {
        self.files.as_ref().filter(|list| list.words == words)
    }

    /// A result to show, read from the output of the run that just ended. The file
    /// list of the dry run before it is gone: the run wrote its files.
    pub fn show_result(&mut self, result: GenerateResult) {
        self.files = None;
        self.focus_result = !result.autoload_failed
            && !result.files.is_empty()
            && result
                .files
                .iter()
                .all(|file| matches!(file.outcome, FileOutcome::Created | FileOutcome::Replaced));
        self.result = Some(result);
    }

    /// Takes the end of a Replace of the left row `row` of the result on screen. A
    /// file of the run whose path is the row's gives the row its outcome. A file of
    /// another path, such as one the app's generator paths now place elsewhere, gets a
    /// row of its own, and `row` stays as it was. The other rows stay, with their
    /// Replace. A run that read no file leaves the rows and shows `fallback`, why it
    /// gave no answer. Returns false when the result holds no such row.
    pub fn replace_row(
        &mut self,
        row: &str,
        files: &[GeneratedFile],
        fallback: Option<String>,
    ) -> bool {
        let Some(result) = self.result.as_mut().filter(|result| {
            result
                .files
                .iter()
                .any(|file| file.path == row && file.outcome == FileOutcome::Left)
        }) else {
            return false;
        };
        let same = |a: &str, b: &str| a.replace('\\', "/") == b.replace('\\', "/");
        for done in files {
            match result
                .files
                .iter_mut()
                .find(|file| same(&file.path, &done.path))
            {
                Some(file) => file.outcome = done.outcome,
                None => result.files.push(done.clone()),
            }
        }
        if fallback.is_some() {
            result.fallback = fallback;
        }
        true
    }

    /// Esc on the picker clears its filter. Returns true when it did.
    pub fn escape(&mut self) -> bool {
        if self.view != View::Picker || self.filter.is_empty() {
            return false;
        }
        self.filter.clear();
        // egui has already taken focus from the field for this Esc: give it back.
        self.focus_filter = self.filter_was_focused;
        true
    }

    /// The filter field of the picker had focus at the end of the last frame.
    pub fn filter_was_focused(&self) -> bool {
        self.filter_was_focused
    }

    /// The names typed in the form, cleared after `module:make` created them.
    pub fn clear_name(&mut self) {
        self.filled.arguments.clear();
        self.result = None;
    }

    /// True while a form is on screen: the header shows the key that runs it.
    pub fn shows_form(&self) -> bool {
        self.view == View::Form
    }

    /// The generator and the module of the form on screen, or `None` on the picker.
    pub fn form(&self) -> Option<(String, Option<String>)> {
        let module = self.module.clone().flatten();
        (self.view == View::Form).then(|| (self.generator.clone(), module))
    }
}

/// What the tab needs besides the app and its own state.
pub struct GenerateInputs<'a> {
    /// The display name of the app.
    pub app: &'a str,
    /// The generators of the app: read from it, or shipped for its version.
    pub catalog: &'a Catalog,
    /// The read of the generators, on a 13.4 app only.
    pub catalog_read: Option<CatalogRead<'a>>,
    /// How PHP runs for the app. Without it nothing can run.
    pub runner: Option<&'a RunnerKind>,
    /// The working folder of the app's PHP settings, `None` for the app folder.
    pub folder: Option<&'a Path>,
    /// The label of the command that runs in the app, if one does.
    pub busy: Option<&'a str>,
    /// The dry run of a form runs (design D5).
    pub listing: bool,
    /// A dialog or the palette holds the keys.
    pub held: bool,
    pub keyboard: bool,
    pub mac: bool,
    pub reduced_motion: bool,
}

/// The title of the workspace on the Generate tab.
pub fn title(state: &GenerateState, module: Option<&Module>, app: &str) -> String {
    match module {
        Some(module) => strings::generate_in(&module.name),
        None if state.view == View::Form && state.generator == MODULE_MAKE => {
            strings::new_module_in(app)
        }
        // Not in the spec.
        None => strings::generate_in(app),
    }
}

/// The path of a file as the output prints it, to open: under the app folder when
/// it is relative.
pub fn file_path(root: &Path, printed: &str) -> PathBuf {
    let path = Path::new(printed);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

/// The part of a path after the last `/` or `\`.
fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// True when the generator takes a module argument: a form for it needs a module.
pub fn takes_module(generator: &Generator) -> bool {
    generator
        .arguments
        .iter()
        .any(|argument| argument.name == "module")
}

/// The names `module:make` words give: the words that are not options.
pub fn typed_names(args: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    let mut after_dashes = false;
    for word in args.iter().skip(1) {
        if word == "--" {
            after_dashes = true;
        } else if after_dashes || !word.starts_with('-') {
            names.push(word.clone());
        }
    }
    names
}

/// The folder names of the app's modules, which place a path of a container.
pub fn module_folders(app: &AppSnapshot) -> Vec<&str> {
    app.modules
        .iter()
        .filter_map(|module| module.dir.file_name()?.to_str())
        .collect()
}

/// The modules a run writes in, which place a path of a container first: those
/// that `module:make` asks for, else the folder of the run's own module `module`.
pub fn run_modules(
    app: Option<&AppSnapshot>,
    generator: &str,
    module: Option<&str>,
    words: &[String],
) -> Vec<String> {
    if generator == MODULE_MAKE {
        return typed_names(words);
    }
    app.zip(module)
        .and_then(|(app, name)| app.modules.iter().find(|known| known.name == name))
        .and_then(|module| module.dir.file_name()?.to_str())
        .map(str::to_owned)
        .into_iter()
        .collect()
}

/// The words that write `path` again, left by a run of `words`, as `replace_words`
/// gives them. The runner's own options, before a `--` in its line, go with every
/// command: a Custom line `php artisan --policy` gives the model its policy, and
/// the package then passes `--force` to the policy too (design D6).
fn replace_for(
    inputs: &GenerateInputs<'_>,
    words: &[String],
    path: &str,
    package: &mortise_core::Version,
) -> Option<Vec<String>> {
    let runner: Vec<String> = inputs
        .runner
        .map(RunnerKind::prefix)
        .unwrap_or_default()
        .into_iter()
        .skip(1)
        .take_while(|word| word != "--")
        .filter(|word| word.starts_with('-'))
        .collect();
    replace_words(inputs.catalog, words, &runner, path, package)
}

/// A path as the output prints it, relative to the folder of `module` when it is
/// inside it (S9.5). Mortise does not compute where a generator writes.
fn path_in_module(app: &AppSnapshot, module: Option<&Module>, printed: &str) -> String {
    let printed_slash = printed.replace('\\', "/");
    let Some(module) = module else {
        return printed_slash;
    };
    let path = Path::new(printed);
    if path.is_absolute() {
        return match path.strip_prefix(&module.dir) {
            Ok(rest) => rest.to_string_lossy().replace('\\', "/"),
            Err(_) => printed_slash,
        };
    }
    let folder = format!("{}/", app.path_shown(&module.dir));
    match printed_slash.strip_prefix(&folder) {
        Some(rest) => rest.to_owned(),
        None => printed_slash,
    }
}

/// Paints the Generate tab into `body`. Returns what was asked for.
pub fn generate(
    ui: &mut Ui,
    pal: &Palette,
    body: Rect,
    app: &AppSnapshot,
    module: Option<&Module>,
    state: &mut GenerateState,
    inputs: &GenerateInputs<'_>,
) -> Option<BoundaryAction> {
    let name = module.map(|module| module.name.as_str());
    state.follow(name, true);
    // A list from another runner or working folder lists the files of another app.
    let place = RunPlace {
        runner: inputs.runner.cloned(),
        folder: inputs.folder.map(Path::to_path_buf),
    };
    state.drop_files_of_another_place(&place);
    let on_form = state.view == View::Form && !inputs.held;
    let run_key = on_form
        && ui.input_mut(|input| {
            input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Enter))
        });
    let copy_key = on_form
        && ui.input_mut(|input| {
            input.consume_shortcut(&KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::SHIFT,
                Key::C,
            ))
        });

    let mut asked = None;
    let salt = name.unwrap_or("all");
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        ScrollArea::vertical()
            .id_salt(("generate", salt))
            .auto_shrink(false)
            .show(ui, |ui| {
                let origin = ui.cursor().min;
                let width = ui.available_width();
                let min = pos2(origin.x + 16.0, origin.y + 14.0);
                let inner = (width - 32.0).max(320.0);
                let (bottom, view_asked) = match state.view {
                    View::Picker => picker(ui, pal, min, inner, app, module, state, inputs),
                    View::Form => form(
                        ui, pal, min, inner, app, module, state, inputs, run_key, copy_key,
                    ),
                };
                asked = view_asked;
                ui.allocate_rect(
                    Rect::from_min_max(origin, pos2(origin.x + width, bottom + 16.0)),
                    Sense::hover(),
                );
            });
    });
    asked
}

// ---------------------------------------------------------------------------
// The picker (S9.1)

/// The groups of the picker that the filter leaves. Under "All modules" only New module.
fn listed<'a>(
    catalog: &'a Catalog,
    module: Option<&Module>,
    filter: &str,
) -> Vec<(Group, Vec<&'a Generator>)> {
    let query = filter.trim().to_lowercase();
    let mut groups = catalog.grouped();
    for (group, generators) in &mut groups {
        if module.is_none() && *group != Group::Module {
            generators.clear();
        }
        generators.retain(|generator| {
            query.is_empty()
                || strings::generator_label(&generator.name)
                    .to_lowercase()
                    .contains(&query)
                || generator.name.to_lowercase().contains(&query)
        });
    }
    groups.retain(|(_, generators)| !generators.is_empty());
    groups
}

/// The picker: the filter, the count line, then the groups in three columns.
/// Returns its bottom and what it asked for.
#[expect(clippy::too_many_arguments)]
fn picker(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    app: &AppSnapshot,
    module: Option<&Module>,
    state: &mut GenerateState,
    inputs: &GenerateInputs<'_>,
) -> (f32, Option<BoundaryAction>) {
    let catalog = inputs.catalog;
    let groups = listed(catalog, module, &state.filter);
    let items: Vec<&Generator> = groups
        .iter()
        .flat_map(|(_, generators)| generators.iter().copied())
        .collect();
    // The current row: the one chosen, else the first file generator listed.
    let mut current = state
        .current
        .as_deref()
        .and_then(|name| items.iter().position(|item| item.name == name))
        .or_else(|| items.iter().position(|item| item.name != MODULE_MAKE))
        .unwrap_or(0);

    // Where each row stands: its line of the grid and its column.
    let mut places = Vec::with_capacity(items.len());
    let mut line = 0;
    for (_, generators) in &groups {
        for (i, _) in generators.iter().enumerate() {
            places.push((line + i / 3, i % 3));
        }
        line += generators.len().div_ceil(3);
    }
    let step = |from: usize, key: Key| -> usize {
        let Some(&(at, col)) = places.get(from) else {
            return from;
        };
        let in_line = |target: usize| {
            places
                .iter()
                .enumerate()
                .filter(|(_, (line, _))| *line == target)
                .min_by_key(|(_, (_, c))| c.abs_diff(col))
                .map(|(i, _)| i)
        };
        match key {
            Key::ArrowLeft => from.saturating_sub(1),
            Key::ArrowRight => (from + 1).min(places.len().saturating_sub(1)),
            Key::ArrowUp => at.checked_sub(1).and_then(in_line).unwrap_or(from),
            _ => in_line(at + 1).unwrap_or(from),
        }
    };

    // The filter field (C3) holds the keys: Up and Down move, Enter opens the form.
    let filter_id = ui.scope_id().with("generate-filter");
    let mut open = None;
    if ui.memory(|memory| memory.has_focus(filter_id)) {
        let (up, down, enter) = ui.input_mut(|input| {
            (
                input.consume_key(Modifiers::NONE, Key::ArrowUp),
                input.consume_key(Modifiers::NONE, Key::ArrowDown),
                input.consume_key(Modifiers::NONE, Key::Enter),
            )
        });
        if up {
            current = step(current, Key::ArrowUp);
        }
        if down {
            current = step(current, Key::ArrowDown);
        }
        if enter && !items.is_empty() {
            open = Some(current);
        }
    }

    let field = Rect::from_min_size(min, vec2(FILTER_W.min(width), size::CONTROL_H));
    let before = state.filter.clone();
    let response = filter_field(
        ui,
        pal,
        field,
        filter_id,
        &mut state.filter,
        inputs.keyboard,
    );
    if std::mem::take(&mut state.focus_filter) {
        response.request_focus();
    }
    state.filter_was_focused = ui.memory(|memory| memory.has_focus(filter_id));
    if state.filter != before {
        // Another filter starts on its first match again.
        state.current = None;
    }

    // The count line at the right of the field, and the mark of another version.
    let mid = field.center().y;
    let line = match module {
        Some(module) => strings::generators_for(
            catalog
                .generators
                .iter()
                .filter(|generator| generator.name != MODULE_MAKE)
                .count(),
            &module.name,
        ),
        None => strings::SELECT_A_MODULE_TO_GENERATE.to_owned(),
    };
    draw::label(
        ui,
        pos2(min.x + width, mid),
        Align2::RIGHT_CENTER,
        &line,
        ty::SECONDARY,
        pal.ink_2,
    );
    let read = matches!(inputs.catalog_read, Some(CatalogRead::Read(_)));
    if let Some(version) = shipped_mark(&app.package).filter(|_| !read) {
        draw::label(
            ui,
            pos2(field.right() + 10.0, mid),
            Align2::LEFT_CENTER,
            &strings::for_package(version),
            ty::SECONDARY,
            pal.warn,
        );
    }

    // On a 13.4 app, the read of its generators under the field.
    let mut y = field.bottom() + 12.0;
    let mut read_asked = false;
    let mut read_shown = None;
    if let Some(read) = inputs.catalog_read {
        let shown = state.read_shown.as_deref() == Some(&read_catalog_words()[..]);
        let (bottom, pressed, whole) =
            catalog_read(ui, pal, pos2(min.x, y), width, read, inputs, shown);
        read_asked = pressed;
        read_shown = whole.then(read_catalog_words);
        y = bottom + 12.0;
    }
    state.read_shown = read_shown;

    // The groups, in three columns. A heading spans the three.
    let col_w = ((width - 2.0 * GAP) / 3.0).floor();
    let salt = ui.scope_id().with("generate-picker");
    let any_focused = (0..items.len()).any(|i| ui.memory(|memory| memory.has_focus(salt.with(i))));
    // The rows are one stop: the arrows move the current row while one has focus.
    if any_focused && !items.is_empty() {
        draw::keep_arrows(ui, salt.with(current), true, true);
        let moved = ui.input_mut(|input| {
            [
                Key::ArrowUp,
                Key::ArrowDown,
                Key::ArrowLeft,
                Key::ArrowRight,
            ]
            .into_iter()
            .find(|key| input.consume_key(Modifiers::NONE, *key))
        });
        if let Some(key) = moved {
            current = step(current, key);
            ui.memory_mut(|memory| memory.request_focus(salt.with(current)));
        }
    }
    let mut index = 0;
    for (group, generators) in &groups {
        let heading = strings::generator_group(*group);
        let text = draw::paint_text(
            ui,
            pos2(min.x, y + HEADING_H - 3.0),
            Align2::LEFT_BOTTOM,
            heading,
            ty::LABEL,
            pal.ink_2,
        );
        draw::announce(ui, text, Role::Heading, heading);
        draw::hline(ui, min.x..min.x + width, y + HEADING_H - 1.0, pal.line);
        y += HEADING_H;
        for (i, generator) in generators.iter().enumerate() {
            let x = min.x + (i % 3) as f32 * (col_w + GAP);
            let top = y + (i / 3) as f32 * (ITEM_H + 1.0);
            let rect = Rect::from_min_size(pos2(x, top), vec2(col_w, ITEM_H));
            if picker_row(
                ui,
                pal,
                rect,
                salt.with(index),
                generator,
                index == current,
                inputs.keyboard,
            ) {
                open = Some(index);
            }
            index += 1;
        }
        y += generators.len().div_ceil(3) as f32 * (ITEM_H + 1.0);
    }
    state.current = items.get(current).map(|item| item.name.clone());

    let asked = open.and_then(|at| items.get(at)).and_then(|generator| {
        if generator.name == MODULE_MAKE {
            return Some(BoundaryAction::Generate(GenerateAsk::NewModule));
        }
        state.open_form(&generator.name, takes_module(generator));
        ui.ctx().request_repaint();
        None
    });
    let asked = asked.or(read_asked.then_some(BoundaryAction::Generate(GenerateAsk::ReadCatalog)));
    (y, asked)
}

/// The read of the app's generators (design D4): a quiet button with what the last
/// read gave beside it, the line it runs under it, and why a read gave no list.
/// The button starts the read only when its line was whole on screen in the last
/// frame (`shown`). Returns its bottom, whether the button was pressed, and whether
/// the line is whole on screen now.
fn catalog_read(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    read: CatalogRead<'_>,
    inputs: &GenerateInputs<'_>,
    shown: bool,
) -> (f32, bool, bool) {
    let label = match read {
        CatalogRead::Read(_) => strings::READ_AGAIN.to_owned(),
        _ => strings::read_generators(inputs.app),
    };
    let off = read_off(inputs, shown);
    let button = Button {
        label: &label,
        kind: ButtonKind::Ghost,
        disabled: off.is_some(),
        ..Default::default()
    };
    let response = button.show(ui, pal, min, inputs.keyboard);
    let mid = response.rect.center().y;
    let right = response.rect.right();
    let pressed = response.clicked() && off.is_none();
    if let Some(tip) = &off {
        draw::tooltip(response, pal, tip);
    }
    let status = match read {
        CatalogRead::Reading => strings::READING.to_owned(),
        CatalogRead::Read(ms) => strings::read_from(inputs.app, &clock(ms)),
        CatalogRead::NotRead | CatalogRead::Failed(_) => String::new(),
    };
    if !status.is_empty() {
        draw::label(
            ui,
            pos2(right + 10.0, mid),
            Align2::LEFT_CENTER,
            &status,
            ty::SECONDARY,
            pal.ink_2,
        );
    }

    let words = read_catalog_words();
    let line = line_now(inputs.runner, &words)
        .unwrap_or_else(|| format!("php artisan {}", words.join(" ")));
    let code = draw::code_block(
        ui,
        pal,
        pos2(min.x, min.y + button.height() + 6.0),
        width,
        &line,
    );
    let whole = ui.clip_rect().contains_rect(code.shrink2(vec2(12.0, 10.0)));
    let CatalogRead::Failed(reason) = read else {
        return (code.bottom(), pressed, whole);
    };
    let text = strings::generators_not_read(inputs.app, reason);
    let bottom = draw::paragraph(
        ui,
        pos2(min.x, code.bottom() + 6.0),
        &text,
        HINT,
        pal.bad,
        width,
    )
    .bottom();
    (bottom, pressed, whole)
}

/// Why the button of a read is off: a run or read is running, the app has no PHP,
/// or its line was not whole on screen in the last frame (deviation S37's rule).
fn read_off(inputs: &GenerateInputs<'_>, shown: bool) -> Option<String> {
    if let Some(label) = inputs.busy {
        Some(strings::is_running(label))
    } else if line_now(inputs.runner, &[]).is_none() {
        Some(strings::choose_php(inputs.app))
    } else if !shown {
        Some(strings::SCROLL_TO_COMMAND.to_owned())
    } else {
        None
    }
}

/// A time in ms since 1970 as the local clock shows it: `14:05`.
fn clock(ms: i64) -> String {
    jiff::Timestamp::from_millisecond(ms)
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH)
        .to_zoned(jiff::tz::TimeZone::system())
        .strftime("%H:%M")
        .to_string()
}

/// The filter field of the picker (C3): a search icon, the text, and the key that
/// gets in, or Esc once inside.
fn filter_field(
    ui: &mut Ui,
    pal: &Palette,
    field: Rect,
    id: egui::Id,
    filter: &mut String,
    keyboard: bool,
) -> egui::Response {
    let focused = ui.memory(|memory| memory.has_focus(id));
    let outline = if focused {
        pal.ink
    } else if ui.rect_contains_pointer(field) {
        pal.ink_2
    } else {
        pal.line_2
    };
    draw::outlined(ui, field, radius::CONTROL, pal.surface, outline);
    let glass = Rect::from_min_size(
        pos2(field.left() + 8.0, field.center().y - 6.5),
        Vec2::splat(13.0),
    );
    let has_text = !filter.is_empty();
    let glass_color = if focused || has_text {
        pal.ink
    } else {
        pal.ink_3
    };
    draw::icon(ui, glass, Icon::Search, glass_color);
    // G gets into the field: / is the filter of the index (deviation S45).
    let keys = if focused || has_text { "Esc" } else { "G" };
    let cap = draw::keycap(ui, pal, pos2(field.right() - 8.0, field.center().y), keys);
    let text_rect = Rect::from_min_max(
        pos2(glass.right() + 7.0, field.top() + 1.0),
        pos2(cap.left() - 6.0, field.bottom() - 1.0),
    );
    let font = FontId::new(12.0, Face::Sans400.family());
    let hint = RichText::new(strings::FIND_A_GENERATOR)
        .color(pal.ink_3)
        .font(font.clone());
    let edit = egui::TextEdit::singleline(filter)
        .id(id)
        .frame(egui::Frame::NONE)
        .margin(egui::Margin::ZERO)
        .font(font)
        .text_color(pal.ink)
        .desired_width(text_rect.width())
        .align(Align2::LEFT_CENTER)
        .hint_text(hint);
    let response = ui.put(text_rect, edit);
    response.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, strings::FIND_A_GENERATOR));
    if keyboard && focused {
        draw::focus_ring(ui, pal, field, radius::CONTROL);
    }
    response
}

/// One row of the picker: the label, and the artisan name at the right in dim
/// mono. Returns true when it was clicked, or pressed with Enter or Space.
fn picker_row(
    ui: &Ui,
    pal: &Palette,
    rect: Rect,
    id: egui::Id,
    generator: &Generator,
    current: bool,
    keyboard: bool,
) -> bool {
    let label = strings::generator_label(&generator.name);
    let response = ui.interact(rect, id, draw::one_stop(current));
    response.widget_info(|| WidgetInfo::selected(Role::ListBoxOption, true, current, &label));
    if current {
        ui.painter().rect_filled(rect, 0, pal.well);
        draw::peg(ui, pal, rect);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, 0, pal.well);
    }
    draw::hline(ui, rect.left()..rect.right(), rect.bottom(), pal.line);
    let mid = rect.center().y;
    let text = draw::paint_text(
        ui,
        pos2(rect.left() + 14.0, mid),
        Align2::LEFT_CENTER,
        &label,
        ty::BODY.with_face(Face::Sans600),
        pal.ink,
    );
    let room = rect.right() - 8.0 - (text.right() + 8.0);
    if room > 20.0 {
        let artisan = Type {
            size: 11.5,
            ..ty::PATH
        };
        let galley = draw::galley_elided(ui, &generator.name, artisan, pal.ink_2, room);
        draw::paint_galley(
            ui,
            pos2(rect.right() - 8.0, mid),
            Align2::RIGHT_CENTER,
            galley,
        );
    }
    if keyboard && response.has_focus() {
        draw::focus_ring(ui, pal, rect, radius::STAMP);
    }
    response.clicked()
}

// ---------------------------------------------------------------------------
// The forms (S9.2, S9.4)

/// Why a name typed in a form cannot run, as the sentence under its field, or
/// `None` when it can or is empty.
fn name_error(generator: &Generator, argument: &str, text: &str) -> Option<String> {
    let refused = generator.refused_name(argument, text)?;
    Some(match generator.name_rule(argument) {
        _ if generator.name == MODULE_MAKE => strings::module_name_error(refused),
        NameRule::Free => strings::free_name_error(&generator.name),
        _ => strings::class_name_error(&generator.name, refused),
    })
}

/// True when every name the generator takes is filled and valid.
fn names_ok(generator: &Generator, filled: &Filled) -> bool {
    generator
        .arguments
        .iter()
        .filter(|argument| argument.name != "module" && argument.required)
        .all(|argument| {
            let text = filled
                .arguments
                .get(&argument.name)
                .map_or("", String::as_str);
            !text.trim().is_empty() && name_error(generator, &argument.name, text).is_none()
        })
}

/// A form: its fields on the left; on the right the line, its buttons, or the result.
#[expect(clippy::too_many_arguments)]
fn form(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    app: &AppSnapshot,
    module: Option<&Module>,
    state: &mut GenerateState,
    inputs: &GenerateInputs<'_>,
    run_key: bool,
    copy_key: bool,
) -> (f32, Option<BoundaryAction>) {
    let catalog = inputs.catalog;
    let Some(generator) = catalog.get(&state.generator) else {
        state.view = View::Picker;
        return (min.y, None);
    };
    let col_w = ((width - GAP) / 2.0).floor();
    let before = state.filled.clone();
    let mut asked = None;
    let focus_name = std::mem::take(&mut state.focus_name);
    let left_bottom = if generator.name == MODULE_MAKE {
        new_module_fields(ui, pal, min, col_w, generator, state, focus_name, inputs)
    } else {
        let (bottom, change) = generator_field(ui, pal, min, col_w, catalog, generator, inputs);
        if change {
            state.open_picker(false);
            ui.ctx().request_repaint();
        }
        let at = pos2(min.x, bottom + 12.0);
        if generator.name == MAKE_MODEL {
            // The flags whose files exist, while the list belongs to the form's words.
            let words = generator.words(&state.filled, module.map(|m| m.name.as_str()));
            let model = state.filled.arguments.get("model").map_or("", |m| m.trim());
            let exists = state
                .files_for(&words)
                .map(|list| list.existing(model))
                .unwrap_or_default();
            model_fields(
                ui, pal, at, col_w, generator, state, &exists, focus_name, inputs,
            )
        } else {
            generator_fields(ui, pal, at, col_w, generator, state, focus_name, inputs)
        }
    };
    // A change of the form brings the line back in place of the result.
    if state.filled != before {
        state.result = None;
    }

    let right = pos2(min.x + col_w + GAP, min.y);
    let shown_result = state
        .result
        .as_ref()
        .is_some_and(|result| !result.autoload_failed && !result.files.is_empty());
    let right_bottom = if shown_result {
        let (bottom, result_asked) = result_view(ui, pal, right, col_w, app, module, state, inputs);
        asked = result_asked;
        state.last_shown = None;
        state.read_shown = None;
        bottom
    } else {
        let (bottom, line_asked) = command(
            ui, pal, right, col_w, app, generator, module, state, inputs, run_key, copy_key,
        );
        asked = asked.or(line_asked);
        bottom
    };
    (left_bottom.max(right_bottom), asked)
}

/// "Generator" with the label, the artisan name and "Change" (S9.2). Returns its
/// bottom and whether "Change" was pressed.
fn generator_field(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    catalog: &Catalog,
    generator: &Generator,
    inputs: &GenerateInputs<'_>,
) -> (f32, bool) {
    let available = strings::available(catalog.generators.len());
    let top = field_label(ui, pal, min, strings::GENERATOR, &available);
    let rect = Rect::from_min_size(pos2(min.x, top), vec2(width, size::FIELD_H));
    draw::outlined(ui, rect, radius::CONTROL, pal.surface, pal.line_2);
    let mid = rect.center().y;
    let label = strings::generator_label(&generator.name);
    let parts = [
        (label.as_str(), ty::BODY.with_face(Face::Sans700), pal.ink),
        ("  ", ty::BODY, pal.ink),
        (generator.name.as_str(), ty::PATH, pal.ink_2),
    ];
    let galley = draw::rich_galley(ui, &parts, f32::INFINITY);
    let said = galley.text().to_owned();
    let text = draw::paint_galley(
        ui,
        pos2(rect.left() + 9.0, mid),
        Align2::LEFT_CENTER,
        galley,
    );
    draw::announce(ui, text, Role::Label, &said);
    let change = Button {
        label: strings::CHANGE,
        small: true,
        said: Some(strings::CHANGE_GENERATOR),
        ..Default::default()
    };
    let at = pos2(
        rect.right() - 3.0 - change.width(ui),
        mid - change.height() / 2.0,
    );
    let clicked = change.show(ui, pal, at, inputs.keyboard).clicked();
    (rect.bottom(), clicked)
}

/// The label of a field and its hint, 4 px above the field. Returns where the field starts.
fn field_label(ui: &Ui, pal: &Palette, min: Pos2, label: &str, hint: &str) -> f32 {
    let text = draw::label(ui, min, Align2::LEFT_TOP, label, ty::LABEL, pal.ink);
    if !hint.is_empty() {
        draw::label(
            ui,
            pos2(text.right() + 4.0, text.top()),
            Align2::LEFT_TOP,
            hint,
            ty::SECONDARY,
            pal.ink_2,
        );
    }
    text.top() + ty::LABEL.line + 4.0
}

/// A text field with its label, its hint and, under it, its error (C2).
struct Field<'a> {
    label: &'a str,
    hint: &'a str,
    /// The name a screen reader says, when it is not the label.
    said: Option<&'a str>,
    placeholder: &'a str,
    mono: bool,
    error: Option<String>,
    focus: bool,
}

impl Field<'_> {
    /// Paints the field with its top left at `min`. Returns its bottom.
    fn show(
        &self,
        ui: &mut Ui,
        pal: &Palette,
        min: Pos2,
        width: f32,
        text: &mut String,
        keyboard: bool,
    ) -> f32 {
        let top = if self.label.is_empty() {
            min.y
        } else {
            field_label(ui, pal, min, self.label, self.hint)
        };
        let rect = Rect::from_min_size(pos2(min.x, top), vec2(width, size::FIELD_H));
        let name = self.said.unwrap_or(self.label);
        let response = draw::text_field_with_hint(
            ui,
            pal,
            rect,
            name,
            text,
            self.placeholder,
            self.mono,
            keyboard,
        );
        if self.focus {
            response.request_focus();
        }
        let Some(error) = &self.error else {
            return rect.bottom();
        };
        // The outline is bad while the name fails; focus keeps its ring.
        draw::outlined(ui, rect, radius::CONTROL, Color32::TRANSPARENT, pal.bad);
        let y = rect.bottom() + 4.0;
        let mark = Rect::from_min_size(pos2(min.x, y + 2.0), Vec2::splat(size::STATE_MARK));
        draw::mark_bad(ui, mark, pal.bad, pal.surface);
        let text_x = mark.right() + 6.0;
        draw::paragraph(
            ui,
            pos2(text_x, y),
            error,
            HINT,
            pal.bad,
            min.x + width - text_x,
        )
        .bottom()
    }
}

/// A checkbox with its flag after it in dim mono, and "exists" before the flag when
/// the dry run found its file (S9.3). Returns its row, and switches `on` when it was clicked.
#[expect(clippy::too_many_arguments)]
fn flag_box(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    label: &str,
    flag: &str,
    on: bool,
    exists: bool,
    keyboard: bool,
    clicked: &mut bool,
) -> Rect {
    let response = draw::checkbox(ui, pal, min, label, on, false, keyboard);
    *clicked = response.clicked();
    let rect = response.rect;
    let mut right = rect.right();
    if exists {
        let mark = draw::paint_text(
            ui,
            pos2(right + 5.0, rect.center().y),
            Align2::LEFT_CENTER,
            strings::EXISTS,
            ty::SECONDARY,
            pal.warn,
        );
        draw::announce(ui, mark, Role::Label, &strings::flag_exists(label));
        right = mark.right();
    }
    if !flag.is_empty() {
        draw::label(
            ui,
            pos2(right + 5.0, rect.center().y),
            Align2::LEFT_CENTER,
            flag,
            ty::PATH,
            pal.ink_2,
        );
    }
    rect
}

/// Switches a flag of the form.
fn toggle(filled: &mut Filled, flag: &str) {
    if !filled.flags.remove(flag) {
        filled.flags.insert(flag.to_owned());
    }
}

/// The name field of a generator. Returns its bottom.
#[expect(clippy::too_many_arguments)]
fn name_field(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    generator: &Generator,
    argument: &str,
    state: &mut GenerateState,
    focus: bool,
    keyboard: bool,
) -> f32 {
    let label = strings::name_field(&generator.name);
    let text = state
        .filled
        .arguments
        .entry(argument.to_owned())
        .or_default();
    let field = Field {
        label: &label,
        hint: "",
        said: None,
        placeholder: "",
        mono: true,
        error: name_error(generator, argument, text),
        focus,
    };
    field.show(ui, pal, min, width, text, keyboard)
}

/// The text field of an option with a value, labelled with the package's description.
#[expect(clippy::too_many_arguments)]
fn value_field(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    label: &str,
    hint: &str,
    text: &mut String,
    keyboard: bool,
) -> f32 {
    let field = Field {
        label,
        hint,
        said: None,
        placeholder: "",
        mono: true,
        error: None,
        focus: false,
    };
    field.show(ui, pal, min, width, text, keyboard)
}

/// The description of an option or an argument as a label: without its full stop.
fn described(text: &str) -> &str {
    text.trim().trim_end_matches('.')
}

/// The fields of a generator the spec does not draw: its name, its options with a
/// value, then its flags under "Options", labelled from the package's descriptions
/// (deviation S46). `--force`, `--dry-run` and `--json` are not offered ([`NOT_OFFERED`]).
#[expect(clippy::too_many_arguments)]
fn generator_fields(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    generator: &Generator,
    state: &mut GenerateState,
    focus_name: bool,
    inputs: &GenerateInputs<'_>,
) -> f32 {
    let mut y = min.y;
    let mut first = true;
    for argument in generator.arguments.iter().filter(|a| a.name != "module") {
        y = name_field(
            ui,
            pal,
            pos2(min.x, y),
            width,
            generator,
            &argument.name,
            state,
            focus_name && first,
            inputs.keyboard,
        ) + 12.0;
        first = false;
    }
    for option in generator
        .options
        .iter()
        .filter(|option| option.accepts_value && !NOT_OFFERED.contains(&option.name.as_str()))
    {
        let flag = format!("--{}", option.name);
        let text = state.filled.values.entry(option.name.clone()).or_default();
        y = value_field(
            ui,
            pal,
            pos2(min.x, y),
            width,
            described(&option.description),
            &flag,
            text,
            inputs.keyboard,
        ) + 12.0;
    }
    let flags: Vec<_> = generator
        .options
        .iter()
        .filter(|option| !option.accepts_value && !NOT_OFFERED.contains(&option.name.as_str()))
        .collect();
    if !flags.is_empty() {
        let top = field_label(ui, pal, pos2(min.x, y), strings::OPTIONS, "");
        y = top;
        for option in flags {
            let on = state.filled.flags.contains(&option.name);
            let flag = format!("--{}", option.name);
            let mut clicked = false;
            let row = flag_box(
                ui,
                pal,
                pos2(min.x, y),
                described(&option.description),
                &flag,
                on,
                false,
                inputs.keyboard,
                &mut clicked,
            );
            if clicked {
                toggle(&mut state.filled, &option.name);
            }
            y = row.bottom();
        }
        y += 12.0;
    }
    y - 12.0
}

/// The form of `module:make-model` as S9.2 draws it. `taken` holds the flags whose
/// file exists, marked "exists" (S9.3).
#[expect(clippy::too_many_arguments)]
fn model_fields(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    generator: &Generator,
    state: &mut GenerateState,
    exists: &[&str],
    focus_name: bool,
    inputs: &GenerateInputs<'_>,
) -> f32 {
    let keyboard = inputs.keyboard;
    let mut y = name_field(
        ui, pal, min, width, generator, "model", state, focus_name, keyboard,
    ) + 12.0;
    let text = state
        .filled
        .values
        .entry("fillable".to_owned())
        .or_default();
    y = value_field(
        ui,
        pal,
        pos2(min.x, y),
        width,
        strings::FILLABLE_ATTRIBUTES,
        strings::FILLABLE_HINT,
        text,
        keyboard,
    ) + 12.0;

    // "Also create": two columns of the seven files, then "All of these".
    let top = field_label(ui, pal, pos2(min.x, y), strings::ALSO_CREATE, "");
    let half = ((width - 12.0) / 2.0).floor();
    let all_on = MODEL_CALLS
        .iter()
        .all(|flag| state.filled.flags.contains(*flag));
    let mut rows = top;
    for (i, flag) in MODEL_CALLS
        .iter()
        .chain(std::iter::once(&"all"))
        .enumerate()
    {
        let at = pos2(
            min.x + (i % 2) as f32 * (half + 12.0),
            top + (i / 2) as f32 * (size::HIT_MIN + 1.0),
        );
        let option = generator.options.iter().find(|option| option.name == *flag);
        let short = option
            .and_then(|option| option.shortcut.as_deref())
            .map(|short| format!("-{short}"))
            .unwrap_or_default();
        let (label, on) = if *flag == "all" {
            (strings::ALL_OF_THESE, all_on)
        } else {
            (
                strings::model_call(flag),
                state.filled.flags.contains(*flag),
            )
        };
        let mut clicked = false;
        let exists = exists.contains(flag);
        let row = flag_box(
            ui,
            pal,
            at,
            label,
            &short,
            on,
            exists,
            keyboard,
            &mut clicked,
        );
        rows = rows.max(row.bottom());
        if !clicked {
            continue;
        }
        // "All of these" checks the seven boxes; clearing one returns to the flags.
        if *flag == "all" {
            for call in MODEL_CALLS {
                if all_on {
                    state.filled.flags.remove(call);
                } else {
                    state.filled.flags.insert(call.to_owned());
                }
            }
        } else {
            toggle(&mut state.filled, flag);
        }
    }
    y = rows + 12.0;

    // The option apart from the files, under a hairline.
    draw::hline(ui, min.x..min.x + width, y, pal.line);
    y += 1.0 + 8.0;
    let on = state.filled.flags.contains("attributes");
    let mut clicked = false;
    let row = flag_box(
        ui,
        pal,
        pos2(min.x, y),
        strings::USE_ATTRIBUTES,
        "--attributes",
        on,
        false,
        keyboard,
        &mut clicked,
    );
    if clicked {
        toggle(&mut state.filled, "attributes");
    }
    row.bottom()
}

/// The new module form of `module:make` as S9.4 draws it.
#[expect(clippy::too_many_arguments)]
fn new_module_fields(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    generator: &Generator,
    state: &mut GenerateState,
    focus_name: bool,
    inputs: &GenerateInputs<'_>,
) -> f32 {
    let keyboard = inputs.keyboard;
    let text = state.filled.arguments.entry("name".to_owned()).or_default();
    let names = Field {
        label: strings::MODULE_NAMES,
        hint: strings::MODULE_NAMES_HINT,
        said: None,
        placeholder: "",
        mono: true,
        error: name_error(generator, "name", text),
        focus: focus_name,
    };
    let mut y = names.show(ui, pal, min, width, text, keyboard) + 12.0;

    // Kind: one of three, because the command resolves them to a single type.
    let top = field_label(ui, pal, pos2(min.x, y), strings::KIND, strings::KIND_HINT);
    let selected = if state.filled.flags.contains("api") {
        1
    } else if state.filled.flags.contains("plain") {
        2
    } else {
        0
    };
    let (track, chosen) = draw::segmented(
        ui,
        pal,
        pos2(min.x, top),
        strings::KIND,
        &strings::KINDS,
        selected,
        keyboard,
    );
    if let Some(chosen) = chosen {
        state.filled.flags.remove("api");
        state.filled.flags.remove("plain");
        match chosen {
            1 => state.filled.flags.insert("api".to_owned()),
            2 => state.filled.flags.insert("plain".to_owned()),
            _ => false,
        };
    }
    y = track.bottom() + 12.0;

    // Options: --inertia is a switch of its own; "on after creation" is --disabled unchecked.
    let mut top = field_label(ui, pal, pos2(min.x, y), strings::OPTIONS, "");
    let boxes = [
        (strings::INERTIA_MODULE, "--inertia", "inertia", false),
        (
            strings::TURN_ON_AFTER_CREATION,
            strings::ADDS_DISABLED,
            "disabled",
            true,
        ),
        (strings::REPLACE_MODULE_IF_EXISTS, "--force", "force", false),
    ];
    for (label, flag, option, inverted) in boxes {
        let on = state.filled.flags.contains(option) != inverted;
        let mut clicked = false;
        let row = flag_box(
            ui,
            pal,
            pos2(min.x, top),
            label,
            flag,
            on,
            false,
            keyboard,
            &mut clicked,
        );
        if clicked {
            toggle(&mut state.filled, option);
        }
        top = row.bottom();
    }
    y = top + 12.0;

    // Author, optional, under a hairline: three fields in a row.
    draw::hline(ui, min.x..min.x + width, y, pal.line);
    y += 1.0 + 8.0;
    let top = field_label(
        ui,
        pal,
        pos2(min.x, y),
        strings::AUTHOR,
        strings::AUTHOR_HINT,
    );
    let third = ((width - 16.0) / 3.0).floor();
    let mut bottom = top;
    for (i, (option, (placeholder, said))) in ["author-name", "author-email", "author-vendor"]
        .into_iter()
        .zip(strings::AUTHOR_FIELDS)
        .enumerate()
    {
        let text = state.filled.values.entry(option.to_owned()).or_default();
        let field = Field {
            label: "",
            hint: "",
            said: Some(said),
            placeholder,
            mono: false,
            error: None,
            focus: false,
        };
        let at = pos2(min.x + i as f32 * (third + 8.0), top);
        bottom = field.show(ui, pal, at, third, text, keyboard);
    }
    bottom
}

/// The keys that run the form.
fn run_keys(mac: bool) -> String {
    format!("{} Enter", strings::command_key(mac))
}

/// The words a run carries: on a 13.4 app with `--json`, once, so that its result
/// is read from its answer (design D6). An app's own generator runs without it: it
/// may give `--json` a meaning of its own.
fn run_words(app: &AppSnapshot, words: Vec<String>) -> Vec<String> {
    let generator = words.first().map_or("", String::as_str);
    if generator_gives_data(&app.package, generator) {
        with_json(words)
    } else {
        words
    }
}

/// "Command", the line the run button starts, the files it creates, and the
/// buttons. Returns its bottom and what was asked for.
#[expect(clippy::too_many_arguments)]
fn command(
    ui: &mut Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    app: &AppSnapshot,
    generator: &Generator,
    module: Option<&Module>,
    state: &mut GenerateState,
    inputs: &GenerateInputs<'_>,
    run_key: bool,
    copy_key: bool,
) -> (f32, Option<BoundaryAction>) {
    let name = module.map(|module| module.name.clone());
    // The words of the form, and the words that run: on a 13.4 app with `--json`.
    let mut words = generator.words(&state.filled, name.as_deref());
    // The file list belongs to the words it ran with: a change of the line drops it,
    // and gives back the flags it took out.
    if state.files.as_ref().is_some_and(|list| list.words != words) {
        state.drop_list();
        words = generator.words(&state.filled, name.as_deref());
    }
    let run_args = run_words(app, words.clone());
    let line = line_now(inputs.runner, &run_args)
        .unwrap_or_else(|| format!("php artisan {}", run_args.join(" ")));
    let label = draw::label(
        ui,
        min,
        Align2::LEFT_TOP,
        strings::COMMAND,
        ty::LABEL,
        pal.ink,
    );
    let code_at = pos2(min.x, label.top() + ty::LABEL.line + 4.0);
    let code = draw::code_block(ui, pal, code_at, width, &line);

    let new_module = generator.name == MODULE_MAKE;
    let ready = names_ok(generator, &state.filled);
    let mut y = code.bottom() + 8.0;
    let mut files = None;
    // A generator the package ships lists its files by a dry run (design D5). An
    // app's own may declare `--dry-run` and write anyway: its files come after the run.
    if generator_gives_data(&app.package, &generator.name) {
        let at = pos2(min.x, y + 4.0);
        let shown = files_this_creates(
            ui, pal, at, width, app, module, generator, &words, state, inputs, ready,
        );
        y = shown.bottom;
        files = Some(shown);
    } else {
        // Without the dry run (P2) the files are listed after the run (deviations S44, S55).
        y = draw::paragraph(
            ui,
            pos2(min.x, y),
            strings::FILES_AFTER_THE_RUN,
            HINT,
            pal.ink_2,
            width,
        )
        .bottom();
    }
    if new_module {
        y = draw::paragraph(
            ui,
            pos2(min.x, y + 2.0),
            strings::THEN_DUMP_AUTOLOAD,
            HINT,
            pal.ink_2,
            width,
        )
        .bottom();
    }
    if state
        .result
        .as_ref()
        .is_some_and(|result| result.autoload_failed)
    {
        let at = pos2(min.x, y + 12.0);
        y = alert_box(
            ui,
            pal,
            at,
            width,
            Severity::Problem,
            "",
            strings::AUTOLOAD_FAILED,
            None,
            true,
        )
        .0;
    }

    // The dry run's line, when it is whole on screen in this frame.
    state.read_shown = files.as_mut().and_then(|files| files.read_shown.take());
    // A run that gave no answer and shows no result, as one that read no file or whose
    // composer dump-autoload failed: why, under the line (design D2).
    if let Some(reason) = state
        .result
        .as_ref()
        .filter(|result| result.files.is_empty() || result.autoload_failed)
        .and_then(|result| result.fallback.as_deref())
    {
        let text = strings::read_from_output(reason);
        let at = pos2(min.x, y + 8.0);
        y = draw::paragraph(ui, at, &text, HINT, pal.ink_2, width).bottom();
    }

    let force = new_module && state.filled.flags.contains("force");
    let blocked = files.as_ref().is_some_and(|files| files.blocked);
    // The button and Ctrl Enter run only the line that was whole on screen in the
    // last frame (deviation S40's rule).
    let shown = state.last_shown.as_ref() == Some(&run_args);
    let off = if let Some(label) = inputs.busy {
        Some(strings::is_running(label))
    } else if line_now(inputs.runner, &[]).is_none() {
        Some(strings::choose_php(inputs.app))
    } else if ready && !shown {
        Some(strings::SCROLL_TO_COMMAND.to_owned())
    } else {
        None
    };
    let keys = run_keys(inputs.mac);
    let create = match files.as_ref().and_then(|files| files.create.clone()) {
        _ if new_module => strings::CREATE_MODULE.to_owned(),
        Some(create) => create,
        None => strings::CREATE_FILES.to_owned(),
    };
    let run = Button {
        label: &create,
        // Replace the module loses files: the red outline, and it asks first.
        kind: if force {
            ButtonKind::Danger
        } else {
            ButtonKind::Primary
        },
        keys: Some(&keys),
        disabled: off.is_some() || !ready || blocked,
        ..Default::default()
    };
    let y = y + 12.0;
    let response = run.show(ui, pal, pos2(min.x, y), inputs.keyboard);
    let mut x = response.rect.right() + 8.0;
    let pressed = response.clicked() || run_key;
    if let Some(tip) = &off {
        draw::tooltip(response, pal, tip);
    }
    let mut asked = None;
    if pressed && off.is_none() && ready && !blocked {
        asked = Some(BoundaryAction::Generate(GenerateAsk::Run(GenerateRun {
            generator: generator.name.clone(),
            module: name.clone(),
            args: run_args.clone(),
            taken_out: state.taken_out_files(&words, inputs, &app.package),
            row: None,
        })));
    }
    // A line cut off by the scroll area is not on screen, so Ctrl Enter does not
    // take it. Its text sits 12 px and 10 px inside the code block.
    let text = code.shrink2(vec2(12.0, 10.0));
    state.last_shown = ui.clip_rect().contains_rect(text).then_some(run_args);

    let copy = Button {
        label: strings::COPY_COMMAND,
        ..Default::default()
    };
    let response = copy.show(ui, pal, pos2(x, y), inputs.keyboard);
    x = response.rect.right() + 8.0;
    let mut bottom = response.rect.bottom();
    if response.clicked() || copy_key {
        asked = Some(BoundaryAction::Copy {
            text: line,
            said: strings::COPIED_COMMAND,
        });
    }

    let Some(files) = files else {
        return (bottom, asked);
    };
    if files.list {
        asked = Some(BoundaryAction::Generate(GenerateAsk::ListFiles {
            words: words.clone(),
        }));
    }
    // The generator's own file exists: Replace writes it again, after the dialog of S9.6.
    let replace = files
        .replace
        .as_deref()
        .and_then(|path| Some((path, replace_for(inputs, &words, path, &app.package)?)));
    if let Some((path, replace_args)) = replace {
        let file = file_name(path);
        let label = strings::replace_file(file);
        let button = Button {
            label: &label,
            kind: ButtonKind::Danger,
            disabled: off.is_some(),
            ..Default::default()
        };
        let response = button.show(ui, pal, pos2(x, y), inputs.keyboard);
        bottom = bottom.max(response.rect.bottom());
        if response.clicked() && off.is_none() {
            asked = Some(BoundaryAction::Generate(GenerateAsk::Replace {
                file: file.to_owned(),
                run: GenerateRun {
                    generator: replace_args.first().cloned().unwrap_or_default(),
                    module: name,
                    args: run_words(app, replace_args),
                    taken_out: Vec::new(),
                    row: None,
                },
            }));
        }
        if let Some(tip) = &off {
            draw::tooltip(response, pal, tip);
        }
    }
    (bottom, asked)
}

/// What "Files this creates" gives the buttons under it.
struct FilesShown {
    bottom: f32,
    /// The label of the primary button, when the list counts its files.
    create: Option<String>,
    /// A file of the generator's own exists: the primary button is off.
    blocked: bool,
    /// The path of that file as printed, for "Replace <file>".
    replace: Option<String>,
    /// The dry run's words when its line is whole on screen now.
    read_shown: Option<Vec<String>>,
    /// "List files" was pressed.
    list: bool,
}

/// What the run of a dry run's list writes.
#[derive(Debug, PartialEq, Eq)]
struct Writes<'a> {
    /// The generator's own file, which exists: Create is off.
    own: Option<&'a ReportedFile>,
    /// How many files the run writes.
    written: usize,
}

/// What the run of `generator` writes, from the files of its dry run in write order.
/// The first file is the generator's own. When it is kept, the run cannot write it,
/// and Create is off. A later kept file belongs to a generator it calls, such as the
/// view of `module:make-component` or a model's policy or factory: the run leaves it,
/// and it does not stop Create. `module:make` has no own file: the package leaves a
/// module that exists and makes the others. Only the files the run writes count.
fn writes<'a>(generator: &str, files: &[&'a ReportedFile]) -> Writes<'a> {
    let own = files
        .first()
        .copied()
        .filter(|file| generator != MODULE_MAKE && file.action == FileAction::Keep);
    let written = files
        .iter()
        .filter(|file| file.action != FileAction::Keep)
        .count();
    Writes { own, written }
}

/// The folder a printed path is shown under, and the rest of it: under the module
/// folder for a file generator (`Modules/Billing/`), under the folder of its module
/// for `module:make` (`Modules/Loyalty/`). A path outside both is shown as printed,
/// under no folder.
/// `asked` holds the modules the run writes in, which place a path first.
fn placed(
    app: &AppSnapshot,
    module: Option<&Module>,
    asked: &[&str],
    printed: &str,
) -> (String, String) {
    let outside = (String::new(), printed.to_owned());
    let modules_dir = app.path_shown(&app.layout.modules_dir);
    let folders = module_folders(app);
    let Some(relative) = app_relative(&app.root, &modules_dir, asked, &folders, printed) else {
        return outside;
    };
    if let Some(module) = module {
        let folder = format!("{}/", app.path_shown(&module.dir));
        return match relative.strip_prefix(&folder) {
            Some(rest) => (folder, rest.to_owned()),
            None => outside,
        };
    }
    let modules = format!("{}/", app.path_shown(&app.layout.modules_dir));
    match relative
        .strip_prefix(&modules)
        .and_then(|inner| inner.split_once('/'))
    {
        Some((name, rest)) => (format!("{modules}{name}/"), rest.to_owned()),
        None => (String::new(), relative),
    }
}

/// A file of the list under its folder.
struct ListRow<'a> {
    /// The folder under the root, with its last `/`, or empty.
    folder: String,
    name: String,
    file: &'a ReportedFile,
}

/// "Files this creates" on a 13.4 app (S9.2 to S9.4, design D5): before a dry run
/// "List files" with its line; after it the files by folder, their head, what
/// exists, and the errors of the package.
#[expect(clippy::too_many_arguments)]
fn files_this_creates(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    app: &AppSnapshot,
    module: Option<&Module>,
    generator: &Generator,
    words: &[String],
    state: &GenerateState,
    inputs: &GenerateInputs<'_>,
    ready: bool,
) -> FilesShown {
    let list = state.files_for(words);
    let Some((report, taken_out)) = list.and_then(|list| {
        let report = list.listed.as_ref().ok()?;
        Some((report, &list.taken_out))
    }) else {
        let shown = state.read_shown.as_deref() == Some(&with_dry_run(words.to_vec())[..]);
        return list_files(ui, pal, min, width, words, list, inputs, ready, shown);
    };
    let new_module = generator.name == MODULE_MAKE;
    let asked = run_modules(
        Some(app),
        &generator.name,
        module.map(|module| module.name.as_str()),
        words,
    );
    let asked: Vec<&str> = asked.iter().map(String::as_str).collect();
    // A `delete` entry is the folder that `module:make --force` deletes, not a file.
    let files: Vec<&ReportedFile> = report
        .files
        .iter()
        .filter(|file| file.action != FileAction::Delete)
        .collect();
    let model = state
        .filled
        .arguments
        .get("model")
        .map_or("", |model| model.trim());
    let flag_of = |file: &ReportedFile| {
        (generator.name == MAKE_MODEL && file.action == FileAction::Keep)
            .then(|| model_flag_of(model, &file.path))
            .flatten()
    };
    let taken = |file: &ReportedFile| flag_of(file).is_some_and(|flag| taken_out.contains(&flag));
    // The factory keeps its flag: the run leaves its file as it is (S9.3).
    let factory_kept = |file: &ReportedFile| flag_of(file) == Some(FACTORY);
    let Writes { own, written } = writes(&generator.name, &files);

    // The rows by root folder, in the order the roots come; in a root, the files at
    // its top first, then by folder.
    let mut roots: Vec<(String, Vec<ListRow>)> = Vec::new();
    for file in &files {
        let (root, rest) = placed(app, module, &asked, &file.path);
        let (folder, name) = match rest.rsplit_once('/') {
            Some((folder, name)) => (format!("{folder}/"), name.to_owned()),
            None => (String::new(), rest),
        };
        let row = ListRow { folder, name, file };
        match roots.iter_mut().find(|(known, _)| *known == root) {
            Some((_, rows)) => rows.push(row),
            None => roots.push((root, vec![row])),
        }
    }
    for (_, rows) in &mut roots {
        rows.sort_by(|a, b| a.folder.cmp(&b.folder));
    }

    let head = if new_module {
        let heads: Vec<String> = roots
            .iter()
            .filter(|(root, _)| !root.is_empty())
            .map(|(root, _)| {
                let folder = root.trim_end_matches('/');
                let count = app
                    .modules
                    .iter()
                    .find(|module| app.path_shown(&module.dir) == folder)
                    .map(|module| module.files);
                strings::module_folder_head(folder, count)
            })
            .collect();
        heads.join(", ")
    } else {
        let exist = files.iter().filter(|file| file.exists).count();
        strings::files_head(files.len(), exist)
    };
    let top = field_label(ui, pal, min, strings::FILES_THIS_CREATES, &head);
    let mut y = file_tree(ui, pal, pos2(min.x, top), width, &roots);

    // What exists, then the errors other than the one of a file that exists.
    let sentence = |y: f32, text: &str, color: Color32| {
        draw::paragraph(ui, pos2(min.x, y + 6.0), text, HINT, color, width).bottom()
    };
    let shown = |file: &ReportedFile| placed(app, module, &asked, &file.path).1;
    for file in files.iter().filter(|file| taken(file)) {
        let flag = flag_of(file).unwrap_or_default();
        let text = strings::taken_out(&shown(file), strings::model_call_kind(flag));
        y = sentence(y, &text, pal.ink_2);
    }
    for file in files.iter().filter(|file| factory_kept(file)) {
        y = sentence(y, &strings::factory_kept(&shown(file)), pal.ink_2);
    }
    if let Some(file) = own {
        let replaceable = replace_for(inputs, words, &file.path, &app.package).is_some();
        let text = strings::own_file_exists(&shown(file), replaceable);
        y = sentence(y, &text, pal.bad);
    }
    for error in &report.errors {
        let error = error.trim();
        if !(error.starts_with("File : ") && error.ends_with(" already exists.")) {
            y = sentence(y, &strings::quoted(error), pal.bad);
        }
    }

    let create = if own.is_some() || files.is_empty() {
        None
    } else if written < files.len() {
        Some(strings::create_other_files(written))
    } else {
        Some(strings::create_n_files(files.len()))
    };
    FilesShown {
        bottom: y,
        create,
        blocked: own.is_some(),
        replace: own.map(|file| file.path.clone()),
        read_shown: None,
        list: false,
    }
}

/// "List files" with "Reading" beside it while the dry run runs, the line it runs
/// under it, and why the last dry run of these words gave no list. The button
/// starts the dry run only when its line was whole on screen in the last frame
/// (`shown`). Returns its bottom and whether the button was pressed.
#[expect(clippy::too_many_arguments)]
fn list_files(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    words: &[String],
    list: Option<&FileList>,
    inputs: &GenerateInputs<'_>,
    ready: bool,
    shown: bool,
) -> FilesShown {
    let top = field_label(ui, pal, min, strings::FILES_THIS_CREATES, "");
    let off = read_off(inputs, shown || !ready);
    let button = Button {
        label: strings::LIST_FILES,
        disabled: off.is_some() || !ready,
        ..Default::default()
    };
    let response = button.show(ui, pal, pos2(min.x, top), inputs.keyboard);
    let mid = response.rect.center().y;
    let right = response.rect.right();
    let pressed = response.clicked() && off.is_none() && ready;
    if let Some(tip) = &off {
        draw::tooltip(response, pal, tip);
    }
    if inputs.listing {
        draw::label(
            ui,
            pos2(right + 10.0, mid),
            Align2::LEFT_CENTER,
            strings::READING,
            ty::SECONDARY,
            pal.ink_2,
        );
    }
    let dry_run = with_dry_run(words.to_vec());
    let line = line_now(inputs.runner, &dry_run)
        .unwrap_or_else(|| format!("php artisan {}", dry_run.join(" ")));
    let code = draw::code_block(
        ui,
        pal,
        pos2(min.x, top + button.height() + 6.0),
        width,
        &line,
    );
    let whole = ui.clip_rect().contains_rect(code.shrink2(vec2(12.0, 10.0)));
    let mut y = code.bottom() + 6.0;
    if let Some(Err(reason)) = list.map(|list| &list.listed) {
        let text = strings::dry_run_failed(reason);
        y = draw::paragraph(ui, pos2(min.x, y), &text, HINT, pal.bad, width).bottom() + 2.0;
    }
    let bottom = draw::paragraph(
        ui,
        pos2(min.x, y),
        strings::FILES_AFTER_THE_RUN,
        HINT,
        pal.ink_2,
        width,
    )
    .bottom();
    FilesShown {
        bottom,
        create: None,
        blocked: false,
        replace: None,
        read_shown: whole.then_some(dry_run),
        list: pressed,
    }
}

/// A line of the tree of a file list.
enum TreeLine<'a> {
    /// A root folder: `Modules/Billing/`.
    Root(&'a str),
    /// A file under its root, laid out.
    File(&'a ListRow<'a>, Arc<egui::Galley>),
}

/// The tree of a file list in a box: each root folder, then its files, the folder
/// in dim mono before the name, with "exists" at the right. Returns its bottom.
fn file_tree(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    roots: &[(String, Vec<ListRow>)],
) -> f32 {
    const ROW_H: f32 = 22.0;
    const PAD: f32 = 6.0;
    const ROOT_X: f32 = 12.0;
    const FILE_X: f32 = 26.0;
    let exists_w = draw::galley(ui, strings::EXISTS, ty::SECONDARY, pal.warn)
        .size()
        .x
        + 12.0;
    let mut lines = Vec::new();
    for (root, rows) in roots {
        let indent = if root.is_empty() {
            ROOT_X
        } else {
            lines.push((ROOT_X, TreeLine::Root(root)));
            FILE_X
        };
        for row in rows {
            let parts = [
                (row.folder.as_str(), PATH, pal.ink_2),
                (row.name.as_str(), PATH, pal.ink),
            ];
            let galley = draw::rich_galley(ui, &parts, width - indent - exists_w - 12.0);
            lines.push((indent, TreeLine::File(row, galley)));
        }
    }
    let height_of = |line: &TreeLine| match line {
        TreeLine::Root(_) => ROW_H,
        TreeLine::File(_, galley) => (galley.size().y + 7.0).max(ROW_H),
    };
    let height = PAD * 2.0 + lines.iter().map(|(_, line)| height_of(line)).sum::<f32>();
    let rect = Rect::from_min_size(min, vec2(width, height));
    draw::outlined(ui, rect, radius::CONTROL, pal.surface, pal.line_2);
    let mut y = rect.top() + PAD;
    for (indent, line) in &lines {
        let h = height_of(line);
        let mid = y + h / 2.0;
        let at = pos2(rect.left() + indent, mid);
        y += h;
        let (row, galley) = match line {
            TreeLine::Root(root) => {
                draw::label(ui, at, Align2::LEFT_CENTER, root, PATH, pal.ink_2);
                continue;
            }
            TreeLine::File(row, galley) => (row, galley),
        };
        let path = format!("{}{}", row.folder, row.name);
        let text = draw::paint_galley(ui, at, Align2::LEFT_CENTER, galley.clone());
        if !row.file.exists {
            draw::announce(ui, text, Role::Label, &path);
            continue;
        }
        draw::paint_text(
            ui,
            pos2(rect.right() - 12.0, mid),
            Align2::RIGHT_CENTER,
            strings::EXISTS,
            ty::SECONDARY,
            pal.warn,
        );
        let said = format!("{path}, {}", strings::EXISTS);
        draw::announce(ui, text, Role::Label, &said);
    }
    rect.bottom()
}

// ---------------------------------------------------------------------------
// The result (S9.5, S9.6)

/// The result of the last run in place of the line: the files, then what to do.
#[expect(clippy::too_many_arguments)]
fn result_view(
    ui: &Ui,
    pal: &Palette,
    min: Pos2,
    width: f32,
    app: &AppSnapshot,
    module: Option<&Module>,
    state: &mut GenerateState,
    inputs: &GenerateInputs<'_>,
) -> (f32, Option<BoundaryAction>) {
    let now = ui.input(|input| input.time);
    // Replace is off while a run or read holds the app, as the other run buttons are.
    let busy = inputs.busy.map(strings::is_running);
    let Some(shown) = state.result.as_mut().map(|result| &mut result.shown) else {
        return (min.y, None);
    };
    let age = now - *shown.get_or_insert(now);
    let Some(result) = state.result.clone() else {
        return (min.y, None);
    };
    let place = result
        .module
        .clone()
        .unwrap_or_else(|| inputs.app.to_owned());
    let total = result.files.len();
    let written = result.written().count();
    let left: Vec<&GeneratedFile> = result
        .files
        .iter()
        .filter(|file| file.outcome == FileOutcome::Left)
        .collect();
    let failed: Vec<&GeneratedFile> = result
        .files
        .iter()
        .filter(|file| file.outcome == FileOutcome::Failed)
        .collect();
    let all_done = written == total;
    let replaced = result
        .files
        .iter()
        .all(|file| file.outcome == FileOutcome::Replaced);

    // The heading of the box: what was done, and the state at the right.
    let title = match (all_done, replaced) {
        (true, true) => strings::replaced_in(&place),
        (true, false) => strings::created_in(&place),
        (false, _) => strings::created_of(written, total),
    };
    let note = if all_done {
        strings::files(total as u64)
    } else {
        strings::failed_count(total - written)
    };
    // A file the run left has its own Replace, not only the first (S9.6).
    let replaces: Vec<Option<Vec<String>>> = result
        .files
        .iter()
        .map(|file| {
            (file.outcome == FileOutcome::Left)
                .then(|| left_replace(&result, inputs, app, &file.path))
                .flatten()
        })
        .collect();
    let open = Button {
        label: strings::OPEN,
        small: true,
        ..Default::default()
    };
    let replace = Button {
        label: strings::REPLACE,
        small: true,
        ..Default::default()
    };
    // The path wraps before the state and the button at the right of its row.
    let right_w = |file: &GeneratedFile, replaceable: bool| {
        let word = state_word(file.outcome);
        let word_w = if word.is_empty() {
            0.0
        } else {
            draw::galley(ui, word, ty::SECONDARY, pal.ink_2).size().x
        };
        let ok = matches!(file.outcome, FileOutcome::Created | FileOutcome::Replaced);
        let button = if ok && openable(app, &file.path) {
            Some(open.width(ui))
        } else if replaceable {
            Some(replace.width(ui))
        } else {
            None
        };
        match button {
            Some(button) if word_w > 0.0 => button + 8.0 + word_w,
            Some(button) => button,
            None => word_w,
        }
    };
    let rows: Vec<(String, std::sync::Arc<egui::Galley>)> = result
        .files
        .iter()
        .zip(&replaces)
        .map(|(file, words)| {
            let right = (right_w(file, words.is_some()) + 12.0).max(70.0);
            let path_w = width - 14.0 - size::STATE_MARK - 8.0 - 10.0 - right;
            let shown = path_in_module(app, module, &file.path);
            let galley = draw::galley_wrapped(ui, &shown, PATH, pal.ink, path_w);
            (shown, galley)
        })
        .collect();
    let heights: Vec<f32> = rows
        .iter()
        .map(|(_, galley)| (galley.size().y + 10.0).max(RESULT_ROW_H))
        .collect();
    let head_h = 8.0 + 18.0 + 7.0;
    let height = 1.0 + head_h + 1.0 + heights.iter().map(|h| h + 1.0).sum::<f32>();
    let rect = Rect::from_min_size(min, vec2(width, height));
    draw::outlined(ui, rect, radius::CONTROL, Color32::TRANSPARENT, pal.line_2);
    let mid = rect.top() + 1.0 + 8.0 + 9.0;
    let title_rect = draw::paint_text(
        ui,
        pos2(rect.left() + 12.0, mid),
        Align2::LEFT_CENTER,
        &title,
        ty::BODY.with_face(Face::Sans700),
        pal.ink,
    );
    draw::announce(ui, title_rect, Role::Heading, &title);
    let color = if all_done { pal.good } else { pal.bad };
    let note_rect = draw::label(
        ui,
        pos2(rect.right() - 12.0, mid),
        Align2::RIGHT_CENTER,
        &note,
        ty::SECONDARY,
        color,
    );
    let mark = mark_rect(note_rect.left() - 5.0, mid);
    if all_done {
        draw::mark_good(ui, mark, pal.good);
    } else {
        draw::mark_bad(ui, mark, pal.bad, pal.surface);
    }
    let mut y = rect.top() + 1.0 + head_h;
    draw::hline(ui, rect.left() + 1.0..rect.right() - 1.0, y, pal.line);
    y += 1.0;

    // New rows carry the well fill, then it fades (S9.5).
    let fill = if inputs.reduced_motion || age >= NEW_FILL_S + NEW_FADE_S {
        0.0
    } else if age <= NEW_FILL_S {
        1.0
    } else {
        (1.0 - (age - NEW_FILL_S) / NEW_FADE_S) as f32
    };
    if fill > 0.0 {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(30));
    }
    let mut asked = None;
    for (index, (((file, (shown, galley)), row_h), words)) in result
        .files
        .iter()
        .zip(rows)
        .zip(&heights)
        .zip(replaces)
        .enumerate()
    {
        let row = Rect::from_min_size(pos2(rect.left() + 1.0, y), vec2(rect.width() - 2.0, *row_h));
        let ok = matches!(file.outcome, FileOutcome::Created | FileOutcome::Replaced);
        if ok && fill > 0.0 {
            ui.painter()
                .rect_filled(row, 0, pal.well.gamma_multiply(fill));
        }
        let mid = row.center().y;
        let mark = Rect::from_min_size(
            pos2(row.left() + 13.0, (mid - size::STATE_MARK / 2.0).round()),
            Vec2::splat(size::STATE_MARK),
        );
        if ok {
            draw::mark_good(ui, mark, pal.good);
        } else {
            draw::mark_bad(ui, mark, pal.bad, pal.surface);
        }
        let text = draw::paint_galley(
            ui,
            pos2(mark.right() + 8.0, mid),
            Align2::LEFT_CENTER,
            galley,
        );
        let right = row.right() - 9.0;
        let state_word = state_word(file.outcome);
        let said = if state_word.is_empty() {
            shown.clone()
        } else {
            format!("{shown}, {state_word}")
        };
        draw::announce(ui, text, Role::Label, &said);
        if ok && openable(app, &file.path) {
            let open_said = strings::open_path(&shown);
            let open = Button {
                label: strings::OPEN,
                small: true,
                said: Some(&open_said),
                ..Default::default()
            };
            let at = pos2(right - open.width(ui), mid - open.height() / 2.0);
            if open.show(ui, pal, at, inputs.keyboard).clicked() {
                asked = Some(open_file_asked(app, &file.path));
            }
            if !state_word.is_empty() {
                draw::paint_text(
                    ui,
                    pos2(at.x - 8.0, mid),
                    Align2::RIGHT_CENTER,
                    state_word,
                    ty::SECONDARY,
                    pal.ink_2,
                );
            }
        } else if ok {
            draw::paint_text(
                ui,
                pos2(right, mid),
                Align2::RIGHT_CENTER,
                state_word,
                ty::SECONDARY,
                pal.ink_2,
            );
        } else {
            let mut word_right = right;
            if let Some(words) = words {
                let replace_said = strings::replace_file(&shown);
                let replace = Button {
                    label: strings::REPLACE,
                    small: true,
                    said: Some(&replace_said),
                    disabled: busy.is_some(),
                    ..Default::default()
                };
                let at = pos2(right - replace.width(ui), mid - replace.height() / 2.0);
                let response = replace.show(ui, pal, at, inputs.keyboard);
                if response.clicked() && busy.is_none() {
                    asked = Some(replace_asked(app, &result, &file.path, words));
                }
                if let Some(tip) = &busy {
                    draw::tooltip(response, pal, tip);
                }
                word_right = at.x - 8.0;
            }
            draw::paint_text(
                ui,
                pos2(word_right, mid),
                Align2::RIGHT_CENTER,
                state_word,
                ty::SECONDARY,
                pal.bad,
            );
        }
        y = row.bottom();
        if index + 1 < total {
            draw::hline(ui, rect.left() + 1.0..rect.right() - 1.0, y, pal.line);
        }
        y += 1.0;
    }
    let mut y = rect.bottom();

    if !all_done {
        let left_names: Vec<&str> = left.iter().map(|file| file_name(&file.path)).collect();
        let failed_names: Vec<&str> = failed.iter().map(|file| file_name(&file.path)).collect();
        let text = strings::left_alert(written, total, &place, &left_names, &failed_names);
        let at = pos2(min.x, y + 12.0);
        y = alert_box(ui, pal, at, width, Severity::Problem, "", &text, None, true).0;
    }
    // Read from the lines: the run gave no answer, and the result says why (design D6).
    if let Some(reason) = &result.fallback {
        let text = strings::read_from_output(reason);
        y = draw::paragraph(ui, pos2(min.x, y + 8.0), &text, HINT, pal.ink_2, width).bottom();
    }

    // What to do next.
    let y = y + 12.0;
    let mut x = min.x;
    if all_done {
        if let Some(first) = result
            .written()
            .next()
            .filter(|first| openable(app, &first.path))
        {
            let name = file_name(&first.path);
            let label = strings::open_file(name);
            let open = Button {
                label: &label,
                kind: ButtonKind::Primary,
                keys: Some("Enter"),
                ..Default::default()
            };
            let response = open.show(ui, pal, pos2(x, y), inputs.keyboard);
            // The result takes focus once, so that Enter opens the first file (S9.5).
            if std::mem::take(&mut state.focus_result) {
                response.request_focus();
            }
            x = response.rect.right() + 8.0;
            if response.clicked() {
                asked = Some(open_file_asked(app, &first.path));
            }
        }
        let another = Button {
            label: strings::GENERATE_ANOTHER,
            ..Default::default()
        };
        let response = another.show(ui, pal, pos2(x, y), inputs.keyboard);
        if response.clicked() {
            // Back to the form: the name cleared, the rest kept.
            for argument in state.filled.arguments.values_mut() {
                argument.clear();
            }
            state.result = None;
            state.focus_name = true;
            ui.ctx().request_repaint();
        }
        return (response.rect.bottom(), asked);
    }
    let Some(first_left) = left.first() else {
        return (y - 12.0, asked);
    };
    let words = left_replace(&result, inputs, app, &first_left.path);
    let name = file_name(&first_left.path);
    let mut bottom = y - 12.0;
    if let Some(words) = words {
        let label = strings::replace_file(name);
        let replace = Button {
            label: &label,
            kind: ButtonKind::Danger,
            disabled: busy.is_some(),
            ..Default::default()
        };
        let response = replace.show(ui, pal, pos2(x, y), inputs.keyboard);
        x = response.rect.right() + 8.0;
        bottom = response.rect.bottom();
        if response.clicked() && busy.is_none() {
            asked = Some(replace_asked(app, &result, &first_left.path, words));
        }
        if let Some(tip) = &busy {
            draw::tooltip(response, pal, tip);
        }
    }
    if !openable(app, &first_left.path) {
        return (bottom, asked);
    }
    let existing = Button {
        label: strings::OPEN_THE_EXISTING_FILE,
        ..Default::default()
    };
    let response = existing.show(ui, pal, pos2(x, y), inputs.keyboard);
    if response.clicked() {
        asked = Some(open_file_asked(app, &first_left.path));
    }
    (response.rect.bottom(), asked)
}

/// The word at the right of a row of a result.
fn state_word(outcome: FileOutcome) -> &'static str {
    match outcome {
        FileOutcome::Created => "",
        FileOutcome::Replaced => strings::REPLACED,
        FileOutcome::Left => strings::EXISTS_LEFT,
        FileOutcome::Failed => strings::NOT_WRITTEN,
    }
}

/// The words that write a file the run left again. A file S9.3 took out has the
/// words of the form before its flag went.
fn left_replace(
    result: &GenerateResult,
    inputs: &GenerateInputs<'_>,
    app: &AppSnapshot,
    path: &str,
) -> Option<Vec<String>> {
    match result.taken_out.iter().find(|file| file.path == path) {
        Some(file) => file.replace.clone(),
        None => replace_for(inputs, &result.args, path, &app.package),
    }
}

/// Replace of a file the run left: the dialog of S9.6 asks first.
fn replace_asked(
    app: &AppSnapshot,
    result: &GenerateResult,
    path: &str,
    words: Vec<String>,
) -> BoundaryAction {
    BoundaryAction::Generate(GenerateAsk::Replace {
        file: file_name(path).to_owned(),
        run: GenerateRun {
            generator: words.first().cloned().unwrap_or_default(),
            module: result.module.clone(),
            args: run_words(app, words),
            taken_out: Vec::new(),
            row: Some(path.to_owned()),
        },
    })
}

/// True when a file of a result opens on this computer: a path under the app folder,
/// or another path that exists. PHP in a container prints its own folders, and a path
/// that `app_relative` could not place may not exist here.
fn openable(app: &AppSnapshot, printed: &str) -> bool {
    let placed = Path::new(printed).is_relative() && !printed.starts_with(['/', '\\']);
    placed || file_path(&app.root, printed).exists()
}

/// Opens a file of a result in the editor, through the code that opens a module.
fn open_file_asked(app: &AppSnapshot, printed: &str) -> BoundaryAction {
    BoundaryAction::OpenInEditor {
        path: file_path(&app.root, printed),
        line: 1,
    }
}

/// A state mark of 12 px whose right edge is at `right`.
fn mark_rect(right: f32, mid: f32) -> Rect {
    Rect::from_min_size(
        pos2(
            right - size::STATE_MARK,
            (mid - size::STATE_MARK / 2.0).round(),
        ),
        Vec2::splat(size::STATE_MARK),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_replace_updates_only_the_row_of_its_path() {
        let file = |path: &str, outcome| GeneratedFile {
            path: path.to_owned(),
            outcome,
        };
        let model = "Modules/Billing/app/Models/Invoice.php";
        let policy = "Modules/Billing/app/Policies/InvoicePolicy.php";
        let mut state = GenerateState {
            result: Some(GenerateResult::new(
                Some("Billing".to_owned()),
                Vec::new(),
                vec![
                    file(model, FileOutcome::Created),
                    file(policy, FileOutcome::Left),
                ],
                false,
            )),
            ..Default::default()
        };
        // The app's policy path changed since the result: the replacement wrote
        // another file, which gets its own row; the left row keeps its Replace.
        let moved = "Modules/Billing/app/Auth/InvoicePolicy.php";
        assert!(state.replace_row(policy, &[file(moved, FileOutcome::Replaced)], None));
        // A replacement that read no file keeps the rows and says why.
        assert!(state.replace_row(policy, &[], Some("no answer".to_owned())));
        let result = state.result.as_ref().unwrap();
        assert_eq!(
            result.files,
            [
                file(model, FileOutcome::Created),
                file(policy, FileOutcome::Left),
                file(moved, FileOutcome::Replaced),
            ]
        );
        assert_eq!(result.fallback.as_deref(), Some("no answer"));
        // A file of the row's own path updates the row.
        assert!(state.replace_row(policy, &[file(policy, FileOutcome::Replaced)], None));
        assert_eq!(
            state.result.unwrap().files[1],
            file(policy, FileOutcome::Replaced)
        );
    }

    #[test]
    fn only_the_generator_s_own_file_stops_create() {
        let file = |path: &str, action| ReportedFile {
            path: path.to_owned(),
            exists: action == FileAction::Keep,
            action,
            result: None,
        };
        // A component whose view exists: the class is written, the view is left.
        let class = file("app/View/Components/Badge.php", FileAction::Create);
        let view = file(
            "resources/views/components/badge.blade.php",
            FileAction::Keep,
        );
        assert_eq!(
            writes("module:make-component", &[&class, &view]),
            Writes {
                own: None,
                written: 1
            }
        );
        // A model that exists stops Create; its policy that exists does not.
        let model = file("app/Models/Invoice.php", FileAction::Keep);
        let policy = file("app/Policies/InvoicePolicy.php", FileAction::Keep);
        assert_eq!(
            writes(MAKE_MODEL, &[&model, &policy]),
            Writes {
                own: Some(&model),
                written: 0
            }
        );
        assert_eq!(writes(MODULE_MAKE, &[&model]).own, None);
    }
}
