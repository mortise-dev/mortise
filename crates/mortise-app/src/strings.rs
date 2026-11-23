//! Interface text, from section 10 of the specification.
//!
//! Sentence case, no exclamation marks, no "please". An error says what happened,
//! then what to do next. Text that is not in section 10 is marked "not in the spec".

use std::path::Path;

use mortise_core::{AppSnapshot, LayoutKey, ReadError, Version};

pub const ALL_MODULES: &str = "All modules";

// Actions (S10.1, S10.2)
pub const SEARCH: &str = "Search or run a command";
pub const MODULES_ON: &str = "Modules on";
pub const MODULE_STATE: &str = "Module state";
pub const ON: &str = "On";
pub const OFF: &str = "Off";
/// Not in the spec: the state of a module when the app does not use the file activator.
pub const STATE_UNKNOWN: &str = "Unknown";

pub const NO_APPS_BODY: &str = "Add the folder of a Laravel app that uses nwidart/laravel-modules. \
    Mortise reads its modules from disk and changes nothing until you run a command.";
pub const ADD_APP: &str = "Add app";
pub const CANCEL: &str = "Cancel";
pub const NAME: &str = "Name";
pub const MONOGRAM: &str = "Monogram";
pub const COPY_COMMAND: &str = "Copy command";
pub const COPIED_COMMAND: &str = "Copied the command.";

/// Not in the spec: the name of the cross on a toast, for a screen reader.
pub const CLOSE_MESSAGE: &str = "Close the message";

/// A whole number with a comma between thousands: `41,212`.
pub fn count(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// `84 files`
pub fn files(n: u64) -> String {
    if n == 1 {
        "1 file".to_owned()
    } else {
        format!("{} files", count(n))
    }
}

/// `7 of 7`
pub fn of(on: usize, total: usize) -> String {
    format!("{on} of {total}")
}

/// Not in the spec: the count beside the app name when the on or off state cannot be read.
pub fn modules(total: usize) -> String {
    if total == 1 {
        "1 module".to_owned()
    } else {
        format!("{total} modules")
    }
}

/// `Laravel 13.2, package 13.3.0`
pub fn versions(laravel: Option<&Version>, package: &Version) -> String {
    match laravel {
        Some(laravel) => format!("Laravel {}, package {}", laravel.short(), package.full()),
        None => format!("package {}", package.full()),
    }
}

/// A path as the user writes it: `~/code/shop-api`.
pub fn tilde(path: &Path) -> String {
    match std::env::home_dir().and_then(|home| path.strip_prefix(home).ok().map(Path::to_owned)) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        // `/` between the parts on every platform, as after the `~`.
        Some(rest) => {
            let parts: Vec<_> = rest
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect();
            format!("~/{}", parts.join("/"))
        }
        None => path.display().to_string(),
    }
}

fn folder_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| tilde(path))
}

/// Why a folder could not be read, and what to do next.
pub fn read_error(error: &ReadError) -> String {
    match error {
        ReadError::NotLaravel { root } => format!(
            "No Laravel app in {}. The folder has no artisan file and no composer.json that \
             requires laravel/framework. Choose the folder that holds artisan.",
            tilde(root)
        ),
        ReadError::NoLock { root } => format!(
            "No composer.lock in {}. Mortise reads versions from it. Run composer install in \
             the app, then choose the folder again.",
            tilde(root)
        ),
        ReadError::NoPackage { root } => format!(
            "{} does not use nwidart/laravel-modules. composer.lock has no entry for the \
             package. Run composer require nwidart/laravel-modules in the app, then choose \
             the folder again.",
            folder_name(root)
        ),
        // Not in the spec. The parser's message is quoted as it is.
        ReadError::InvalidLock { path, message } => format!(
            "{} is not valid JSON: {message}. Run composer install in the app, then choose \
             the folder again.",
            tilde(path)
        ),
        // Not in the spec. The system's message is quoted as it is.
        ReadError::Io { path, source } => {
            format!("Mortise could not read {}: {source}.", tilde(path))
        }
    }
}

