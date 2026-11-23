//! The generators of the package, the words a form gives, and the name checks.
//!
//! A [`Catalog`] has the shape of the package's generator description (P3).
//! Mortise ships the ones of 13.3.0 and 13.4.0 ([`Catalog::shipped`]). [`Generator::words`]
//! turns a filled form into the words of a command. Nothing here starts a process
//! or builds a file path.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::modules::composer::Version;
use crate::package::gives_data;

/// The artisan name of the generator of a new module.
pub const MODULE_MAKE: &str = "module:make";

const MAKE_MODEL: &str = "module:make-model";

/// The flags that `--all` of `module:make-model` turns on.
const MODEL_FLAGS: [&str; 7] = [
    "controller",
    "factory",
    "migration",
    "request",
    "resource",
    "policy",
    "seed",
];

/// The options Mortise adds itself from 13.4 on: `--dry-run` for the file list and
/// `--json` for the answer (design D4). A form never sets them.
const ADDED: [&str; 2] = ["dry-run", "json"];

/// The generators whose name is not a class name. The package takes a name that
/// the class rule refuses:
///
/// - `module:make-view` and `module:make-component-view` write
///   `<name in lower case>.blade.php`, so a view is often `user-card` or `posts.index`.
/// - `module:make-config` keeps the name as given, such as `stripe-keys` or `services.php`.
/// - `module:make-inertia-page` and `module:make-inertia-component` take a script path
///   and make its last part studly, so `user-profile` writes `UserProfile.vue`.
///
/// Every other generator writes a class, also `module:make-command` and
/// `module:make-helper`. A migration name such as `create_posts_table` passes the class rule.
const FREE: [&str; 5] = [
    "module:make-component-view",
    "module:make-config",
    "module:make-inertia-component",
    "module:make-inertia-page",
    "module:make-view",
];

/// The groups of the picker, in the order of S9.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    /// "In the app": the new module.
    Module,
    Data,
    Http,
    Events,
    Views,
    Classes,
    Console,
    Tests,
    /// A generator that the order of S9.1 does not name.
    Other,
}

/// The generators of each group, in the order S9.1 draws them.
/// `module:make-health-indicator` is on the package's `master` only.
const ORDER: [(Group, &[&str]); 8] = [
    (Group::Module, &[MODULE_MAKE]),
    (
        Group::Data,
        &[
            MAKE_MODEL,
            "module:make-migration",
            "module:make-factory",
            "module:make-seed",
            "module:make-observer",
            "module:make-scope",
            "module:make-cast",
            "module:make-policy",
            "module:make-repository",
        ],
    ),
    (
        Group::Http,
        &[
            "module:make-controller",
            "module:make-request",
            "module:make-resource",
            "module:make-middleware",
            "module:make-rule",
            "module:route-provider",
        ],
    ),
    (
        Group::Events,
        &[
            "module:make-event",
            "module:make-listener",
            "module:make-job",
            "module:make-job-middleware",
            "module:make-notification",
            "module:make-mail",
            "module:make-channel",
            "module:make-event-provider",
        ],
    ),
    (
        Group::Views,
        &[
            "module:make-view",
            "module:make-component",
            "module:make-component-view",
            "module:make-inertia-page",
            "module:make-inertia-component",
        ],
    ),
    (
        Group::Classes,
        &[
            "module:make-class",
            "module:make-health-indicator",
            "module:make-action",
            "module:make-service",
            "module:make-interface",
            "module:make-trait",
            "module:make-enum",
            "module:make-helper",
            "module:make-exception",
        ],
    ),
    (
        Group::Console,
        &[
            "module:make-command",
            "module:make-config",
            "module:make-provider",
            "module:make-replacement",
        ],
    ),
    (Group::Tests, &["module:make-test"]),
];

/// The generators of the package, as P3 describes them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    pub generators: Vec<Generator>,
}

/// One generator, such as `module:make-model`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Generator {
    /// The artisan name.
    pub name: String,
    /// The group P3 gives. The picker uses [`Catalog::grouped`] instead.
    pub group: String,
    pub description: String,
    pub arguments: Vec<Argument>,
    pub options: Vec<GeneratorOption>,
}

/// An argument of a generator, in the order artisan reads it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Argument {
    pub name: String,
    pub required: bool,
    /// The argument takes one or more words.
    pub array: bool,
    /// What Symfony gives: `null`, `false`, a string or an array.
    pub default: serde_json::Value,
    pub description: String,
}

/// An option of a generator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratorOption {
    pub name: String,
    pub shortcut: Option<String>,
    /// False for a flag.
    pub accepts_value: bool,
    pub value_required: bool,
    pub array: bool,
    pub negatable: bool,
    /// What Symfony gives: `null`, `false`, a string or an array.
    pub default: serde_json::Value,
    pub description: String,
}

