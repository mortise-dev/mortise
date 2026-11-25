//! Text of the command line tool (section 10 of the specification). The specification
//! draws no command line, so none of this text is in the spec. The sentences of the
//! findings follow those of the Findings list.

use mortise_core::{Check, Severity};

pub const HELP: &str = "\
Mortise checks the boundary rules of a Laravel app that uses nwidart/laravel-modules.

Usage: mortise check [PATH] [OPTIONS]

PATH is the app folder. It defaults to the current folder.

Options:
  --coverage <clover.xml>     The coverage report. Wins over mortise.json
  --coverage-limit <0-100>    A module covered less is a warning. Wins over mortise.json
  --format text|json|github   How the findings are printed. Default: text
  --fail-on warnings          Warnings fail the check too
  -h, --help                  Print this help
  -V, --version               Print the version

Exit codes:
  0  No finding fails the check, or mortise.json sets \"ci\": { \"fail\": false }
  1  A finding fails the check
  2  The command line is wrong
  3  The folder cannot be read as an app
";

pub const NO_COMMAND: &str = "No command was given.";

/// Said when `mortise.json` sets `ci.fail` to false and a finding would fail the check.
pub const CI_FAIL_OFF: &str =
    "mortise.json turns off \"Fail CI when a rule breaks\", so the check passes.";

/// `mortise: --format takes text, json or github, not xml. Run mortise --help for the options.`
pub fn usage_error(message: &str) -> String {
    format!("mortise: {message} Run mortise --help for the options.")
}

pub fn unknown_command(command: &str) -> String {
    format!("{command} is not a command. The command is check.")
}

pub fn unknown_flag(flag: &str) -> String {
    format!("{flag} is not an option of check.")
}

pub fn extra_argument(argument: &str) -> String {
    format!("check takes one folder. {argument} is one too many.")
}

pub fn missing_value(flag: &str) -> String {
    format!("{flag} needs a value.")
}

pub fn wrong_limit(value: &str) -> String {
    format!("--coverage-limit takes a number from 0 to 100, not {value}.")
}

pub fn wrong_format(value: &str) -> String {
    format!("--format takes text, json or github, not {value}.")
}

pub fn wrong_fail_on(value: &str) -> String {
    format!("--fail-on takes warnings, not {value}.")
}

pub fn no_report(path: &str) -> String {
    format!("The coverage report {path} is not a file.")
}

/// `mortise: /tmp is not a Laravel app. Run mortise check in the folder of an app ...`
pub fn not_an_app(error: &str) -> String {
    format!(
        "mortise: {error}. Run mortise check in the folder of an app that uses \
         nwidart/laravel-modules, or give that folder."
    )
}

pub fn invalid_rules_file(message: &str) -> String {
    format!("mortise: mortise.json cannot be read: {message}. The default rules apply.")
}

pub fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Problem => "problem",
        Severity::Warning => "warning",
    }
}

/// The title of a line in the CI log.
pub fn check(check: Check) -> &'static str {
    match check {
        Check::NoCircularUse => "No circular use",
        Check::PublicSurfaceOnly => "Public surface only",
        Check::ValidManifest => "Valid module.json",
        Check::NamespaceMatchesFolder => "Namespace",
        Check::NoStaleStatus => "Statuses file",
        Check::CoverageLimit => "Coverage limit",
        Check::TranslationsComplete => "Translations",
    }
}

/// `2 problems, 1 warning. 7 modules checked against 2 rules and 4 checks.`
pub fn summary(
    problems: usize,
    warnings: usize,
    modules: usize,
    rules: usize,
    checks: usize,
) -> String {
    let checked = format!(
        "{} checked against {} and {}.",
        count(modules, "module", "modules"),
        count(rules, "rule", "rules"),
        count(checks, "check", "checks")
    );
    if problems + warnings == 0 {
        format!("No findings. {checked}")
    } else {
        format!(
            "{}, {}. {checked}",
            count(problems, "problem", "problems"),
            count(warnings, "warning", "warnings")
        )
    }
}

/// `Checkout and Shipping use each other. 2 files in Checkout, 4 files in Shipping.`
pub fn circle(files: &[(String, usize)]) -> String {
    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    let head = match names.as_slice() {
        [a, b] => format!("{a} and {b} use each other"),
        names => format!("{} use each other in a circle", list_of(names)),
    };
    let counts: Vec<String> = files
        .iter()
        .map(|(name, n)| format!("{} in {name}", count(*n, "file", "files")))
        .collect();
    format!("{head}. {}.", counts.join(", "))
}

/// `Checkout reads Modules\Billing\Models\Invoice directly. Billing offers Contracts and Events.`
pub fn reads_internals(module: &str, of: &str, classes: &[&str], public: &[String]) -> String {
    let public: Vec<&str> = public.iter().map(String::as_str).collect();
    format!(
        "{module} reads {} directly. {of} offers {}.",
        list_of(classes),
        list_of(&public)
    )
}

pub fn invalid_manifest(module: &str, message: &str) -> String {
    format!("The module.json of {module} is not valid JSON: {message}.")
}

pub fn no_manifest_name(module: &str) -> String {
    format!("The module.json of {module} has no name. The package needs one.")
}

/// `Blog declares the namespace Modules\Blogs. Its folder is Modules/Blog.`
pub fn namespace_mismatch(module: &str, namespace: &str, folder: &str) -> String {
    format!("{module} declares the namespace {namespace}. Its folder is {folder}.")
}

/// `modules_statuses.json lists Legacy, which has no folder.`
pub fn stale_status(entry: &str, file: &str) -> String {
    format!("{file} lists {entry}, which has no folder.")
}

/// `Notifications is 39% covered. The limit is 50%.`
pub fn low_coverage(module: &str, percent: u8, limit: u8) -> String {
    format!("{module} is {percent}% covered. The limit is {limit}%.")
}

/// `Checkout lacks 2 keys and 1 file in fr.`
pub fn missing_translations(module: &str, locale: &str, keys: usize, files: usize) -> String {
    let what: Vec<String> = [(keys, "key", "keys"), (files, "file", "files")]
        .into_iter()
        .filter(|(n, _, _)| *n > 0)
        .map(|(n, one, many)| count(n, one, many))
        .collect();
    let what: Vec<&str> = what.iter().map(String::as_str).collect();
    format!("{module} lacks {} in {locale}.", list_of(&what))
}

fn count(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

fn list_of(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}