/// Not in the spec: a note for settings of `config/modules.php` that need PHP to evaluate.
pub fn assumed(app: &AppSnapshot, name: &str) -> Option<String> {
    let keys: Vec<&str> = app
        .layout
        .assumed
        .iter()
        .map(|key| key.config_key())
        .collect();
    let listed = match keys[..] {
        [] => return None,
        [one] => one.to_owned(),
        [ref rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    };
    let verb = if keys.len() == 1 { "needs" } else { "need" };
    let follows = if app.layout.assumed.contains(&LayoutKey::ModulesFolder) {
        format!(
            " If the modules of {name} are not in {}, this list is wrong.",
            app.modules_dir_shown()
        )
    } else {
        String::new()
    };
    Some(format!(
        "{listed} in config/modules.php {verb} PHP to work out. Mortise uses the default of \
         the package instead.{follows}"
    ))
}

/// Not in the spec: why the on or off state is unknown.
pub fn state_not_in_a_file(app: &AppSnapshot, name: &str) -> String {
    format!(
        "{name} keeps the state of its modules with the activator \"{}\", not in a file. \
         Mortise reads files only, so it cannot tell which modules are on.",
        app.layout.activator
    )
}

/// Not in the spec: an app whose modules folder holds no module.
pub fn no_modules(app: &AppSnapshot, name: &str) -> String {
    format!("{name} has no modules in {}.", app.modules_dir_shown())
}

pub const LINES_OF_PHP: &str = "Lines of PHP";
pub const TEST_COVERAGE: &str = "Test coverage";
/// The word for a coverage that cannot be worked out: there is no report.
pub const UNKNOWN: &str = "unknown";
pub const COVERAGE_BY_MODULE: &str = "Coverage by module";
pub const BY_MODULE: &str = "By module";
pub const FILES_BY_KIND: &str = "Files by kind";
pub const GIT_CHURN: &str = "Git churn";
pub const MIGRATIONS: &str = "Migrations";
pub const COVERAGE_REPORT: &str = "Coverage report";
pub const GIT: &str = "Git";
/// Not in the spec: the Git line of "Found in this folder" outside a repository.
pub const NOT_A_REPOSITORY: &str = "not a repository";
/// Not in the spec: the Git line when the git program cannot be started.
pub const GIT_NOT_FOUND: &str = "git was not found";

/// The headers of the "By module" table.
pub const COL_MODULE: &str = "Module";
pub const COL_FILES: &str = "Files";
pub const COL_LINES: &str = "Lines";
pub const COL_TESTS: &str = "Tests";
pub const COL_COVERAGE: &str = "Coverage";
pub const COL_COMMITS: &str = "Commits, 30 days";

/// `54%`
pub fn percent(percent: u8) -> String {
    format!("{percent}%")
}

/// `307 tests`
pub fn tests(n: u32) -> String {
    if n == 1 {
        "1 test".to_owned()
    } else {
        format!("{} tests", count(u64::from(n)))
    }
}

/// `84 files, 6,412 lines`
pub fn files_and_lines(files_count: u64, lines: u64) -> String {
    let lines = if lines == 1 {
        "1 line".to_owned()
    } else {
        format!("{} lines", count(lines))
    };
    format!("{}, {lines}", files(files_count))
}

/// `sorted by name`
pub fn sorted_by(column: &str, ascending: bool) -> String {
    let by = if column == COL_MODULE {
        "name".to_owned()
    } else {
        column.to_lowercase()
    };
    if ascending {
        format!("sorted by {by}")
    } else {
        format!("sorted by {by}, highest first")
    }
}

/// How long ago, in whole units: `8 s ago`, `2 minutes ago`, `2 days ago`.
pub fn ago(now: i64, then: i64) -> String {
    let seconds = (now - then).max(0);
    let unit = |n: i64, one: &str, many: &str| {
        if n == 1 {
            format!("1 {one} ago")
        } else {
            format!("{n} {many} ago")
        }
    };
    match seconds {
        0..60 => format!("{seconds} s ago"),
        60..3_600 => unit(seconds / 60, "minute", "minutes"),
        3_600..172_800 => unit(seconds / 3_600, "hour", "hours"),
        172_800..5_184_000 => unit(seconds / 86_400, "day", "days"),
        5_184_000..63_072_000 => unit(seconds / 2_592_000, "month", "months"),
        _ => unit(seconds / 31_536_000, "year", "years"),
    }
}

/// `14 commits in 30 days, last 2 days ago`
pub fn churn(commits: u32, last: Option<i64>, now: i64) -> String {
    let count = match commits {
        0 => "no commits in 30 days".to_owned(),
        1 => "1 commit in 30 days".to_owned(),
        n => format!("{n} commits in 30 days"),
    };
    match last {
        Some(last) => format!("{count}, last {}", ago(now, last)),
        // Not in the spec: a module that no commit ever touched.
        None => "never committed".to_owned(),
    }
}

/// `From build/coverage/clover.xml, written 2 days ago`
pub fn coverage_from(path: &str, written: Option<i64>, now: i64) -> String {
    match written {
        Some(written) => format!("From {path}, written {}", ago(now, written)),
        None => format!("From {path}"),
    }
}

pub const DEPENDENCIES: &str = "Dependencies";
pub const USED_BY: &str = "Used by";
pub const USES: &str = "Uses";
pub const RULE_NO_CIRCULAR: &str = "No circular use";
pub const RULE_PUBLIC_SURFACE: &str = "Other modules use only a module's public surface";

pub const COL_USES: &str = "Uses";
pub const COL_USED_BY: &str = "Used by";
pub const GOOD: &str = "Good";

/// `4 out, 2 in`
pub fn out_in(out: usize, into: usize) -> String {
    format!("{out} out, {into} in")
}

/// `14 refs`
pub fn refs(n: usize) -> String {
    if n == 1 {
        "1 ref".to_owned()
    } else {
        format!("{n} refs")
    }
}

/// `6 refs, 3 to internals` and `2 refs, circular`
pub fn refs_state(link: &mortise_core::Link, rules: &mortise_core::Rules) -> String {
    let internal = link
        .refs
        .iter()
        .filter(|reference| reference.internal)
        .count();
    if rules.public_surface_only && internal > 0 {
        format!("{}, {internal} to internals", refs(link.refs.len()))
    } else if rules.no_circular_use && link.circular {
        format!("{}, circular", refs(link.refs.len()))
    } else {
        refs(link.refs.len())
    }
}

/// `Checkout uses Billing`
pub fn a_uses_b(a: &str, b: &str) -> String {
    format!("{a} uses {b}")
}

/// `Checkout and Shipping use each other`
pub fn use_each_other(a: &str, b: &str) -> String {
    format!("{a} and {b} use each other")
}

/// Not in the spec: a circle of more than two modules.
pub fn use_each_other_in_a_circle(modules: &[String]) -> String {
    match modules {
        [rest @ .., last] => format!("{} and {last} use each other in a circle", rest.join(", ")),
        [] => String::new(),
    }
}

/// `2 problems`
pub fn problems(n: usize) -> String {
    if n == 1 {
        "1 problem".to_owned()
    } else {
        format!("{n} problems")
    }
}

pub const FINDINGS: &str = "Findings";
pub const COL_FINDINGS: &str = "Findings";
pub const LOW_COVERAGE: &str = "Low coverage";
pub const WARNING: &str = "Warning";
pub const SUMMARY: &str = "Summary";
pub const BOUNDARY: &str = "Boundary";
pub const ACTIONS: &str = "Actions";

/// What a locale lacks: `4 keys in en are missing in fr`. `from` is the other locale
/// of a module with two. Not in the spec: more locales, and missing files.
fn missing_translations(locale: &str, keys: usize, files: usize, from: Option<&str>) -> String {
    let keys_part = match keys {
        0 => None,
        1 => Some("1 key".to_owned()),
        n => Some(format!("{n} keys")),
    };
    let files_part = match files {
        0 => None,
        1 => Some("1 file".to_owned()),
        n => Some(format!("{n} files")),
    };
    let what = match (keys_part, files_part) {
        (Some(keys), Some(files)) => format!("{keys} and {files}"),
        (Some(one), None) | (None, Some(one)) => one,
        (None, None) => String::new(),
    };
    let verb = if keys + files == 1 { "is" } else { "are" };
    match from {
        Some(from) => format!("{what} in {from} {verb} missing in {locale}"),
        None => format!("{what} {verb} missing in {locale}"),
    }
}

/// `1 warning`
pub fn warnings(n: usize) -> String {
    if n == 1 {
        "1 warning".to_owned()
    } else {
        format!("{n} warnings")
    }
}

/// `Findings in shop-api`
pub fn findings_in(app: &str) -> String {
    format!("Findings in {app}")
}

/// `Show all 3 findings`
pub fn show_all_findings(n: usize) -> String {
    if n == 1 {
        "Show the finding".to_owned()
    } else {
        format!("Show all {n} findings")
    }
}

pub(crate) fn list_of(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The sentence of a finding, as in section 10 of the specification.
/// `name` is the display name of the app.
pub fn finding(finding: &mortise_core::Finding, app: &AppSnapshot, name: &str) -> String {
    use mortise_core::Detail;
    let module = finding.module.as_deref().unwrap_or_default();
    match &finding.detail {
        // `Checkout and Shipping use each other. 2 files in Checkout, 4 files in Shipping.`
        Detail::CircularUse { files: in_each } => {
            let names: Vec<String> = in_each.iter().map(|(name, _)| name.clone()).collect();
            let counts: Vec<String> = in_each
                .iter()
                .map(|(name, n)| format!("{} in {name}", files(*n as u64)))
                .collect();
            let head = match names.as_slice() {
                [a, b] => use_each_other(a, b),
                names => use_each_other_in_a_circle(names),
            };
            format!("{head}. {}.", counts.join(", "))
        }
        // `3 files in Checkout read Modules\Billing\Models\Invoice directly. Billing offers
        // Contracts and Events.`
        Detail::ReadsInternals {
            of,
            files: n,
            classes,
        } => {
            let what = match classes.as_slice() {
                [class] => class.clone(),
                // Not in the spec: more than one class.
                classes => format!("{} classes of {of}", classes.len()),
            };
            let verb = if *n == 1 { "reads" } else { "read" };
            format!(
                "{} in {module} {verb} {what} directly. {of} offers {}.",
                files(*n as u64),
                list_of(&app.rules.public_folders)
            )
        }
        // `Modules/Loyalty/module.json is not valid JSON: unexpected end of file at line 14.`
        // The parser's own words are quoted as they are.
        Detail::InvalidManifest {
            file,
            message,
            line,
        } => {
            format!("{file} is not valid JSON: {message} at line {line}.")
        }
        // Not in the spec.
        Detail::NoManifestName { file } => format!("{file} has no name. The package needs one."),
        // `Blog declares the namespace Modules\Blogs. Its folder is Modules/Blog.`
        Detail::NamespaceMismatch { namespace, folder } => {
            format!("{module} declares the namespace {namespace}. Its folder is {folder}.")
        }
        // `modules_statuses.json lists Legacy, which has no folder.`
        Detail::StaleStatus { entry, file } => {
            format!("{file} lists {entry}, which has no folder.")
        }
        // `Notifications is 39% covered. The limit for shop-api is 50%.`
        Detail::LowCoverage { percent, limit } => {
            format!("{module} is {percent}% covered. The limit for {name} is {limit}%.")
        }
        // `Checkout: 4 keys in en are missing in fr.`
        Detail::MissingTranslations {
            locale,
            keys,
            files,
            from,
        } => format!(
            "{module}: {}.",
            missing_translations(locale, *keys, *files, from.as_deref())
        ),
    }
}

/// The class without the namespace all modules share: `Billing\Models\Invoice`.
fn short_class<'a>(class: &'a str, app: &AppSnapshot) -> &'a str {
    class
        .strip_prefix(app.layout.namespace.as_str())
        .and_then(|rest| rest.strip_prefix('\\'))
        .unwrap_or(class)
}