impl Catalog {
    /// Reads the P3 document.
    pub fn from_json(text: &str) -> Result<Catalog, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// The catalog Mortise ships for an app before 13.4: the 42 generators of 13.3.0.
    pub fn v13_3() -> &'static Catalog {
        static CATALOG: OnceLock<Catalog> = OnceLock::new();
        CATALOG.get_or_init(|| {
            Catalog::from_json(include_str!("v13_3.json")).expect("the shipped catalog is valid")
        })
    }

    /// The catalog Mortise ships for a 13.4 app: the 43 generators that
    /// `module:generators --json` gave on a fresh 13.4.0 app.
    pub fn v13_4() -> &'static Catalog {
        static CATALOG: OnceLock<Catalog> = OnceLock::new();
        CATALOG.get_or_init(|| {
            Catalog::from_json(include_str!("v13_4.json")).expect("the shipped catalog is valid")
        })
    }

    /// The catalog Mortise ships for this package version: [`Catalog::v13_4`] when
    /// the package gives data ([`gives_data`]), else [`Catalog::v13_3`].
    pub fn shipped(version: &Version) -> &'static Catalog {
        if gives_data(version) {
            Catalog::v13_4()
        } else {
            Catalog::v13_3()
        }
    }

    /// The generator with this artisan name.
    pub fn get(&self, name: &str) -> Option<&Generator> {
        self.generators
            .iter()
            .find(|generator| generator.name == name)
    }

    /// The generators in the groups and the order of S9.1. A generator that the
    /// order does not name goes to [`Group::Other`], sorted by name, whatever its
    /// P3 group. An empty group is left out.
    pub fn grouped(&self) -> Vec<(Group, Vec<&Generator>)> {
        let mut groups: Vec<(Group, Vec<&Generator>)> = ORDER
            .iter()
            .map(|(group, names)| {
                (
                    *group,
                    names.iter().filter_map(|name| self.get(name)).collect(),
                )
            })
            .collect();
        let mut other: Vec<&Generator> = self
            .generators
            .iter()
            .filter(|generator| {
                !ORDER
                    .iter()
                    .any(|(_, names)| names.contains(&generator.name.as_str()))
            })
            .collect();
        other.sort_by(|a, b| a.name.cmp(&b.name));
        groups.push((Group::Other, other));
        groups.retain(|(_, generators)| !generators.is_empty());
        groups
    }
}

/// True when the package ships the generator named `generator`: it is in the
/// catalog of 13.4.0 ([`Catalog::v13_4`]), `module:make` among them.
///
/// The package gives `--dry-run` and `--json` their meaning only for the options it
/// adds itself (`PlansGeneratorRun::addGeneratorRunOptions`). An app's own generator
/// may declare an option of that name and write its files anyway.
pub(crate) fn ships(generator: &str) -> bool {
    Catalog::v13_4().get(generator).is_some()
}

/// True when Mortise lists the files of `generator` by a dry run and reads the
/// result of its run from its answer (design D5, D6): the package gives data
/// ([`gives_data`]) and ships the generator ([`ships`]). An app's own generator
/// runs as before 13.4.
pub fn generator_gives_data(version: &Version, generator: &str) -> bool {
    gives_data(version) && ships(generator)
}

/// The version that [`Catalog::shipped`] describes, when it is not the app's own
/// major and minor version: `13.4` for 13.5.0, `13.3` for 13.2.0 or a dev version.
/// `None` when the shipped catalog is of the app's version.
pub fn shipped_mark(version: &Version) -> Option<&'static str> {
    let shipped = if gives_data(version) { "13.4" } else { "13.3" };
    (version.short() != shipped).then_some(shipped)
}

/// What the user filled in on the form of a generator.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filled {
    /// The text of each argument, by name. The module argument is not here.
    pub arguments: BTreeMap<String, String>,
    /// The flags that are on, by option name.
    pub flags: BTreeSet<String>,
    /// The text of each option with a value, by option name.
    pub values: BTreeMap<String, String>,
}

/// How Mortise checks a name the user types for an argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameRule {
    /// A class name: see [`class_name_ok`].
    Class,
    /// A module name: see [`module_name_ok`].
    Module,
    /// A name that is not a class name, such as a view: no white space and no leading `-`.
    Free,
}

impl NameRule {
    /// True when `name` passes the rule. An empty name never passes.
    pub fn accepts(self, name: &str) -> bool {
        match self {
            NameRule::Class => class_name_ok(name),
            NameRule::Module => module_name_ok(name),
            NameRule::Free => {
                !name.is_empty() && !name.starts_with('-') && !name.contains(char::is_whitespace)
            }
        }
    }
}

