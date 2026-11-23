//! Interface text of the Generate tab (section 10 of the specification).

/// The title of the new module form: `New module in shop-api`.
pub fn new_module_in(app: &str) -> String {
    format!("New module in {app}")
}
pub const FIND_A_GENERATOR: &str = "Find a generator";
/// Not in the spec: the line of the picker under "All modules", which lists New module only.
pub const SELECT_A_MODULE_TO_GENERATE: &str =
    "Select a module in the index to generate files in it.";
/// The mark of the picker when the app has another version than the shipped
/// catalog: `for nwidart/laravel-modules 13.3`.
pub fn for_package(version: &str) -> String {
    format!("for nwidart/laravel-modules {version}")
}

/// Not in the spec: the button of the picker that reads the generators of a 13.4
/// app (design D4): `Read the generators of shop-api`.
pub fn read_generators(app: &str) -> String {
    format!("Read the generators of {app}")
}

/// Not in the spec: the button that reads the generators again after a read.
pub const READ_AGAIN: &str = "Read again";

/// Not in the spec: the place of the data while a read of it runs (design D3).
pub const READING: &str = "Reading";

/// Not in the spec: the picker after a read of the generators, `Read from shop-api at 14:05.`
pub fn read_from(app: &str, clock: &str) -> String {
    format!("Read from {app} at {clock}.")
}

/// Not in the spec: the picker after a read of the generators that gave none (design D2).
pub fn generators_not_read(app: &str, reason: &str) -> String {
    format!(
        "Mortise could not read the generators of {app}: \"{reason}\". The list is the one \
         shipped for 13.4."
    )
}

/// `41 generators for Billing, and New module. Enter opens the form.`
pub fn generators_for(n: usize, module: &str) -> String {
    match n {
        1 => format!("1 generator for {module}, and New module. Enter opens the form."),
        n => format!("{n} generators for {module}, and New module. Enter opens the form."),
    }
}

/// The groups of the picker (S9.1). "Other" holds a generator Mortise does not place.
pub fn generator_group(group: mortise_core::Group) -> &'static str {
    use mortise_core::Group;
    match group {
        Group::Module => "In the app",
        Group::Data => "Data",
        Group::Http => "HTTP",
        Group::Events => "Events and queues",
        Group::Views => "Views",
        Group::Classes => "Classes",
        Group::Console => "Console and setup",
        Group::Tests => "Tests",
        Group::Other => "Other",
    }
}

pub const GENERATOR: &str = "Generator";
/// `42 available`, beside "Generator".
pub fn available(n: usize) -> String {
    format!("{n} available")
}
pub const CHANGE: &str = "Change";
/// Not in the spec: the name a screen reader says for "Change".
pub const CHANGE_GENERATOR: &str = "Change generator";

/// The label of the name field of a generator: `Model name`. The labels of the
/// generators the spec does not draw are not in the spec.
pub fn name_field(name: &str) -> String {
    format!("{} name", crate::strings::generator_label(name))
}

pub const MODULE_NAMES: &str = "Module names";
pub const MODULE_NAMES_HINT: &str = "one or more, separated by spaces";
pub const FILLABLE_ATTRIBUTES: &str = "Fillable attributes";
pub const FILLABLE_HINT: &str = "--fillable, comma separated";
pub const ALSO_CREATE: &str = "Also create";
pub const ALL_OF_THESE: &str = "All of these";
pub const USE_ATTRIBUTES: &str = "Use Eloquent attributes such as #[Fillable]";
pub const KIND: &str = "Kind";
pub const KIND_HINT: &str = "--web is the default";
pub const KINDS: [&str; 3] = ["Web", "API", "Plain"];
pub const OPTIONS: &str = "Options";
pub const INERTIA_MODULE: &str = "Inertia controller and pages, no Blade views";
pub const TURN_ON_AFTER_CREATION: &str = "Turn the module on after creation";
pub const ADDS_DISABLED: &str = "unchecked adds --disabled";
pub const REPLACE_MODULE_IF_EXISTS: &str = "Replace the module if it exists";
pub const AUTHOR: &str = "Author";
pub const AUTHOR_HINT: &str = "optional, written to composer.json";
/// The placeholders of the author fields, and, not in the spec, their names.
pub const AUTHOR_FIELDS: [(&str, &str); 3] = [
    ("Name", "Author name"),
    ("Email", "Author email"),
    ("Vendor", "Author vendor"),
];

/// The label of a file of `module:make-model` under "Also create", by option name.
pub fn model_call(option: &str) -> &'static str {
    match option {
        "controller" => "Controller",
        "factory" => "Factory",
        "migration" => "Migration",
        "request" => "Request",
        "resource" => "Resource",
        "policy" => "Policy",
        _ => "Seeder",
    }
}

/// Not in the spec: a form says when the files are listed, until the package gives
/// a dry run (P2, deviation S44).
pub const FILES_AFTER_THE_RUN: &str = "The files are listed after the run.";
pub const THEN_DUMP_AUTOLOAD: &str = "Then the command runs composer dump-autoload in the app.";
/// Not in the spec: the button of a file generator, without a count (deviation S44).
pub const CREATE_FILES: &str = "Create files";
pub const CREATE_MODULE: &str = "Create module";