/// The findings of one module as clauses of one sentence. `own` is true on the
/// module's own Summary: `2 files here, 4 files there` and the full class name.
fn clauses(module: &str, app: &AppSnapshot, name: &str, own: bool) -> Vec<String> {
    use mortise_core::Detail;
    app.findings
        .of(module)
        .map(|found| match &found.detail {
            Detail::CircularUse { files: in_each } => {
                let total: usize = in_each.iter().map(|(_, n)| n).sum();
                let others: Vec<String> = in_each
                    .iter()
                    .filter(|(name, _)| name != module)
                    .map(|(name, _)| name.clone())
                    .collect();
                let here = in_each
                    .iter()
                    .find(|(name, _)| name == module)
                    .map_or(0, |(_, n)| *n);
                match (own, others.as_slice()) {
                    (true, [other]) => format!(
                        "{module} and {other} use each other ({} here, {} there)",
                        files(here as u64),
                        files((total - here) as u64)
                    ),
                    (false, [other]) => {
                        format!("it and {other} use each other ({})", files(total as u64))
                    }
                    (_, others) => format!(
                        "it is in a circle with {} ({})",
                        list_of(others),
                        files(total as u64)
                    ),
                }
            }
            Detail::ReadsInternals {
                of,
                files: n,
                classes,
            } => {
                let what = match classes.as_slice() {
                    [class] if own => class.clone(),
                    [class] => short_class(class, app).to_owned(),
                    classes => format!("{} classes of {of}", classes.len()),
                };
                let verb = if *n == 1 { "reads" } else { "read" };
                if own {
                    format!("{} {verb} {what} directly", files(*n as u64))
                } else {
                    format!("{n} of its files {verb} {what} directly")
                }
            }
            // Not in the spec: the sentence without the module in front.
            Detail::MissingTranslations {
                locale,
                keys,
                files,
                from,
            } => missing_translations(locale, *keys, *files, from.as_deref()),
            _ => {
                let sentence = finding(found, app, name);
                let mut letters = sentence.trim_end_matches('.').chars();
                // A clause in the middle of a sentence starts small, unless it starts with a path.
                match letters.next() {
                    Some(first) if !sentence.starts_with(module) && !sentence.contains('/') => {
                        first.to_lowercase().chain(letters).collect()
                    }
                    _ => sentence.trim_end_matches('.').to_owned(),
                }
            }
        })
        .collect()
}

fn sentence_of(clauses: &[String]) -> String {
    let joined = match clauses {
        [] => return String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    };
    let mut letters = joined.chars();
    let first: String = letters
        .next()
        .map(|first| first.to_uppercase().collect())
        .unwrap_or_default();
    format!("{first}{}.", letters.as_str())
}

/// The alert at the top of the Summary of "All modules", with its severity: the
/// module with the most problems, in bold, and what is wrong with it.
/// `Checkout` and ` has 2 problems. It and Shipping use each other (6 files), and 3 of its
/// files read Billing\Models\Invoice directly.`
/// Without a module that has a problem, the first finding, with its own severity.
pub fn app_alert(
    app: &AppSnapshot,
    name: &str,
) -> Option<(mortise_core::Severity, String, String)> {
    use mortise_core::{FindingState, Severity};
    let worst = app
        .modules
        .iter()
        .filter_map(|module| match app.findings.state_of(&module.name) {
            FindingState::Problems(n) => Some((n, &module.name)),
            _ => None,
        })
        .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(a.1)));
    let Some((count, module)) = worst else {
        let first = app.findings.list.first()?;
        return Some((first.severity, String::new(), finding(first, app, name)));
    };
    let rest = format!(
        " has {}. {}",
        problems(count),
        sentence_of(&clauses(module, app, name, false))
    );
    Some((Severity::Problem, module.clone(), rest))
}