/// True for a class name: one or more segments split on `/` or `\`, each an ASCII
/// letter, then ASCII letters, digits or `_`. `create_posts_table` passes.
pub fn class_name_ok(name: &str) -> bool {
    name.split(['/', '\\']).all(|segment| {
        let mut chars = segment.chars();
        chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

/// True for a module name: an ASCII letter, then ASCII letters, digits, `-` or `_`.
/// The package also takes a leading `_`; Mortise does not.
pub fn module_name_ok(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

impl Generator {
    /// The words of the command the form describes, with `module` for the module argument.
    ///
    /// The words are the artisan name, then the arguments in their order, then the
    /// flags that are on and the options with a value, each in the order of the
    /// options. An array argument is split on white space into one word per name.
    /// An argument or a value is trimmed and left out when empty. A value is one
    /// word, `--<name>=<value>`, so no shell reads it. A flag or a value that the
    /// generator does not have is ignored, and so are `--dry-run` and `--json`,
    /// which Mortise adds itself.
    ///
    /// For `module:make-model`, `--all` stands in for its seven flags when they are
    /// all on or `all` is on. When an argument word starts with `-`, the arguments
    /// go last, after `--`, so that artisan does not read one as an option.
    pub fn words(&self, filled: &Filled, module: Option<&str>) -> Vec<String> {
        let mut arguments = Vec::new();
        for argument in &self.arguments {
            if argument.name == "module" {
                arguments.extend(module.filter(|module| !module.is_empty()).map(String::from));
                continue;
            }
            let text = filled
                .arguments
                .get(&argument.name)
                .map_or("", String::as_str);
            if argument.array {
                arguments.extend(text.split_whitespace().map(String::from));
            } else if !text.trim().is_empty() {
                arguments.push(text.trim().to_string());
            }
        }

        let all = self.name == MAKE_MODEL
            && self.flag("all")
            && (filled.flags.contains("all")
                || MODEL_FLAGS.iter().all(|flag| filled.flags.contains(*flag)));
        let mut options = Vec::new();
        let offered = |option: &&GeneratorOption| !ADDED.contains(&option.name.as_str());
        for option in self
            .options
            .iter()
            .filter(|option| !option.accepts_value)
            .filter(offered)
        {
            let on = if all && option.name == "all" {
                true
            } else if all && MODEL_FLAGS.contains(&option.name.as_str()) {
                false
            } else {
                filled.flags.contains(&option.name)
            };
            if on {
                options.push(format!("--{}", option.name));
            }
        }
        for option in self
            .options
            .iter()
            .filter(|option| option.accepts_value)
            .filter(offered)
        {
            let value = filled
                .values
                .get(&option.name)
                .map_or("", |value| value.trim());
            if !value.is_empty() {
                options.push(format!("--{}={value}", option.name));
            }
        }

        let mut words = vec![self.name.clone()];
        if arguments.iter().any(|word| word.starts_with('-')) {
            words.extend(options);
            words.push("--".to_string());
            words.extend(arguments);
        } else {
            words.extend(arguments);
            words.extend(options);
        }
        words
    }

    /// The rule for a name typed for `argument`: [`NameRule::Module`] for the
    /// names of `module:make` and for a module argument, [`NameRule::Free`] for the
    /// generators whose names are not class names, else [`NameRule::Class`].
    pub fn name_rule(&self, argument: &str) -> NameRule {
        if argument == "module" || self.name == MODULE_MAKE {
            NameRule::Module
        } else if FREE.contains(&self.name.as_str()) {
            NameRule::Free
        } else {
            NameRule::Class
        }
    }

    /// The first name typed for `argument` that its rule ([`Generator::name_rule`])
    /// refuses, or `None` when every name passes or none is typed. An array argument
    /// holds one name per word, as [`Generator::words`] splits it; another argument is
    /// one name, trimmed.
    pub fn refused_name<'t>(&self, argument: &str, text: &'t str) -> Option<&'t str> {
        let rule = self.name_rule(argument);
        let array = self
            .arguments
            .iter()
            .any(|known| known.name == argument && known.array);
        if array {
            return text.split_whitespace().find(|name| !rule.accepts(name));
        }
        let text = text.trim();
        (!text.is_empty() && !rule.accepts(text)).then_some(text)
    }

    /// True when the generator has a flag with this name.
    fn flag(&self, name: &str) -> bool {
        self.options
            .iter()
            .any(|option| option.name == name && !option.accepts_value)
    }
}

/// The calls of `module:make-model`: the flag, the generator, what the model adds
/// to its name to make the name it passes, and what the generator adds to that
/// name for its file. The factory gets the model name and writes `<Model>Factory.php`.
const MODEL_CALLS: [(&str, &str, &str, &str); 6] = [
    ("controller", "module:make-controller", "Controller", ""),
    ("factory", "module:make-factory", "", "Factory"),
    ("request", "module:make-request", "Request", ""),
    ("resource", "module:make-resource", "Resource", ""),
    ("seed", "module:make-seed", "Seeder", ""),
    ("policy", "module:make-policy", "Policy", ""),
];

/// The words that write `left` again with `--force`, if Mortise knows the
/// generator that wrote it. `run` holds the words of the run that left the file.
///
/// - A generator that writes one file: the run's words and `--force`.
/// - `module:make-model`: the file name is compared with the names the model
///   passes to its calls. A match gives that call with `--force`. The model's own
///   file gives the run's words without the seven flags and `--all`, with `--force`.
///   Before 13.4 `--factory` stays when the run made the factory, as the model's stub
///   uses it, and the factory call leaves its file. From 13.4 on the model passes
///   `--force` to its calls (package PR #2302), so `--factory` would write over the
///   factory too, and without it the stub loses `HasFactory`: a run with `--factory`
///   or `--all` gets `None`. A run without them writes only the model's file.
/// - A migration, `module:make-component` and `module:make`: `None`. A migration
///   gets a new date in its file name, so `--force` would not write the same file.
/// - A generator the package does not ship (an app's own): `None`. It may call
///   another generator and pass `--force` on, so it could write over more than `left`.
///
/// These are names the package passes, not paths: the generator writes where the
/// package says.
///
/// `runner` holds the option words of the runner's own line, before a `--` (a
/// Custom line `php artisan --policy` gives `--policy`). They go with every command,
/// and a replacement cannot take them out: they count as options of the run. From
/// 13.4 on, a runner word that sets any option of `module:make-model` but `--json`
/// gives the model's own file `None`, as the model would pass `--force` on to a call.
///
/// `package` is the app's package version.
pub fn replace_words(
    catalog: &Catalog,
    run: &[String],
    runner: &[String],
    left: &str,
    package: &Version,
) -> Option<Vec<String>> {
    let generator = catalog.get(run.first()?)?;
    if !ships(&generator.name) {
        return None;
    }
    match generator.name.as_str() {
        MAKE_MODEL => replace_from_model(catalog, generator, run, runner, left, package),
        MODULE_MAKE | "module:make-component" | "module:make-migration" => None,
        _ => generator.flag("force").then(|| with_force(run)),
    }
}

fn replace_from_model(
    catalog: &Catalog,
    model_generator: &Generator,
    run: &[String],
    runner: &[String],
    left: &str,
    package: &Version,
) -> Option<Vec<String>> {
    let stem = file_name(left).strip_suffix(".php")?;
    let (mut options, arguments) = split_run(run);
    options.extend(runner.iter().map(String::as_str));
    let model = studly(arguments.first()?);
    let module = arguments.get(1).map(|module| module.as_str());
    let all = turns_on(model_generator, &options, "all");

    if stem == file_name(&model) {
        if !model_generator.flag("force") {
            return None;
        }
        // The model stub renders its factory from `--factory`, so it stays; `--all`
        // gives it too. Before 13.4 the factory call then leaves its file: it has no
        // `--force`. From 13.4 on it gets `--force` and would write over the factory,
        // and without the flag the model loses its factory: no Replace.
        let factory = all || turns_on(model_generator, &options, "factory");
        if factory && !released_before_13_4(package) {
            return None;
        }
        // A runner word stays in the command: from 13.4 on, a call it turns on
        // gets `--force` from the model.
        if !released_before_13_4(package) && sets_an_option(model_generator, runner) {
            return None;
        }
        // Only a word before `--` is a flag: a module named `--factory` stays.
        let mut words: Vec<String> = Vec::with_capacity(run.len());
        let mut positional = false;
        for word in run {
            positional |= word == "--";
            let flag = word.strip_prefix("--").unwrap_or_default();
            let call = !positional && (flag == "all" || MODEL_FLAGS.contains(&flag));
            if call && factory && !words.iter().any(|kept| kept == "--factory") {
                words.push("--factory".to_owned());
            } else if !call {
                words.push(word.clone());
            }
        }
        return Some(with_force(&words));
    }

    let flag = model_flag_of(&model, left)?;
    if !(all || turns_on(model_generator, &options, flag)) {
        return None;
    }
    let (_, command, name_suffix, _) = MODEL_CALLS.iter().find(|(call, ..)| *call == flag)?;
    let call = catalog.get(command)?;
    if !call.flag("force") {
        return None;
    }
    let argument = call
        .arguments
        .iter()
        .find(|argument| argument.name != "module")?;
    let filled = Filled {
        arguments: BTreeMap::from([(argument.name.clone(), format!("{model}{name_suffix}"))]),
        flags: BTreeSet::from(["force".to_string()]),
        values: BTreeMap::new(),
    };
    Some(call.words(&filled, module))
}

/// True when the option words turn on the option `name` of `generator`: `--name`,
/// or its shortcut in a group of short options, such as `-f` or `-mf`. A runner's
/// own words can carry one (a Custom line `php artisan -f`), so a short group that
/// holds the letter counts, whatever else it holds.
fn turns_on(generator: &Generator, options: &[&str], name: &str) -> bool {
    let long = format!("--{name}");
    let shortcut = generator
        .options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| option.shortcut.as_deref())
        .filter(|shortcut| !shortcut.is_empty());
    options.iter().any(|word| {
        *word == long
            || word
                .strip_prefix('-')
                .filter(|group| !group.starts_with('-'))
                .zip(shortcut)
                .is_some_and(|(group, shortcut)| group.contains(shortcut))
    })
}

/// True when one of `words` sets an option of `generator` other than `--json`:
/// `--name` or `--name=value`, or a group of short options that holds a shortcut,
/// such as `-p` or `-mp`.
fn sets_an_option(generator: &Generator, words: &[String]) -> bool {
    let options = || {
        generator
            .options
            .iter()
            .filter(|option| option.name != "json")
    };
    words.iter().any(|word| match word.strip_prefix("--") {
        Some(long) => {
            let name = long.split_once('=').map_or(long, |(name, _)| name);
            options().any(|option| option.name == name)
        }
        None => word.strip_prefix('-').is_some_and(|group| {
            options()
                .filter_map(|option| option.shortcut.as_deref())
                .filter(|shortcut| !shortcut.is_empty())
                .any(|shortcut| group.contains(shortcut))
        }),
    })
}

/// True for a released version older than 13.4.0, whose model does not pass
/// `--force` to its calls. A dev build or a pre-release may pass it, so it is false.
fn released_before_13_4(version: &Version) -> bool {
    let parts: Vec<&str> = version.full().split('.').collect();
    let [major, minor, patch] = parts.as_slice() else {
        return false;
    };
    let number = |part: &str| {
        (!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| part.parse::<u64>().ok())
            .flatten()
    };
    match (number(major), number(minor), number(patch)) {
        (Some(major), Some(minor), Some(_)) => (major, minor) < (13, 4),
        _ => false,
    }
}

/// The flag of `module:make-model` whose call writes the file at `path`, for the
/// model typed as `model`: `policy` for `<Model>Policy.php`. `None` for the model's
/// own file, for a migration, whose file name carries the time, and for any other file.
pub fn model_flag_of(model: &str, path: &str) -> Option<&'static str> {
    let stem = file_name(path).strip_suffix(".php")?;
    let model = studly(model);
    MODEL_CALLS
        .iter()
        .find(|(_, _, name_suffix, file_suffix)| {
            let passed = format!("{model}{name_suffix}");
            stem == format!("{}{file_suffix}", file_name(&passed))
        })
        .map(|(flag, ..)| *flag)
}