/// The heading of the file list of a 13.4 app (S9.2).
pub const FILES_THIS_CREATES: &str = "Files this creates";
/// Not in the spec: the button that starts the dry run of the form (design D5).
pub const LIST_FILES: &str = "List files";
/// Not in the spec: the button beside the head of a file list that drops the list, so
/// that "List files" and the line of its dry run come back for a fresh answer.
pub const LIST_FILES_AGAIN: &str = "List files again";
/// A file of the list that exists (S9.3).
pub const EXISTS: &str = "exists";

/// The head of a file list: `3 files, none exist yet`, `3 files, 1 exists already`
/// (S9.2, S9.3). More than one file that exists, and a list without files, are not
/// in the spec.
pub fn files_head(n: usize, exist: usize) -> String {
    match (n, exist) {
        (0, _) => "No files".to_owned(),
        (n, 0) => format!("{}, none exist yet", crate::strings::files(n as u64)),
        (n, 1) => format!("{}, 1 exists already", crate::strings::files(n as u64)),
        (n, exist) => format!("{}, {exist} exist already", crate::strings::files(n as u64)),
    }
}

/// The head of the file list of `module:make` (S9.4): `Modules/Loyalty does not exist
/// yet`, or, with the module in the snapshot, `Modules/Loyalty exists with 23 files`.
pub fn module_folder_head(folder: &str, files_count: Option<u64>) -> String {
    match files_count {
        Some(n) => format!("{folder} exists with {}", crate::strings::files(n)),
        None => format!("{folder} does not exist yet"),
    }
}

/// The button of a file generator with a file list: `Create 3 files` (S9.2).
pub fn create_n_files(n: usize) -> String {
    format!("Create {}", crate::strings::files(n as u64))
}

/// The button after a flag was taken out of the command: `Create the other 2 files`
/// (S9.3). One file is not in the spec.
pub fn create_other_files(n: usize) -> String {
    match n {
        1 => "Create the other file".to_owned(),
        n => format!("Create the other {n} files"),
    }
}

/// Not in the spec: the name a screen reader says for the mark of a box whose file
/// exists, `Policy exists`.
pub fn flag_exists(label: &str) -> String {
    format!("{label} {EXISTS}")
}

/// What a flag of `module:make-model` creates, in a sentence: `policy`.
pub fn model_call_kind(flag: &str) -> &'static str {
    match flag {
        "controller" => "controller",
        "factory" => "factory",
        "request" => "request",
        "resource" => "resource",
        "seed" => "seeder",
        _ => "policy",
    }
}

/// A flag taken out of the command because its file exists (S9.3).
pub fn taken_out(path: &str, kind: &str) -> String {
    format!(
        "{path} exists. A generator does not write over a file, so the {kind} was taken out \
         of the command. Create it afterwards with \"Replace\" if you want the generated one."
    )
}

/// Not in the spec: the factory of a model exists. Its flag stays, as the model's
/// stub reads it (S9.3).
pub fn factory_kept(path: &str) -> String {
    format!(
        "{path} exists. The model uses its factory, so the factory stays in the command. The \
         run leaves the file as it is."
    )
}

/// Not in the spec: the generator's own file exists (design D5). `replaceable`: the
/// form offers Replace for it.
pub fn own_file_exists(path: &str, replaceable: bool) -> String {
    if replaceable {
        format!(
            "{path} exists. The generator does not write over a file. Choose another name, \
             or replace the file."
        )
    } else {
        format!("{path} exists. The generator does not write over a file. Choose another name.")
    }
}

/// Not in the spec: an error of the dry run, in quotes, under the list (design D5).
pub fn quoted(text: &str) -> String {
    format!("\"{}\"", text.trim())
}

/// Not in the spec: a dry run that gave no document (design D2).
pub fn dry_run_failed(reason: &str) -> String {
    format!("The dry run gave no file list: \"{reason}\".")
}

/// Not in the spec: a result read from the output lines because the run gave no
/// document (design D2).
pub fn read_from_output(reason: &str) -> String {
    format!("Read from the output: the package's answer was \"{reason}\".")
}

/// The error of a class name: `A model name starts with a letter and has no spaces.
/// Write PaymentIntent, not "payment intent".` The example is the typed name as a
/// class name, when it makes one. Only the model sentence is in the spec.
pub fn class_name_error(generator: &str, typed: &str) -> String {
    let first = format!(
        "{} name starts with a letter and has no spaces.",
        crate::strings::capitalised(&crate::strings::with_article(
            &crate::strings::generator_noun(generator)
        ))
    );
    let studly: String = typed
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '/' && c != '\\')
        .filter(|word| !word.is_empty())
        .map(crate::strings::capitalised)
        .collect();
    if studly != typed && mortise_core::class_name_ok(&studly) {
        format!("{first} Write {studly}, not \"{typed}\".")
    } else {
        first
    }
}