/// The alert at the top of the Summary of one module, with its severity.
/// `2 problems.` and ` Checkout and Shipping use each other (2 files here, 4 files there), and
/// 3 files read Modules\Billing\Models\Invoice directly.`
pub fn module_alert(
    app: &AppSnapshot,
    name: &str,
    module: &str,
) -> Option<(mortise_core::Severity, String, String)> {
    use mortise_core::{FindingState, Severity};
    match app.findings.state_of(module) {
        FindingState::Problems(n) => {
            let head = format!("{}.", problems(n));
            let rest = format!(" {}", sentence_of(&clauses(module, app, name, true)));
            Some((Severity::Problem, capitalised(&head), rest))
        }
        FindingState::Warning => {
            let sentences: Vec<String> = app
                .findings
                .of(module)
                .map(|found| finding(found, app, name))
                .collect();
            Some((Severity::Warning, String::new(), sentences.join(" ")))
        }
        FindingState::Good => None,
    }
}

/// Not in the spec: the line in the Summary of a module that is in a circle another
/// module carries.
pub fn part_of_circle_counted_on(module: &str) -> String {
    format!("Part of a circle counted on {module}")
}

pub(crate) fn capitalised(text: &str) -> String {
    let mut letters = text.chars();
    let first: String = letters
        .next()
        .map(|first| first.to_uppercase().collect())
        .unwrap_or_default();
    format!("{first}{}", letters.as_str())
}

/// The modifier of app shortcuts as a word: `Cmd` on macOS, `Ctrl` elsewhere.
pub fn command_key(mac: bool) -> &'static str {
    if mac { "Cmd" } else { "Ctrl" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::strings::files_turned_off;

    #[test]
    fn counts_have_a_comma_between_thousands() {
        assert_eq!(count(0), "0");
        assert_eq!(count(999), "999");
        assert_eq!(count(1000), "1,000");
        assert_eq!(count(41212), "41,212");
        assert_eq!(count(1234567), "1,234,567");
        assert_eq!(files(1), "1 file");
        assert_eq!(files(4912), "4,912 files");
        assert_eq!(files_turned_off(38), "38 files, turned off");
    }

    #[test]
    fn clauses_make_one_sentence() {
        assert_eq!(sentence_of(&[]), "");
        assert_eq!(
            sentence_of(&["it and Shipping use each other (6 files)".into()]),
            "It and Shipping use each other (6 files)."
        );
        assert_eq!(
            sentence_of(&[
                "it and Shipping use each other (6 files)".into(),
                "3 of its files read X directly".into()
            ]),
            "It and Shipping use each other (6 files), and 3 of its files read X directly."
        );
    }

    #[test]
    fn missing_translations_count_keys_and_files() {
        assert_eq!(
            missing_translations("fr", 1, 0, Some("en")),
            "1 key in en is missing in fr"
        );
        assert_eq!(
            missing_translations("de", 4, 1, None),
            "4 keys and 1 file are missing in de"
        );
        assert_eq!(
            missing_translations("fr", 0, 2, Some("en")),
            "2 files in en are missing in fr"
        );
    }

    #[test]
    fn time_ago_is_in_whole_units() {
        assert_eq!(ago(100, 92), "8 s ago");
        assert_eq!(ago(100, 100), "0 s ago");
        assert_eq!(ago(100, 200), "0 s ago");
        assert_eq!(ago(1_000, 1_000 - 60), "1 minute ago");
        assert_eq!(ago(10_000, 10_000 - 150), "2 minutes ago");
        assert_eq!(ago(100_000, 100_000 - 3_600), "1 hour ago");
        assert_eq!(ago(1_000_000, 1_000_000 - 47 * 3_600), "47 hours ago");
        assert_eq!(ago(1_000_000, 1_000_000 - 48 * 3_600), "2 days ago");
        assert_eq!(ago(100_000_000, 100_000_000 - 59 * 86_400), "59 days ago");
        assert_eq!(ago(100_000_000, 100_000_000 - 90 * 86_400), "3 months ago");
        assert_eq!(ago(200_000_000, 200_000_000 - 800 * 86_400), "2 years ago");
    }

    #[test]
    fn churn_says_how_many_and_when() {
        let day = 86_400;
        assert_eq!(
            churn(14, Some(100 * day - 2 * day), 100 * day),
            "14 commits in 30 days, last 2 days ago"
        );
        assert_eq!(
            churn(1, Some(100 * day - 3_600), 100 * day),
            "1 commit in 30 days, last 1 hour ago"
        );
        assert_eq!(
            churn(0, Some(10 * day), 100 * day),
            "no commits in 30 days, last 3 months ago"
        );
        assert_eq!(churn(0, None, 100 * day), "never committed");
    }

    #[test]
    fn a_problem_of_the_app_itself_is_alerted_as_a_problem() {
        use mortise_core::{Check, Detail, Finding, Severity};
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shop-api");
        let mut app = mortise_core::read_app(&fixture).unwrap();
        // No module has a finding; the statuses file lists a module that has no folder.
        app.findings.list = vec![Finding {
            severity: Severity::Problem,
            check: Check::NoStaleStatus,
            module: None,
            detail: Detail::StaleStatus {
                entry: "Legacy".to_owned(),
                file: "modules_statuses.json".to_owned(),
            },
        }];
        assert_eq!(
            app_alert(&app, &app.name),
            Some((
                Severity::Problem,
                String::new(),
                "modules_statuses.json lists Legacy, which has no folder.".to_owned()
            ))
        );
    }

    #[test]
    fn a_path_under_home_starts_with_a_tilde() {
        let home = std::env::home_dir().unwrap();
        assert_eq!(
            tilde(&home.join("code").join("shop-api")),
            "~/code/shop-api"
        );
        assert_eq!(tilde(&home), "~");
        assert_eq!(tilde(Path::new("/srv/app")), "/srv/app");
    }
}

// --- Runtime: what the running app reports through the actuator ---

pub const RUNTIME: &str = "Runtime";
pub const STATUS: &str = "Status";
pub const UP: &str = "Up";
pub const DOWN: &str = "Down";
pub const OUT_OF_SERVICE: &str = "Out of service";
pub const RUNTIME_UNKNOWN: &str = "Unknown";
pub const NO_ANSWER: &str = "No answer";
pub const NOT_AUTHORIZED: &str = "Not authorized";
pub const TRY_AGAIN: &str = "Try again";
pub const OPEN_SETTINGS: &str = "Open settings";
pub const ADD_ENVIRONMENT: &str = "Add environment";
pub const COPY_LINE: &str = "Copy line";
/// Not in the spec: what the toast says after "Copy line".
pub const COPIED_LINE: &str = "Copied the line.";
pub const REQUIRE_ACTUATOR: &str = "composer require nwidart/laravel-actuator";
pub const EXPOSURE_LINE: &str =
    "'exposure' => [\n    'include' => ['health', 'info', 'mappings', 'scheduledtasks'],\n],";