/// The option words and the argument words of a run, after its artisan name.
/// Every word after `--` is an argument.
fn split_run(run: &[String]) -> (Vec<&str>, Vec<&String>) {
    let rest = run.get(1..).unwrap_or_default();
    let end = rest
        .iter()
        .position(|word| word == "--")
        .unwrap_or(rest.len());
    let (before, after) = rest.split_at(end);
    let options = before
        .iter()
        .filter(|word| word.starts_with('-'))
        .map(String::as_str)
        .collect();
    let arguments = before
        .iter()
        .filter(|word| !word.starts_with('-'))
        .chain(after.iter().skip(1))
        .collect();
    (options, arguments)
}

/// The run's words with `--force`, before a `--`. A run that has it stays as it is.
fn with_force(run: &[String]) -> Vec<String> {
    let end = run
        .iter()
        .position(|word| word == "--")
        .unwrap_or(run.len());
    let mut words = run.to_vec();
    if !run[..end].iter().any(|word| word == "--force") {
        words.insert(end, "--force".to_string());
    }
    words
}

/// The part of a path after the last `/` or `\`.
fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// The name as Laravel's `Str::studly` writes it: `-` and `_` are spaces, each
/// word starts with a capital, and the spaces go.
fn studly(name: &str) -> String {
    name.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect::<String>())
                .unwrap_or_default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn words(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| word.to_string()).collect()
    }

    /// A P3 entry with these arguments (name, array), flags and value options.
    fn generator(
        name: &str,
        arguments: &[(&str, bool)],
        flags: &[&str],
        values: &[&str],
    ) -> serde_json::Value {
        let option = |name: &str, accepts_value: bool| {
            json!({
                "name": name, "shortcut": null, "accepts_value": accepts_value,
                "value_required": false, "array": false, "negatable": false,
                "default": if accepts_value { json!(null) } else { json!(false) },
                "description": "",
            })
        };
        json!({
            "name": name,
            "group": "other",
            "description": "",
            "arguments": arguments.iter().map(|(name, array)| json!({
                "name": name, "required": true, "array": array,
                "default": null, "description": "",
            })).collect::<Vec<_>>(),
            "options": flags.iter().map(|flag| option(flag, false))
                .chain(values.iter().map(|value| option(value, true)))
                .collect::<Vec<_>>(),
        })
    }

    fn catalog(generators: Vec<serde_json::Value>) -> Catalog {
        Catalog::from_json(&json!({ "generators": generators }).to_string()).unwrap()
    }

    /// The model generator with its options in the package's order, the calls it
    /// makes, `module:make` and a view.
    fn sample() -> Catalog {
        let file = |name: &str, argument: &str| {
            generator(
                name,
                &[(argument, false), ("module", false)],
                &["force"],
                &[],
            )
        };
        let mut model = generator(
            MAKE_MODEL,
            &[("model", false), ("module", false)],
            &[
                "all",
                "attributes",
                "controller",
                "factory",
                "migration",
                "request",
                "resource",
                "policy",
                "seed",
                "force",
            ],
            &["fillable"],
        );
        // The package lists `fillable` between `controller` and `factory`.
        let options = model["options"].as_array_mut().unwrap();
        let fillable = options.pop().unwrap();
        options.insert(3, fillable);
        catalog(vec![
            generator(MODULE_MAKE, &[("name", true)], &["api", "force"], &[]),
            model,
            file("module:make-controller", "controller"),
            file("module:make-factory", "name"),
            file("module:make-policy", "name"),
            file("module:make-migration", "name"),
            file("module:make-view", "name"),
            generator(
                "module:make-test",
                &[("name", false), ("module", false)],
                &[],
                &[],
            ),
            file("app:make-widget", "name"),
        ])
    }

    #[test]
    fn the_shipped_catalogs_have_the_generators_of_their_version() {
        let catalog = Catalog::v13_3();
        assert_eq!(catalog.generators.len(), 42);
        for (_, names) in ORDER {
            for name in names {
                if *name != "module:make-health-indicator" {
                    assert!(catalog.get(name).is_some(), "{name}");
                }
            }
        }
        // 13.4 adds the health indicator: every generator has its place in S9.1.
        let catalog = Catalog::v13_4();
        assert_eq!(catalog.generators.len(), 43);
        for (_, names) in ORDER {
            for name in names {
                assert!(catalog.get(name).is_some(), "{name}");
            }
        }
        assert!(
            catalog
                .grouped()
                .iter()
                .all(|(group, _)| *group != Group::Other)
        );
    }

    #[test]
    fn an_apps_own_generator_goes_to_other_without_a_module_word() {
        // `app:make-widget` is a generator of the app's own, with no module argument.
        let catalog = catalog(vec![
            generator("module:make-zeta", &[], &[], &[]),
            generator(MAKE_MODEL, &[], &[], &[]),
            generator("app:make-widget", &[("name", false)], &[], &[]),
            generator(MODULE_MAKE, &[], &[], &[]),
        ]);
        let grouped: Vec<(Group, Vec<&str>)> = catalog
            .grouped()
            .into_iter()
            .map(|(group, generators)| {
                let names = generators.iter().map(|g| g.name.as_str()).collect();
                (group, names)
            })
            .collect();
        assert_eq!(
            grouped,
            [
                (Group::Module, vec![MODULE_MAKE]),
                (Group::Data, vec![MAKE_MODEL]),
                (Group::Other, vec!["app:make-widget", "module:make-zeta"]),
            ]
        );
        let filled = Filled {
            arguments: BTreeMap::from([("name".to_string(), "Widget".to_string())]),
            ..Filled::default()
        };
        assert_eq!(
            catalog
                .get("app:make-widget")
                .unwrap()
                .words(&filled, Some("Billing")),
            words(&["app:make-widget", "Widget"])
        );
    }

    #[test]
    fn words_follow_the_form() {
        let catalog = sample();
        let model = catalog.get(MAKE_MODEL).unwrap();
        let filled = |name: &str, flags: &[&str], values: &[(&str, &str)]| Filled {
            arguments: BTreeMap::from([("model".to_string(), name.to_string())]),
            flags: flags.iter().map(|flag| flag.to_string()).collect(),
            values: values
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        };

        // Flags in the order of the options, then values. A value is trimmed and is
        // one word; an empty one and an option the generator lacks are left out.
        let form = filled(
            " PaymentIntent ",
            &["policy", "migration", "unknown"],
            &[("fillable", " order_id, amount "), ("unknown", "x")],
        );
        assert_eq!(
            model.words(&form, Some("Billing")),
            words(&[
                MAKE_MODEL,
                "PaymentIntent",
                "Billing",
                "--migration",
                "--policy",
                "--fillable=order_id, amount",
            ])
        );
        let form = filled("PaymentIntent", &[], &[("fillable", "  ")]);
        assert_eq!(
            model.words(&form, None),
            words(&[MAKE_MODEL, "PaymentIntent"])
        );

        // `--all` stands in for the seven flags, when they are all on or `all` is on.
        let expected = words(&[MAKE_MODEL, "Invoice", "Billing", "--all", "--attributes"]);
        let mut seven: Vec<&str> = MODEL_FLAGS.to_vec();
        seven.push("attributes");
        assert_eq!(
            model.words(&filled("Invoice", &seven, &[]), Some("Billing")),
            expected
        );
        assert_eq!(
            model.words(
                &filled("Invoice", &["all", "policy", "attributes"], &[]),
                Some("Billing")
            ),
            expected
        );

        // An array argument gives one word per name.
        let module_make = catalog.get(MODULE_MAKE).unwrap();
        let names = Filled {
            arguments: BTreeMap::from([("name".to_string(), "  Blog \t Shop ".to_string())]),
            flags: BTreeSet::from(["api".to_string()]),
            values: BTreeMap::new(),
        };
        assert_eq!(
            module_make.words(&names, None),
            words(&[MODULE_MAKE, "Blog", "Shop", "--api"])
        );

        // A name that starts with `-` would read as an option: the arguments go last.
        assert_eq!(
            model.words(&filled("-Odd", &["policy"], &[]), Some("Billing")),
            words(&[MAKE_MODEL, "--policy", "--", "-Odd", "Billing"])
        );

        // The 13.4 model has `--dry-run` and `--json`: Mortise adds them itself.
        let model = Catalog::v13_4().get(MAKE_MODEL).unwrap();
        assert_eq!(
            model.words(
                &filled("Invoice", &["policy", "dry-run", "json"], &[]),
                Some("Billing")
            ),
            words(&[MAKE_MODEL, "Invoice", "Billing", "--policy"])
        );
    }

    #[test]
    fn names_follow_their_rule() {
        let catalog = sample();
        let rule =
            |generator: &str, argument: &str| catalog.get(generator).unwrap().name_rule(argument);
        assert_eq!(rule(MAKE_MODEL, "model"), NameRule::Class);
        assert_eq!(rule(MAKE_MODEL, "module"), NameRule::Module);
        assert_eq!(rule(MODULE_MAKE, "name"), NameRule::Module);
        assert_eq!(rule("module:make-view", "name"), NameRule::Free);

        // Each word of an array argument is a name; another argument is one name.
        let widgets = self::catalog(vec![generator(
            "app:make-widgets",
            &[("names", true)],
            &[],
            &[],
        )]);
        let widgets = widgets.get("app:make-widgets").unwrap();
        assert_eq!(widgets.refused_name("names", " First  Second "), None);
        assert_eq!(widgets.refused_name("names", "First 2nd"), Some("2nd"));
        assert_eq!(widgets.refused_name("names", "  "), None);
        let model = catalog.get(MAKE_MODEL).unwrap();
        assert_eq!(
            model.refused_name("model", " First Second "),
            Some("First Second")
        );

        let table: [(NameRule, &str, bool); 21] = [
            (NameRule::Class, "PaymentIntent", true),
            (NameRule::Class, "Admin/Post", true),
            (NameRule::Class, "Admin\\Post_2", true),
            (NameRule::Class, "create_posts_table", true),
            (NameRule::Class, "payment intent", false),
            (NameRule::Class, "2fa", false),
            (NameRule::Class, "Admin//Post", false),
            (NameRule::Class, "Post-Card", false),
            (NameRule::Class, "", false),
            (NameRule::Module, "Blog", true),
            (NameRule::Module, "blog-post_2", true),
            (NameRule::Module, "2fa", false),
            (NameRule::Module, "_blog", false),
            (NameRule::Module, "-blog", false),
            (NameRule::Module, "Blog/Post", false),
            (NameRule::Module, "", false),
            (NameRule::Free, "user-card", true),
            (NameRule::Free, "posts.index", true),
            (NameRule::Free, "user card", false),
            (NameRule::Free, "-card", false),
            (NameRule::Free, "", false),
        ];
        for (rule, name, ok) in table {
            assert_eq!(rule.accepts(name), ok, "{rule:?} {name:?}");
        }
    }

    #[test]
    fn the_shipped_catalog_is_marked_for_another_version() {
        // The version, the catalog shipped for it, and whether the picker marks it.
        for (raw, shipped, marked) in [
            ("13.3.0", "13.3", false),
            ("v13.3.12", "13.3", false),
            ("13.30.0", "13.4", true),
            ("13.2.0", "13.3", true),
            ("13.x-dev", "13.3", true),
            ("13.4.0-RC1", "13.3", true),
            ("v13.4.0", "13.4", false),
            ("13.5.0", "13.4", true),
        ] {
            let version = Version {
                raw: raw.to_string(),
                alias: None,
            };
            assert_eq!(shipped_mark(&version), marked.then_some(shipped), "{raw}");
            let catalog = match shipped {
                "13.3" => Catalog::v13_3(),
                _ => Catalog::v13_4(),
            };
            assert!(std::ptr::eq(Catalog::shipped(&version), catalog), "{raw}");
        }
    }

    #[test]
    fn a_models_flag_is_found_by_the_file_its_call_writes() {
        let flag = |model: &str, path: &str| model_flag_of(model, path);
        let policies = "Modules/Billing/app/Policies";
        assert_eq!(
            flag(
                "payment_intent",
                &format!("{policies}/PaymentIntentPolicy.php")
            ),
            Some("policy")
        );
        assert_eq!(flag("Invoice", "InvoiceFactory.php"), Some("factory"));
        assert_eq!(flag("Invoice", "InvoiceSeeder.php"), Some("seed"));
        assert_eq!(
            flag("Admin/Invoice", "InvoiceController.php"),
            Some("controller")
        );
        // The model's own file, a migration and another model's file have no flag.
        assert_eq!(flag("Invoice", "Invoice.php"), None);
        assert_eq!(
            flag("Invoice", "2026_11_23_120000_create_invoices_table.php"),
            None
        );
        assert_eq!(flag("Invoice", "PaymentPolicy.php"), None);
    }

    #[test]
    fn replace_writes_a_left_file_with_force() {
        let catalog = sample();
        let version = |raw: &str| Version {
            raw: raw.to_string(),
            alias: None,
        };
        let v13_3 = version("13.3.0");
        let replace =
            |run: &[&str], left: &str| replace_words(&catalog, &words(run), &[], left, &v13_3);

        // One generator: its words and `--force`, before a `--`.
        assert_eq!(
            replace(
                &["module:make-policy", "InvoicePolicy", "Billing"],
                "Modules/Billing/app/Policies/InvoicePolicy.php"
            ),
            Some(words(&[
                "module:make-policy",
                "InvoicePolicy",
                "Billing",
                "--force"
            ]))
        );
        assert_eq!(
            replace(&["module:make-policy", "--", "-Odd", "Billing"], "-Odd.php"),
            Some(words(&[
                "module:make-policy",
                "--force",
                "--",
                "-Odd",
                "Billing"
            ]))
        );
        // A generator without `--force` cannot replace.
        assert_eq!(
            replace(&["module:make-test", "PostTest"], "PostTest.php"),
            None
        );
        // An app's own generator may pass `--force` to the generators it calls.
        assert_eq!(
            replace(&["app:make-widget", "Widget", "Billing"], "Widget.php"),
            None
        );

        // A model's calls get the names the model passes, made studly.
        let run = [
            MAKE_MODEL,
            "invoice",
            "Billing",
            "--migration",
            "--factory",
            "--policy",
            "--fillable=total",
        ];
        assert_eq!(
            replace(&run, "Modules/Billing/app/Policies/InvoicePolicy.php"),
            Some(words(&[
                "module:make-policy",
                "InvoicePolicy",
                "Billing",
                "--force"
            ]))
        );
        assert_eq!(
            replace(
                &run,
                "Modules\\Billing\\database\\factories\\InvoiceFactory.php"
            ),
            Some(words(&[
                "module:make-factory",
                "Invoice",
                "Billing",
                "--force"
            ]))
        );
        // The model's own file: the run without the flags of the calls but
        // `--factory`, which the model's stub reads. `--all` gives it too.
        assert_eq!(
            replace(&run, "Modules/Billing/app/Models/Invoice.php"),
            Some(words(&[
                MAKE_MODEL,
                "invoice",
                "Billing",
                "--factory",
                "--fillable=total",
                "--force"
            ]))
        );
        assert_eq!(
            replace(&[MAKE_MODEL, "Invoice", "Billing", "--all"], "Invoice.php"),
            Some(words(&[
                MAKE_MODEL,
                "Invoice",
                "Billing",
                "--factory",
                "--force"
            ]))
        );
        // From 13.4 on (and on a dev build) the model passes `--force` to its calls:
        // with `--factory` it would write over the factory, and without it the model
        // loses its factory, so a run with `--factory` gets nothing. A run without it
        // writes only the model's file. `--json` stays.
        let model_file = "Modules/Billing/app/Models/Invoice.php";
        let without_factory: Vec<&str> = run
            .iter()
            .copied()
            .filter(|word| *word != "--factory")
            .chain(["--json"])
            .collect();
        for raw in ["13.4.0", "dev-master"] {
            let replace =
                |run: &[&str]| replace_words(&catalog, &words(run), &[], model_file, &version(raw));
            assert_eq!(replace(&run), None, "{raw}");
            assert_eq!(
                replace(&[MAKE_MODEL, "Invoice", "Billing", "--all"]),
                None,
                "{raw}"
            );
            assert_eq!(
                replace(&without_factory),
                Some(words(&[
                    MAKE_MODEL,
                    "invoice",
                    "Billing",
                    "--fillable=total",
                    "--json",
                    "--force"
                ])),
                "{raw}"
            );
            // A module named `--factory`, after `--`, is a name, not the flag.
            assert_eq!(
                replace(&[MAKE_MODEL, "--", "Invoice", "--factory"]),
                Some(words(&[
                    MAKE_MODEL,
                    "--force",
                    "--",
                    "Invoice",
                    "--factory"
                ])),
                "{raw}"
            );
        }
        // A runner's own words stay in the command: from 13.4 on, one that sets an
        // option of the model (the shortcut `-f` in a group, or `--policy`) gives
        // the model's file no Replace. Docker's `-T` is no option of the model.
        let v13_4 = version("13.4.0");
        let invoice = words(&[MAKE_MODEL, "Invoice", "Billing"]);
        let by_runner = |runner: &[&str]| {
            replace_words(
                Catalog::v13_4(),
                &invoice,
                &words(runner),
                model_file,
                &v13_4,
            )
        };
        assert_eq!(by_runner(&["-T", "-mf"]), None);
        assert_eq!(by_runner(&["--policy"]), None);
        assert!(by_runner(&["-T"]).is_some());
        // A call the run did not make, and a migration, get nothing.
        assert_eq!(replace(&run, "InvoiceController.php"), None);
        assert_eq!(
            replace(
                &run,
                "Modules/Billing/database/migrations/2026_11_23_120000_create_invoices_table.php"
            ),
            None
        );
        assert_eq!(
            replace(
                &["module:make-migration", "create_invoices_table", "Billing"],
                "2026_11_23_120000_create_invoices_table.php"
            ),
            None
        );
    }
}