/// Not in the spec: the error of a name that is not a class name, such as a view.
pub fn free_name_error(generator: &str) -> String {
    format!(
        "{} name has no spaces and does not start with \"-\".",
        crate::strings::capitalised(&crate::strings::with_article(
            &crate::strings::generator_noun(generator)
        ))
    )
}

/// The package's own sentence for a module name it refuses.
pub fn module_name_error(name: &str) -> String {
    format!(
        "The module name [{name}] is not valid. Use letters, digits, \"-\", \"_\" or spaces, \
         starting with a letter."
    )
}

pub const AUTOLOAD_FAILED: &str = "The module was created, but composer dump-autoload failed. \
                                   Fix the module (for example its composer.json), then run \
                                   composer dump-autoload.";

/// The heading of a result that wrote every file: `Created in Billing`.
pub fn created_in(module: &str) -> String {
    format!("Created in {module}")
}

/// Not in the spec: the heading of a result that wrote every file again with `--force`.
pub fn replaced_in(module: &str) -> String {
    format!("Replaced in {module}")
}

/// `Created 2 of 3 files`
pub fn created_of(done: usize, total: usize) -> String {
    format!("Created {done} of {}", crate::strings::files(total as u64))
}

/// `1 failed`
pub fn failed_count(n: usize) -> String {
    format!("{n} failed")
}

pub const EXISTS_LEFT: &str = "exists, left as it is";
/// Not in the spec: a file a run wrote again with `--force`.
pub const REPLACED: &str = "replaced";
/// Not in the spec: a file the output names as not written, for another reason
/// than that it exists.
pub const NOT_WRITTEN: &str = "not written";
pub const OPEN: &str = "Open";
/// Not in the spec: the name a screen reader says for "Open" of a row.
pub fn open_path(path: &str) -> String {
    format!("Open {path}")
}
/// `Open PaymentIntent.php`
pub fn open_file(name: &str) -> String {
    format!("Open {name}")
}
pub const GENERATE_ANOTHER: &str = "Generate another";
/// `Replace PaymentIntentPolicy.php`
pub fn replace_file(name: &str) -> String {
    format!("Replace {name}")
}
pub const OPEN_THE_EXISTING_FILE: &str = "Open the existing file";

/// The alert of a run that left files (S9.6). More than one file left, and a file
/// not written for another reason, are not in the spec.
pub fn left_alert(
    done: usize,
    total: usize,
    module: &str,
    left: &[&str],
    failed: &[&str],
) -> String {
    let mut text = format!(
        "Created {done} of {} in {module}.",
        crate::strings::files(total as u64)
    );
    match left {
        [] => {}
        [one] => text.push_str(&format!(" {one} already exists and was left as it is.")),
        many => text.push_str(&format!(
            " {} already exist and were left as they are.",
            crate::strings::and_list(many)
        )),
    }
    match failed {
        [] => {}
        [one] => text.push_str(&format!(
            " {one} was not written. The output is in the runner."
        )),
        many => text.push_str(&format!(
            " {} were not written. The output is in the runner.",
            crate::strings::and_list(many)
        )),
    }
    text
}

/// The done toast of a generator run: `Created 3 files in Billing.` A run that wrote
/// files again (not in the spec) says "Replaced".
pub fn created_files(n: usize, module: &str, replaced: bool) -> String {
    let verb = if replaced { "Replaced" } else { "Created" };
    format!("{verb} {} in {module}.", crate::strings::files(n as u64))
}

/// `Replace PaymentIntentPolicy.php?`
pub fn replace_file_title(name: &str) -> String {
    format!("Replace {name}?")
}
pub const REPLACE_FILE_BODY: &str = "The file exists. The generator writes a new one over it, \
                                     and Mortise cannot bring the old one back.";
pub const REPLACE_FILE: &str = "Replace file";

/// `Replace the module Loyalty?` More than one name is not in the spec.
pub fn replace_module_title(names: &[&str]) -> String {
    match names {
        [one] => format!("Replace the module {one}?"),
        many => format!("Replace the modules {}?", crate::strings::and_list(many)),
    }
}

/// `Modules/Loyalty exists with 23 files. module:make --force deletes the folder,
/// then generates the module again. Files you wrote in it are lost.` The package
/// deletes the folder first, so the spec's sentence is not used (deviation S50).
/// The sentence for a module Mortise does not see is not in the spec.
pub fn replace_module_body(existing: Option<(&str, u64)>) -> String {
    match existing {
        Some((folder, n)) => format!(
            "{folder} exists with {}. module:make --force deletes the folder, then generates \
             the module again. Files you wrote in it are lost.",
            crate::strings::files(n)
        ),
        None => "If the module exists, module:make --force deletes its folder, then generates \
                 the module again. Files you wrote in it are lost."
            .to_owned(),
    }
}
pub const REPLACE_MODULE: &str = "Replace module";

/// Not in the spec: the done toast of `module:make`.
pub fn created_modules(names: &[&str]) -> String {
    match names {
        [one] => format!("Created the module {one}."),
        many => format!("Created the modules {}.", crate::strings::and_list(many)),
    }
}