pub const ENVIRONMENT: &str = "Environment";
pub const PHP: &str = "PHP";
pub const MEMORY: &str = "Memory";
pub const ROUTES_BY_MODULE: &str = "Routes by module";
pub const ROUTES: &str = "Routes";
pub const NOT_IN_A_MODULE: &str = "Not in a module";
pub const SCHEDULED_TASKS: &str = "Scheduled tasks";
pub const COL_TASK: &str = "Task";
pub const COL_SCHEDULE: &str = "Schedule";
pub const COL_NEXT_RUN: &str = "Next run";
pub const COL_METHOD: &str = "Method";
pub const COL_URI: &str = "URI";
pub const COL_NAME: &str = "Name";
pub const COL_ACTION: &str = "Action";
pub const COL_MIDDLEWARE: &str = "Middleware";
pub const NO_FIXED_TIME: &str = "no fixed time";
pub const NONE: &str = "none";
/// Not in the spec: the status line before the first answer is there.
pub const ASKING: &str = "asking";

/// `Runtime on local`
pub fn runtime_on(environment: &str) -> String {
    format!("Runtime on {environment}")
}

/// `on local`
pub fn on_environment(environment: &str) -> String {
    format!("on {environment}")
}

/// `checked 8 s ago`
pub fn checked_ago(seconds: u64) -> String {
    match seconds {
        0..60 => format!("checked {seconds} s ago"),
        60..3_600 => format!("checked {} min ago", seconds / 60),
        _ => format!("checked {} h ago", seconds / 3_600),
    }
}

/// `Up on local, checked 8 s ago`
pub fn runtime_tooltip(word: &str, environment: &str, seconds: Option<u64>) -> String {
    match seconds {
        Some(seconds) => format!("{word} on {environment}, {}", checked_ago(seconds)),
        None => format!("{word} on {environment}"),
    }
}

pub fn actuator_not_installed(app: &str) -> String {
    format!(
        "Runtime status needs nwidart/laravel-actuator. {app} has no entry for it in \
         composer.lock. Run composer require nwidart/laravel-actuator in the app."
    )
}

pub fn no_environment(app: &str) -> String {
    format!(
        "No environment is connected. Add the URL of a running {app} to read its runtime status."
    )
}

pub fn cannot_reach(environment: &str, url: &str, reason: &str) -> String {
    format!(
        "Cannot reach {environment}. No answer from {url}: {reason}. Start the app, or correct \
         the URL in Settings."
    )
}

/// `info, mappings and scheduledtasks`
pub fn and_list(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [first @ .., last] => format!("{} and {last}", first.join(", ")),
    }
}

pub fn not_exposed(endpoints: &[&str], environment: &str) -> String {
    let (verb, them) = if endpoints.len() == 1 {
        ("is", "it")
    } else {
        ("are", "them")
    };
    format!(
        "{} {verb} not exposed on {environment}. Only health is exposed by default. Add {them} \
         to exposure.include in config/actuator.php.",
        and_list(endpoints)
    )
}

/// The short form, in the place of routes or scheduled tasks.
pub fn endpoint_not_exposed(endpoint: &str, environment: &str) -> String {
    format!(
        "{endpoint} is not exposed on {environment}. Add it to exposure.include in \
         config/actuator.php."
    )
}

pub fn refused_credentials(environment: &str, url: &str, status: u16) -> String {
    format!(
        "{environment} refused the credentials. {url} answered {status}. Replace the credentials \
         for {environment} in Settings."
    )
}

pub const COMPONENTS_HIDDEN: &str = "Components are hidden. The app shows them only when \
    health.show_details allows it, and that is never by default. Set it to when_authorized or \
    always in config/actuator.php.";

pub fn out_of_service(component: &str) -> String {
    if component == "readinessState" {
        format!(
            "{component} is out of service. The app reports this while it is in maintenance mode."
        )
    } else {
        format!("{component} is out of service.")
    }
}

/// The path of an address: `/actuator/health`
fn path_of(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    rest.find('/').map_or("/", |at| &rest[at..])
}

/// What a test in the environment form gave, when the app answered with a status.
/// `http://shop.test/actuator answered 301 and sends to https://shop.test/actuator.`
pub fn tested(url: &str, status: u16, location: Option<&str>) -> String {
    match location {
        // Not in the spec: a redirect, which Mortise does not follow.
        Some(location) => format!("{url} answered {status} and sends to {location}."),
        None => format!("{url} answered {status}."),
    }
}

pub fn answered_unexpected(
    environment: &str,
    url: &str,
    status: u16,
    location: Option<&str>,
) -> String {
    let path = path_of(url);
    if let Some(location) = location {
        // Not in the spec: a redirect, which Mortise does not follow.
        return format!(
            "{environment} answered {status} for {path} and sends to {location}. Correct the \
             base URL or the actuator path in Settings to match that address."
        );
    }
    match status {
        500..600 => format!(
            "{environment} answered {status} for {path}. The actuator configuration has an \
             error. Read the app log."
        ),
        // Not in the spec: an address where no actuator answers.
        _ => format!(
            "{environment} answered {status} for {path}. Correct the base URL or the actuator \
             path in Settings."
        ),
    }
}

/// `4 up`
pub fn n_up(n: usize) -> String {
    format!("{n} up")
}

/// `4 other components`
pub fn other_components(n: usize) -> String {
    match n {
        1 => "1 other component".to_owned(),
        n => format!("{n} other components"),
    }
}

/// `198.6 GB free of 460.4 GB`
pub fn disk(free: f64, total: f64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    format!("{:.1} GB free of {:.1} GB", free / GB, total / GB)
}

/// `38 MB now, 41 MB peak, limit 512 MB`
pub fn memory(usage: Option<i64>, peak: Option<i64>, limit: Option<i64>) -> String {
    let mb = |bytes: i64| format!("{} MB", bytes / (1024 * 1024));
    let mut parts = Vec::new();
    if let Some(usage) = usage {
        parts.push(format!("{} now", mb(usage)));
    }
    if let Some(peak) = peak {
        parts.push(format!("{} peak", mb(peak)));
    }
    match limit {
        Some(limit) if limit > 0 => parts.push(format!("limit {}", mb(limit))),
        // PHP writes -1 for a process without a limit.
        Some(_) => parts.push("no limit".to_owned()),
        None => {}
    }
    parts.join(", ")
}

/// `106 routes, read from local`
pub fn routes_read_from(n: usize, environment: &str) -> String {
    match n {
        1 => format!("1 route, read from {environment}"),
        n => format!("{} routes, read from {environment}", count(n as u64)),
    }
}

/// `5 tasks, read from local`
pub fn tasks_read_from(n: usize, environment: &str) -> String {
    match n {
        1 => format!("1 task, read from {environment}"),
        n => format!("{n} tasks, read from {environment}"),
    }
}

/// `1 task`
pub fn tasks_count(n: usize) -> String {
    match n {
        1 => "1 task".to_owned(),
        n => format!("{n} tasks"),
    }
}

/// `18 (12 API, 6 web)`
pub fn routes_of_module(total: usize, api: usize, web: usize) -> String {
    let mut parts = Vec::new();
    if api > 0 {
        parts.push(format!("{api} API"));
    }
    if web > 0 {
        parts.push(format!("{web} web"));
    }
    let other = total - api - web;
    if other > 0 && !parts.is_empty() {
        parts.push(format!("{other} other"));
    }
    if parts.is_empty() {
        total.to_string()
    } else {
        format!("{total} ({})", parts.join(", "))
    }
}

/// `every 10 s`
pub fn every(interval_ms: u64) -> String {
    if interval_ms.is_multiple_of(1000) {
        format!("every {} s", interval_ms / 1000)
    } else {
        format!("every {interval_ms} ms")
    }
}

/// Days since 1970 of a date in the calendar.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// When a scheduled task runs next: `14:10 UTC`, `tomorrow 06:00 UTC`, or the date.
/// `next` is what the app wrote, ISO 8601 with an offset: `2026-11-23T06:00:00+00:00`.
pub fn next_run(next: Option<&str>, timezone: Option<&str>, now: i64) -> String {
    let Some(next) = next else {
        return NO_FIXED_TIME.to_owned();
    };
    let parsed = (|| {
        let number = |from: usize, to: usize| next.get(from..to)?.parse::<i64>().ok();
        let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
        let (hour, minute) = (number(11, 13)?, number(14, 16)?);
        // The offset of the task's timezone, in seconds.
        let offset = match next.get(19..) {
            Some("Z") | Some("") | None => 0,
            Some(offset) => {
                let sign = if offset.starts_with('-') { -1 } else { 1 };
                let hours = offset.get(1..3)?.parse::<i64>().ok()?;
                let minutes = offset
                    .get(4..6)
                    .and_then(|m| m.parse::<i64>().ok())
                    .unwrap_or(0);
                sign * (hours * 3600 + minutes * 60)
            }
        };
        Some((year, month, day, hour, minute, offset))
    })();
    let Some((year, month, day, hour, minute, offset)) = parsed else {
        // As the app wrote it.
        return next.to_owned();
    };
    let zone = match (timezone, offset) {
        (Some(zone), _) => zone.to_owned(),
        (None, 0) => "UTC".to_owned(),
        (None, _) => next.get(19..).unwrap_or_default().to_owned(),
    };
    let time = format!("{hour:02}:{minute:02} {zone}");
    // Today, in the timezone of the task.
    let today = (now + offset).div_euclid(86_400);
    match days_from_civil(year, month, day) - today {
        0 => time,
        1 => format!("tomorrow {time}"),
        _ => format!("{year:04}-{month:02}-{day:02} {time}"),
    }
}

// --- Environments and their credentials ---

pub const CREDENTIALS: &str = "Credentials";
pub const CREDENTIALS_HINT: &str = "what the app's middleware asks for";
pub const CREDENTIAL_KINDS: [&str; 4] = ["None", "Bearer token", "HTTP basic", "Custom header"];
pub const NO_CREDENTIALS: &str =
    "Mortise sends no credentials. Right for a local app whose actuator is open.";
pub const TOKEN: &str = "Token";
pub const TOKEN_HINT: &str = "sent as Authorization: Bearer";
pub const USER_NAME: &str = "User name";
pub const PASSWORD: &str = "Password";
pub const HEADER_NAME: &str = "Header name";
pub const HEADER_VALUE: &str = "Header value";
pub const KEPT_IN_STORE: &str =
    "Kept in the credential store of this computer, not in mortise.json and not in the app.";
/// Not in the spec as a sentence. The spec says that Mortise says so and keeps the
/// secret only until it closes.
pub const NO_STORE: &str =
    "No credential store is running on this computer. The secret is kept until Mortise closes.";
pub const SAVED_NOT_SHOWN: &str = "Saved. Not shown again.";
pub const REPLACE: &str = "Replace";
pub const REMOVE: &str = "Remove";
pub const TEST: &str = "Test";
pub const BASE_URL: &str = "Base URL";
pub const BASE_URL_HINT: &str = "from APP_URL in .env";
pub const ACTUATOR_PATH: &str = "Actuator path";
pub const COL_ENVIRONMENT: &str = "Environment";
pub const COL_PATH: &str = "Path";
pub const COL_LAST_ANSWER: &str = "Last answer";
/// Not in the spec: an environment that was not asked in this run.
pub const NOT_ASKED: &str = "not asked yet";
/// Not in the spec: the way to drop one environment.
pub const REMOVE_ENVIRONMENT: &str = "Remove environment";

/// `Environments of shop-api`
pub fn environments_of(app: &str) -> String {
    format!("Environments of {app}")
}

/// `Bearer token, saved`
pub fn credentials_saved(kind: &str, saved: bool) -> String {
    if saved {
        format!("{kind}, saved")
    } else {
        kind.to_owned()
    }
}

/// `Exposed: health, info, mappings, scheduledtasks.`
pub fn exposed(endpoints: &[String]) -> String {
    format!("Exposed: {}.", endpoints.join(", "))
}

// --- How PHP runs (S1.3, S8.2) ---

/// The choices of "Run PHP with", in the order of [`runner_place`].
pub const RUNNERS: [&str; 4] = ["Herd", "Sail", "Docker", "Custom"];
pub const COL_APP: &str = "App";
pub const COL_RUNS_WITH: &str = "Runs with";
pub const COL_COMMAND: &str = "Command";
pub const COL_LAST_TEST: &str = "Last test";
/// Not in the spec: an app whose runner is not chosen yet.
pub const NOT_CHOSEN: &str = "Not chosen";
/// Not in the spec: an app that was not tested since Mortise started (design D8).
pub const NOT_TESTED: &str = "Not tested";
pub const WORKING_FOLDER: &str = "Working folder";
pub const WORKING_FOLDER_HINT: &str = "where the command starts";
/// Not in the spec: the empty working folder. The spec draws the folder itself.
pub const THE_APP_FOLDER: &str = "The app folder";
pub const STOP_AFTER: &str = "Stop a command after";
pub const STOP_AFTER_HINT: &str = "so a command that waits for input cannot hang";
pub const WHAT_EACH_RUNS: &str = "What each choice runs";
pub const WHAT_EACH_RUNS_NOTE: &str = "the part before the command name";
pub const CUSTOM_RUNS: &str = "what you type, for example /usr/bin/php artisan";

/// The place of a runner in [`RUNNERS`].
pub fn runner_place(runner: &mortise_core::RunnerKind) -> usize {
    use mortise_core::RunnerKind;
    match runner {
        RunnerKind::Herd => 0,
        RunnerKind::Sail => 1,
        RunnerKind::Docker { .. } => 2,
        RunnerKind::Custom { .. } => 3,
    }
}

/// `10 minutes`
pub fn minutes(n: u16) -> String {
    match n {
        1 => "1 minute".to_owned(),
        n => format!("{n} minutes"),
    }
}

/// The reason in quotes of `x.nophp`, as design D12 gives it. `herd: command not found`
/// is the spec's. The others are not in the spec.
pub fn php_reason(failure: &mortise_core::Failure) -> String {
    use mortise_core::Failure;
    match failure {
        Failure::NotFound { program } => format!("{program}: command not found"),
        Failure::NotStarted { program, message } => format!("{program}: {message}"),
        Failure::Exited { line, .. } if !line.is_empty() => line.clone(),
        Failure::Exited {
            code: Some(code), ..
        } => format!("exit code {code}"),
        Failure::Exited { code: None, .. } => "stopped by a signal".to_owned(),
        Failure::TimedOut { seconds } => format!("no answer in {seconds} seconds"),
        Failure::NoCommand => "no command entered".to_owned(),
        Failure::NoFolder { .. } => "the working folder does not exist".to_owned(),
    }
}

/// `No answer: service "app" is not running`, in the "Last test" column of S8.2.
pub fn no_answer(failure: &mortise_core::Failure) -> String {
    format!("No answer: {}", php_reason(failure))
}

#[cfg(test)]
mod runtime_tests {
    use super::*;

    #[test]
    fn the_next_run_is_said_relative_to_today_in_the_zone_of_the_task() {
        // 2026-11-22 13:00:00 UTC
        let now = 1_795_352_400;
        assert_eq!(
            next_run(Some("2026-11-22T14:10:00+00:00"), Some("UTC"), now),
            "14:10 UTC"
        );
        assert_eq!(
            next_run(Some("2026-11-23T06:00:00+00:00"), Some("UTC"), now),
            "tomorrow 06:00 UTC"
        );
        assert_eq!(
            next_run(Some("2026-12-01T03:00:00+00:00"), Some("UTC"), now),
            "2026-12-01 03:00 UTC"
        );
        // 23:30 in Brussels on the 22nd is still today there.
        assert_eq!(
            next_run(
                Some("2026-11-22T23:30:00+01:00"),
                Some("Europe/Brussels"),
                now
            ),
            "23:30 Europe/Brussels"
        );
        // At 23:30 UTC it is already the 23rd in Brussels.
        assert_eq!(
            next_run(
                Some("2026-11-23T06:00:00+01:00"),
                None,
                now + 10 * 3600 + 1800
            ),
            "06:00 +01:00"
        );
        assert_eq!(next_run(None, None, now), "no fixed time");
        assert_eq!(next_run(Some("soon"), None, now), "soon");
    }

    #[test]
    fn figures_are_written_as_in_the_drawings() {
        assert_eq!(
            disk(213_212_008_448.0, 494_384_795_648.0),
            "198.6 GB free of 460.4 GB"
        );
        assert_eq!(
            memory(Some(39_845_888), Some(42_991_616), Some(536_870_912)),
            "38 MB now, 41 MB peak, limit 512 MB"
        );
        assert_eq!(
            memory(Some(39_845_888), None, Some(-1)),
            "38 MB now, no limit"
        );
        assert_eq!(every(10_000), "every 10 s");
        assert_eq!(routes_of_module(18, 12, 6), "18 (12 API, 6 web)");
        assert_eq!(routes_of_module(3, 0, 0), "3");
        assert_eq!(routes_of_module(5, 2, 0), "5 (2 API, 3 other)");
        assert_eq!(
            and_list(&["info", "mappings", "scheduledtasks"]),
            "info, mappings and scheduledtasks"
        );
        assert_eq!(
            not_exposed(&["mappings"], "local"),
            "mappings is not exposed on local. Only health is exposed by default. Add it to \
             exposure.include in config/actuator.php."
        );
        assert_eq!(
            answered_unexpected("local", "https://shop-api.test/actuator/health", 500, None),
            "local answered 500 for /actuator/health. The actuator configuration has an error. \
             Read the app log."
        );
        assert_eq!(
            answered_unexpected(
                "local",
                "http://shop-api.test/actuator",
                301,
                Some("https://shop-api.test/actuator")
            ),
            "local answered 301 for /actuator and sends to https://shop-api.test/actuator. \
             Correct the base URL or the actuator path in Settings to match that address."
        );
    }
}

// --- The public surface of a module ---

pub const PUBLIC_SURFACE: &str = "Public surface";

/// `Reading 61 of 96 files`
pub fn reading_files(read: usize, total: usize) -> String {
    format!("Reading {read} of {}", files(total as u64))
}

// --- Runner drawer (S11.1) ---

/// The keys that show and hide the drawer, on every platform (section 7).
pub const TOGGLE_RUNNER_KEYS: &str = "Ctrl `";
/// Not in the spec: the reason of a start without a runner (design D1).
pub const NO_RUNNER_CHOSEN: &str = "no way to run PHP is chosen";

/// Not in the spec: the sentence under a failed run (design D9). The sentence that
/// names a migration comes with the actions.
pub fn stopped_with(label: &str, code: Option<i32>) -> String {
    match code {
        Some(code) => format!("{label} stopped with exit code {code}."),
        None => format!("{label} was ended by a signal."),
    }
}

/// `module:migrate --all is running`, the tooltip of a run button that is off.
pub fn is_running(label: &str) -> String {
    format!("{label} is running")
}

/// The reason in quotes of a command that did not start.
pub fn start_reason(error: &mortise_core::StartError) -> String {
    use mortise_core::StartError;
    match error {
        StartError::Busy { command } => is_running(command),
        StartError::NoRunner => NO_RUNNER_CHOSEN.to_owned(),
        StartError::Failed(failure) => php_reason(failure),
    }
}

pub const DRY_RUN: &str = "Dry run";
pub const SHOW_OUTPUT: &str = "Show output";

// ---------------------------------------------------------------------------
// Actions (S10.1 to S10.3)

pub const COMMAND: &str = "Command";

/// Not in the spec: a run button is off while its line was not whole on screen,
/// because a command runs only with its exact line on screen first.
pub const SCROLL_TO_COMMAND: &str = "Scroll up to see the whole command first.";

/// Not in the spec: a run button is off because no way to run PHP is chosen (design D4).
pub fn choose_php(app: &str) -> String {
    format!("Choose how PHP runs for {app} in Settings.")
}

/// The name of an action in the list. "All modules" names its rows in full (S10.2 f).
pub fn action_row(action: mortise_core::Action, all: bool) -> &'static str {
    use mortise_core::Action;
    match (action, all) {
        (Action::Migrate, true) => "Migrate all modules",
        (Action::Seed, true) => "Seed all modules",
        (Action::RunTests, true) => "Run all module tests",
        (action, _) => action_button(action),
    }
}

/// The label of the button that runs an action. The Migrate button carries no count
/// without the migration state (deviation S41).
pub fn action_button(action: mortise_core::Action) -> &'static str {
    use mortise_core::Action;
    match action {
        Action::TurnOn => "Turn on",
        Action::TurnOff => "Turn off",
        Action::Migrate => "Migrate",
        Action::RollBack => "Roll back",
        Action::Seed => "Seed",
        Action::RunTests => "Run tests",
        Action::CheckTranslations => "Check translations",
        Action::ResetMigrations => "Reset migrations",
        Action::RefreshMigrations => "Refresh migrations",
        Action::FreshMigrate => "Drop all tables and migrate",
        Action::DeleteModule => "Delete module",
    }
}

/// Not in the spec: the failed toast of a run that was stopped.
pub fn was_stopped(label: &str) -> String {
    format!("{label} was stopped.")
}

// ---------------------------------------------------------------------------
// The migration state (P4, design D7)

/// Not in the spec: the state box when nothing waits, and the row detail of Migrate.
pub const NOTHING_WAITING: &str = "Nothing waiting";

/// The row detail of Migrate: `1 waiting` (S10.1).
pub fn waiting(n: usize) -> String {
    match n {
        0 => NOTHING_WAITING.to_owned(),
        n => format!("{} waiting", count(n as u64)),
    }
}

/// The row detail of Reset and Refresh, and the Migrations line of S4.3: `9 ran`.
pub fn ran(n: usize) -> String {
    format!("{} ran", count(n as u64))
}

/// `Migrate (1)`, the run button of Migrate and of the header of S4.3 with waiting
/// migrations (S10.1, S4.3).
pub fn migrate_counted(waiting: usize) -> String {
    match waiting {
        0 => action_button(mortise_core::Action::Migrate).to_owned(),
        n => format!("Migrate ({})", count(n as u64)),
    }
}

// ---------------------------------------------------------------------------
// Destructive actions (S10.4, S10.5)

/// `Type Checkout to confirm`
pub fn type_to_confirm(name: &str) -> String {
    format!("Type {name} to confirm")
}

// ---------------------------------------------------------------------------
// Generate (S9.1 to S9.6)

pub const NEW_MODULE: &str = "New module";

/// Not in the spec: the reason in quotes of a read that gave no document (design D2):
/// serde's message for a malformed one, [`ANSWER_CUT`] for a cut one, else the last
/// line of standard error, else how the read ended.
pub fn no_document(end: &mortise_core::ReadEnd, error: &mortise_core::DataError) -> String {
    use mortise_core::DataError;
    match error {
        DataError::Malformed(message) => message.clone(),
        DataError::Cut => ANSWER_CUT.to_owned(),
        DataError::Missing => end.error_line().unwrap_or_else(|| ended_as(end.end)),
    }
}

/// Not in the spec: the reason of a document that was too long to keep whole (design D2).
pub const ANSWER_CUT: &str =
    "the answer was cut: Mortise keeps 64 KiB of a line and 1 MB of output";

/// The label of a generator in the picker (S9.1). A generator the spec does not
/// name shows its artisan name without `module:` (not in the spec).
pub fn generator_label(name: &str) -> String {
    let label = match name {
        mortise_core::MODULE_MAKE => NEW_MODULE,
        "module:make-model" => "Model",
        "module:make-migration" => "Migration",
        "module:make-factory" => "Factory",
        "module:make-seed" => "Seeder",
        "module:make-observer" => "Observer",
        "module:make-scope" => "Scope",
        "module:make-cast" => "Cast",
        "module:make-policy" => "Policy",
        "module:make-repository" => "Repository",
        "module:make-controller" => "Controller",
        "module:make-request" => "Request",
        "module:make-resource" => "Resource",
        "module:make-middleware" => "Middleware",
        "module:make-rule" => "Rule",
        "module:route-provider" => "Route provider",
        "module:make-event" => "Event",
        "module:make-listener" => "Listener",
        "module:make-job" => "Job",
        "module:make-job-middleware" => "Job middleware",
        "module:make-notification" => "Notification",
        "module:make-mail" => "Mail",
        "module:make-channel" => "Channel",
        "module:make-event-provider" => "Event provider",
        "module:make-view" => "View",
        "module:make-component" => "Component class",
        "module:make-component-view" => "Component view",
        "module:make-inertia-page" => "Inertia page",
        "module:make-inertia-component" => "Inertia component",
        "module:make-class" => "Class",
        "module:make-health-indicator" => "Health indicator",
        "module:make-action" => "Action",
        "module:make-service" => "Service",
        "module:make-interface" => "Interface",
        "module:make-trait" => "Trait",
        "module:make-enum" => "Enum",
        "module:make-helper" => "Helper",
        "module:make-exception" => "Exception",
        "module:make-command" => "Command",
        "module:make-config" => "Config",
        "module:make-provider" => "Provider",
        "module:make-replacement" => "Stub replacement",
        "module:make-test" => "Test",
        other => return other.strip_prefix("module:").unwrap_or(other).to_owned(),
    };
    label.to_owned()
}

/// The label of a generator inside a sentence: `model`, `Inertia page`.
pub(crate) fn generator_noun(name: &str) -> String {
    let label = generator_label(name);
    if label.starts_with("Inertia") {
        return label;
    }
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => label,
    }
}

/// `a model`, `an event`.
pub(crate) fn with_article(noun: &str) -> String {
    let vowel = noun
        .chars()
        .next()
        .is_some_and(|c| "aeiouAEIOU".contains(c));
    if vowel {
        format!("an {noun}")
    } else {
        format!("a {noun}")
    }
}

/// Not in the spec: why a run gave no document. serde's message for a malformed one,
/// else how the run ended.
pub fn no_run_document(error: &mortise_core::DataError, end: mortise_core::RunEnd) -> String {
    use mortise_core::{DataError, RunEnd};
    match (error, end) {
        (DataError::Malformed(message), _) => message.clone(),
        (DataError::Cut, _) => ANSWER_CUT.to_owned(),
        (DataError::Missing, RunEnd::Exited { code: Some(0), .. }) => {
            "nothing on standard output".to_owned()
        }
        (DataError::Missing, end) => ended_as(end),
    }
}

/// How a command ended, in a reason: `exit code 1`.
fn ended_as(end: mortise_core::RunEnd) -> String {
    use mortise_core::RunEnd;
    match end {
        RunEnd::Exited {
            code: Some(code), ..
        } => format!("exit code {code}"),
        RunEnd::Exited { code: None, .. } => "ended by a signal".to_owned(),
        RunEnd::Stopped { limit: Some(_), .. } => "stopped at the time limit".to_owned(),
        RunEnd::Stopped { limit: None, .. } => "stopped".to_owned(),
    }
}
