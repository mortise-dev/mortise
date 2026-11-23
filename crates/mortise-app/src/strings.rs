//! Interface text, from section 10 of the specification.
//!
//! Sentence case, no exclamation marks, no "please". An error says what happened,
//! then what to do next. Text that is not in section 10 is marked "not in the spec".

use std::path::Path;

use mortise_core::{AppSnapshot, LayoutKey, ReadError, Version};

pub const ALL_MODULES: &str = "All modules";

// Actions (S10.1, S10.2)
/// The hint of an action row of the palette.
pub const ACTION: &str = "Action";
/// Not in the spec: the group of the palette when "All modules" is selected.
pub const ACTIONS_FOR_ALL: &str = "Actions for all modules";
pub fn actions_for(module: &str) -> String {
    format!("Actions for {module}")
}
/// The row of an action in the palette: "Migrate Checkout", or the S10.2 f name for all modules.
pub fn action_label(action: mortise_core::Action, module: Option<&str>) -> String {
    match module {
        Some(module) => match action {
            mortise_core::Action::RunTests => format!("Run tests of {module}"),
            mortise_core::Action::CheckTranslations => format!("Check translations in {module}"),
            action => format!("{} {module}", action_button(action)),
        },
        None => action_row(action, true).to_owned(),
    }
}
pub const SEARCH: &str = "Search or run a command";
pub const SORTED_BY_NAME: &str = "Sorted: by name";
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

/// `38 files, turned off`
pub fn files_turned_off(n: u64) -> String {
    format!("{}, turned off", files(n))
}

/// `7 of 7 on`
pub fn of_on(on: usize, total: usize) -> String {
    format!("{on} of {total} on")
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

pub const FILTER_MODULES: &str = "Filter modules";
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

/// `84 files, 54% covered`
pub fn files_covered(n: u64, percent: Option<u8>) -> String {
    match percent {
        Some(percent) => format!("{}, {percent}% covered", files(n)),
        None => files(n),
    }
}

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

/// `1 of 7 modules`
pub fn of_modules(shown: usize, total: usize) -> String {
    format!("{shown} of {total} modules")
}

/// `No module matches "paym". Press Esc to clear the filter.`
pub fn no_module_matches(filter: &str) -> String {
    format!("No module matches \"{filter}\". Press Esc to clear the filter.")
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

/// The age of a recent action in the palette, in seconds since 1970: `ran 2 minutes ago`
/// (S12.2 a). The other units, such as `ran 8 s ago`, are not in the spec.
pub fn ran_ago(now: i64, then: i64) -> String {
    format!("ran {}", ago(now, then))
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
pub const SELECTED: &str = "Selected";
pub const ALL_LINKS: &str = "All links";
pub const PROBLEMS_ONLY: &str = "Problems only";
pub const LINKS_THAT_BREAK: &str = "Links that break a rule";
pub const RULE_NO_CIRCULAR: &str = "No circular use";
pub const RULE_PUBLIC_SURFACE: &str = "Other modules use only a module's public surface";
pub const RULE_FAIL_CI: &str = "Fail CI when a rule breaks";
pub const RULE_FAIL_CI_HINT: &str = "Needs the command line tool, which does not exist yet.";
pub const RULES_KEPT: &str = "kept in mortise.json in the app folder";
pub const GRAPH_CAPTION: &str = "Arrow points at the module that is used. A square with a cross marks a link that breaks a rule.";
pub const COPY_PATH: &str = "Copy path";
/// What a screen reader says for "Open in editor" under the files of a link.
pub const OPEN_FILE_IN_EDITOR: &str = "Open the selected file in the editor";

pub const COL_FILE: &str = "File";
pub const COL_LINE: &str = "Line";
pub const COL_READS: &str = "Reads";
pub const COL_FROM: &str = "From";
pub const COL_USES: &str = "Uses";
pub const COL_USED_BY: &str = "Used by";
pub const COL_REFS: &str = "Refs";
pub const COL_STATE: &str = "State";
pub const GOOD: &str = "Good";
pub const CIRCULAR: &str = "Circular";
pub const FILTER_LINKS: &str = "Filter links";
/// The name of the folders field of the second rule, for a screen reader. Not in the spec.
pub const PUBLIC_FOLDERS: &str = "Folders that count as public";

/// `Used by, 2`
pub fn used_by_count(n: usize) -> String {
    format!("Used by, {n}")
}

/// `Uses, 4`
pub fn uses_count(n: usize) -> String {
    format!("Uses, {n}")
}

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

/// `Checkout reads Billing internals`
pub fn reads_internals(a: &str, b: &str) -> String {
    format!("{a} reads {b} internals")
}

/// `3 of 6 files read internals`
pub fn files_read_internals(internal: usize, files_count: usize) -> String {
    format!("{internal} of {} read internals", files(files_count as u64))
}

/// `3 files read internals`
pub fn n_files_read_internals(internal: usize) -> String {
    if internal == 1 {
        "1 file reads internals".to_owned()
    } else {
        format!("{internal} files read internals")
    }
}

/// `Circular, 6 files`
pub fn circular_files(files_count: usize) -> String {
    format!("Circular, {}", files(files_count as u64))
}

/// `9 links between 7 modules`
pub fn links_between(links: usize, modules_count: usize) -> String {
    let links = if links == 1 {
        "1 link".to_owned()
    } else {
        format!("{links} links")
    };
    format!("{links} between {}", modules(modules_count))
}

/// `2 problems`
pub fn problems(n: usize) -> String {
    if n == 1 {
        "1 problem".to_owned()
    } else {
        format!("{n} problems")
    }
}

/// `214 links, 5 problems, 14 shown`
pub fn links_shown(links: usize, problem_count: usize, shown: usize) -> String {
    let links = if links == 1 {
        "1 link".to_owned()
    } else {
        format!("{links} links")
    };
    format!("{links}, {}, {shown} shown", problems(problem_count))
}

/// `12 more`
pub fn more(n: usize) -> String {
    format!("{n} more")
}

/// `Rules for shop-api`
pub fn rules_for(app: &str) -> String {
    format!("Rules for {app}")
}

/// `Accepted for 0 of 7 modules. For a module without one, these folders count:`
pub fn accepted_for(accepted: usize, total: usize) -> String {
    format!(
        "Accepted for {accepted} of {}. For a module without one, these folders count:",
        modules(total)
    )
}

/// `Rule: other modules use only a module's public surface. Billing has none yet, so the
/// folders Contracts and Events count.`
pub fn rule_of_link(module: &str, rules: &mortise_core::Rules) -> String {
    public_surface_rule(module, "has", rules)
}

/// Not in the spec: the rule under the files of two modules that use each other while
/// the rule "No circular use" is off.
/// `Rule: other modules use only a module's public surface. Checkout and Shipping have
/// none yet, so the folders Contracts and Events count.`
pub fn rule_of_links(a: &str, b: &str, rules: &mortise_core::Rules) -> String {
    public_surface_rule(&format!("{a} and {b}"), "have", rules)
}

fn public_surface_rule(modules: &str, has: &str, rules: &mortise_core::Rules) -> String {
    if !rules.public_surface_only {
        // Not in the spec: the rule is turned off.
        return "The rule about a module's public surface is turned off for this app.".to_owned();
    }
    let folders = match rules.public_folders.as_slice() {
        [] => "no folders count".to_owned(),
        [one] => format!("the folder {one} counts"),
        [rest @ .., last] => format!("the folders {} and {last} count", rest.join(", ")),
    };
    format!(
        "Rule: other modules use only a module's public surface. {modules} {has} none yet, so {folders}."
    )
}

/// `Reported on Checkout, the side with fewer files to change.`
pub fn reported_on(module: &str) -> String {
    format!(
        "Reported on {module}, the side with fewer files to change. Every file in a circle is \
         marked: removing either direction ends it."
    )
}

/// Not in the spec: the footer of a link in a circle of three or more modules,
/// where removing one direction need not end the circle.
pub fn reported_on_in_circle(module: &str) -> String {
    format!(
        "Reported on {module}, the module of the circle with the fewest files to change. Every \
         file of a link in a circle is marked."
    )
}

/// `platform has 60 modules. Above 12 modules the graph of the whole app gives way to this table.`
pub fn many_modules(app: &str, modules_count: usize) -> String {
    format!(
        "{app} has {modules_count} modules. Above 12 modules the graph of the whole app gives way \
         to this table. Select one module for its own graph."
    )
}

/// Not in the spec: an app whose modules do not use each other.
pub fn no_links(app: &str) -> String {
    format!("No module of {app} uses another.")
}

/// Not in the spec: a module that uses no other module and is used by none.
pub fn no_links_of(module: &str) -> String {
    format!("{module} uses no other module, and no other module uses it.")
}

/// Not in the spec: mortise.json is there but cannot be read as rules.
pub fn rules_invalid(message: &str) -> String {
    format!("mortise.json is not valid: {message}. The default rules apply until it is corrected.")
}

pub const FINDINGS: &str = "Findings";
pub const SORTED_PROBLEMS_FIRST: &str = "Sorted: problems first";
pub const CHANGE_THE_ORDER: &str = "Change the order";
pub const NEEDS_ATTENTION: &str = "Needs attention";
pub const COL_FINDING: &str = "Finding";
pub const COL_CHECK: &str = "Check";
pub const COL_ENTER_OPENS: &str = "Enter opens";
pub const COL_FINDINGS: &str = "Findings";
pub const PROBLEMS: &str = "Problems";
pub const WARNINGS: &str = "Warnings";
pub const CHECKS_WITHOUT_A_FINDING: &str = "Checks without a finding";
pub const LOW_COVERAGE: &str = "Low coverage";
pub const WARNING: &str = "Warning";
pub const SUMMARY: &str = "Summary";
pub const BOUNDARY: &str = "Boundary";
pub const ACTIONS: &str = "Actions";
pub const TODO_NOT_COUNTED: &str = "To do, not counted as findings";

/// `Checkout has 1 migration waiting.`: a row of the To do group of Findings.
pub fn migrations_waiting(module: &str, n: usize) -> String {
    if n == 1 {
        format!("{module} has 1 migration waiting.")
    } else {
        format!("{module} has {n} migrations waiting.")
    }
}

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

/// `Modules in shop-api`: the group of modules in the palette.
pub fn modules_in(app: &str) -> String {
    format!("Modules in {app}")
}

/// `Show all 3 findings`
pub fn show_all_findings(n: usize) -> String {
    if n == 1 {
        "Show the finding".to_owned()
    } else {
        format!("Show all {n} findings")
    }
}

/// `4 of 7 checks`
pub fn of_checks(quiet: usize, total: usize) -> String {
    format!("{quiet} of {total} checks")
}

/// `No findings. 7 modules checked against 2 rules and 4 checks.`
pub fn no_findings(modules_count: usize, rules: usize, checks: usize) -> String {
    let rules = if rules == 1 {
        "1 rule".to_owned()
    } else {
        format!("{rules} rules")
    };
    let checks = if checks == 1 {
        "1 check".to_owned()
    } else {
        format!("{checks} checks")
    };
    format!(
        "No findings. {} checked against {rules} and {checks}.",
        modules(modules_count)
    )
}

fn list_of(names: &[String]) -> String {
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

/// The name of a check in the Findings list: `Rule: no circular use`.
pub fn check_name(check: mortise_core::Check, app: &AppSnapshot) -> String {
    use mortise_core::Check;
    match check {
        Check::NoCircularUse => "Rule: no circular use".to_owned(),
        Check::PublicSurfaceOnly => format!("Rule: {} only", list_of(&app.rules.public_folders)),
        Check::CoverageLimit => "Coverage limit".to_owned(),
        Check::TranslationsComplete => "Translations".to_owned(),
        // Not in the spec: the drawing has no finding of these three.
        Check::ValidManifest => "Valid module.json".to_owned(),
        Check::NamespaceMatchesFolder => "Namespace".to_owned(),
        Check::NoStaleStatus => "Statuses file".to_owned(),
    }
}

/// A check that found nothing, as a sentence: `Every module.json is valid JSON`.
pub fn check_passed(check: mortise_core::Check, app: &AppSnapshot) -> String {
    use mortise_core::Check;
    match check {
        // Not in the spec: the drawing has findings for both rules.
        Check::NoCircularUse => RULE_NO_CIRCULAR.to_owned(),
        Check::PublicSurfaceOnly => RULE_PUBLIC_SURFACE.to_owned(),
        Check::ValidManifest => "Every module.json is valid JSON".to_owned(),
        Check::NamespaceMatchesFolder => "Declared namespace matches the folder".to_owned(),
        Check::NoStaleStatus => {
            format!(
                "No entry in {} without a module",
                app.path_shown(&app.layout.statuses_file)
            )
        }
        // Not in the spec.
        Check::CoverageLimit => format!("Every module is covered {}% or more", app.coverage_limit),
        Check::TranslationsComplete => "Translations are complete".to_owned(),
    }
}

/// The command a check reads from, after its sentence: `module:lang`.
pub fn check_source(check: mortise_core::Check) -> Option<&'static str> {
    match check {
        mortise_core::Check::TranslationsComplete => Some("module:lang"),
        _ => None,
    }
}

/// What a check looked at: `7 modules`, `7 entries`.
pub fn checked(result: &mortise_core::CheckResult) -> String {
    match result.check {
        mortise_core::Check::NoStaleStatus if result.checked == 1 => "1 entry".to_owned(),
        mortise_core::Check::NoStaleStatus => format!("{} entries", result.checked),
        // `5 modules with lang files`
        mortise_core::Check::TranslationsComplete => {
            format!("{} with lang files", modules(result.checked))
        }
        _ => modules(result.checked),
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

fn capitalised(text: &str) -> String {
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

/// Not in the spec: the row of the palette that opens Settings on PHP.
pub const SETTINGS_PHP: &str = "Settings: PHP";
pub const RUN_PHP_WITH: &str = "Run PHP with";
/// The choices of "Run PHP with", in the order of [`runner_place`].
pub const RUNNERS: [&str; 4] = ["Herd", "Sail", "Docker", "Custom"];
/// Not in the spec: the name a screen reader says for the Test of PHP. The label is
/// "Test", as for an environment, and the add panel shows both.
pub const TEST_PHP: &str = "Test how PHP runs";
/// Not in the spec: the words before the line that Test runs, under the field.
/// A command runs only after its exact line is on screen.
pub const TEST_RUNS: &str = "Test runs";
/// Not in the spec: the tooltip of Sail on Windows (design D6).
pub const SAIL_NOT_ON_WINDOWS: &str = "Sail runs in WSL, not on Windows itself";
pub const ANSWERED: &str = "Answered";
/// Not in the spec: the line under the field while the test runs.
pub const TESTING_PHP: &str = "Running artisan --version";
pub const FOUND_HERD: &str = "Found: this folder is a Herd site.";
/// Not in the spec, written like the Herd hint.
pub const FOUND_SAIL: &str = "Found: vendor/bin/sail and a compose file.";
/// Not in the spec, written like the Herd hint.
pub const FOUND_COMPOSE: &str = "Found: a compose file.";
pub const FOUND_NO_RUNNER: &str = "Not found: Herd, Sail, a compose file.";
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

/// The hint of "Run PHP with" after detection: what the folder points to.
pub fn php_found(found: Option<mortise_core::Detected>) -> &'static str {
    use mortise_core::Detected;
    match found {
        Some(Detected::Herd) => FOUND_HERD,
        Some(Detected::Sail) => FOUND_SAIL,
        Some(Detected::Docker) => FOUND_COMPOSE,
        None => FOUND_NO_RUNNER,
    }
}

/// `10 minutes`
pub fn minutes(n: u16) -> String {
    match n {
        1 => "1 minute".to_owned(),
        n => format!("{n} minutes"),
    }
}

/// Who did not answer, for `x.nophp`: Herd, Sail or Docker, and for Custom the
/// program of the line. Not in the spec: "The command" for a Custom line without one.
pub fn runner_name(runner: &mortise_core::RunnerKind) -> String {
    use mortise_core::RunnerKind;
    match runner {
        RunnerKind::Custom { .. } => runner
            .prefix()
            .into_iter()
            .next()
            .unwrap_or_else(|| "The command".to_owned()),
        other => RUNNERS[runner_place(other)].to_owned(),
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

/// `Test runs herd php artisan --version --no-interaction --ansi`, as a screen
/// reader says it. Not in the spec.
pub fn test_runs(line: &str) -> String {
    format!("{TEST_RUNS} {line}")
}

/// `x.nophp`, under the field of the add panel and of Settings.
pub fn php_failed(runner: &mortise_core::RunnerKind, failure: &mortise_core::Failure) -> String {
    format!(
        "{} did not answer: \"{}\". You can add the app now and read it. Generating and \
         running need PHP: choose another way, or enter the command.",
        runner_name(runner),
        php_reason(failure)
    )
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

pub const LINKS: &str = "Links";
pub const PUBLIC_SURFACE: &str = "Public surface";
pub const SWITCH_WITH: &str = "Switch with";
pub const SCAN_AGAIN: &str = "Scan again";
pub const STOP_THE_SCAN: &str = "Stop the scan";
pub const NOTHING_IS_WRITTEN: &str = "nothing is written";
pub const CLASSES_OTHERS_USE: &str = "Classes that other modules use";
pub const CONTRACTS: &str = "Contracts";
pub const EVENTS: &str = "Events";
pub const OTHER_CLASSES: &str = "Other classes that other modules use";
pub const CONSOLE_COMMANDS: &str = "Console commands";
pub const COL_ENTRY: &str = "Entry";
pub const COL_WHY: &str = "Why it is here";
pub const COL_USED_TODAY: &str = "Used today by";
pub const AN_EVENT: &str = "an event";
pub const ROUTES_WITH_PREFIX: &str = "routes with this prefix";
pub const A_CONSOLE_COMMAND: &str = "a console command";
pub const NOT_USED_OUTSIDE: &str = "not used outside yet";
pub const ADD_BY_HAND: &str = "Add a class, a route or a command";
pub const CANNOT_STORE_SURFACE: &str = "The package cannot store a public surface yet";
pub const ROUTES_NOT_READ: &str =
    "Routes were not read. Connect an environment or set how PHP runs, then scan again.";
/// Not in the spec as a control. The spec says that the proposal can be copied as text.
pub const COPY_AS_TEXT: &str = "Copy as text";
/// Not in the spec: what the toast says after "Copy as text".
pub const COPIED_PROPOSAL: &str = "Copied the proposal.";
/// Not in the spec: an entry the developer added.
pub const ADDED_BY_HAND: &str = "added by hand";
/// Not in the spec: the routes line of the scanning view when no environment is connected.
pub const NOT_READ: &str = "not read";

/// `Billing has no public surface yet`
pub fn no_surface_yet(module: &str) -> String {
    format!("{module} has no public surface yet")
}

pub fn scan_explained(module: &str, folders: &[String]) -> String {
    let names: Vec<&str> = folders.iter().map(String::as_str).collect();
    let rule = match names.len() {
        0 => "Until then the rule has no folders that count as public.".to_owned(),
        1 => format!("Until then the rule uses the folder {}.", names[0]),
        _ => format!("Until then the rule uses the folders {}.", and_list(&names)),
    };
    format!(
        "A scan reads the files of {module} and proposes what other modules, HTTP clients and \
         the command line may use. Nothing is written until you accept the proposal. {rule}"
    )
}

/// `Scan Billing`
pub fn scan_module(module: &str) -> String {
    format!("Scan {module}")
}

/// `Scan the public surface of Billing`
pub fn scan_the_surface_of(module: &str) -> String {
    format!("Scan the public surface of {module}")
}

/// `Scanning Billing`
pub fn scanning(module: &str) -> String {
    format!("Scanning {module}")
}

/// `Reading 61 of 96 files`
pub fn reading_files(read: usize, total: usize) -> String {
    format!("Reading {read} of {}", files(total as u64))
}

/// `3 classes in 6 files`
pub fn classes_in_files(classes: usize, in_files: usize) -> String {
    let classes = match classes {
        1 => "1 class".to_owned(),
        n => format!("{n} classes"),
    };
    format!("{classes} in {}", files(in_files as u64))
}

/// `2 interfaces`
pub fn interfaces(n: usize) -> String {
    match n {
        1 => "1 interface".to_owned(),
        n => format!("{n} interfaces"),
    }
}

/// `3 events`
pub fn events(n: usize) -> String {
    match n {
        1 => "1 event".to_owned(),
        n => format!("{n} events"),
    }
}

/// `2 commands`
pub fn commands(n: usize) -> String {
    match n {
        1 => "1 command".to_owned(),
        n => format!("{n} commands"),
    }
}

/// `Proposed public surface of Billing`
pub fn proposed_surface_of(module: &str) -> String {
    format!("Proposed public surface of {module}")
}

/// `22 found, 21 proposed`
pub fn found_proposed(found: usize, proposed: usize) -> String {
    format!("{found} found, {proposed} proposed")
}

/// `scanned just now`
pub fn scanned_ago(seconds: u64) -> String {
    match seconds {
        0..60 => "scanned just now".to_owned(),
        _ => format!("scanned {}", ago(seconds as i64, 0)),
    }
}

/// `an interface in Contracts`
pub fn an_interface_in(folder: &str) -> String {
    format!("an interface in {folder}")
}

/// `Checkout, 3 files`
pub fn used_by_modules(used_by: &[(String, usize)]) -> String {
    let names: Vec<&str> = used_by.iter().map(|(module, _)| module.as_str()).collect();
    let total: usize = used_by.iter().map(|(_, n)| n).sum();
    format!("{}, {}", names.join(", "), files(total as u64))
}

/// `in Models, used by Checkout, 3 files`
pub fn in_folder_used_by(folder: &str, used_by: &[(String, usize)]) -> String {
    match (folder.is_empty(), used_by.is_empty()) {
        (false, false) => format!("in {folder}, used by {}", used_by_modules(used_by)),
        (true, false) => format!("used by {}", used_by_modules(used_by)),
        (false, true) => format!("in {folder}, {ADDED_BY_HAND}"),
        (true, true) => ADDED_BY_HAND.to_owned(),
    }
}

pub fn not_proposed(modules: &[(String, usize)]) -> String {
    let names: Vec<&str> = modules.iter().map(|(module, _)| module.as_str()).collect();
    format!(
        "Used from outside, not proposed. Include it to allow these reads, or leave it out and \
         keep the finding on {}.",
        and_list(&names)
    )
}

/// Not in the spec: a scan that found nothing.
pub fn nothing_to_propose(module: &str) -> String {
    format!(
        "The scan found no interface, no event and no console command in {module}, and no \
         class that another module uses."
    )
}

/// `Routes, read from local`
pub fn routes_group(environment: &str) -> String {
    format!("Routes, read from {environment}")
}

/// `9 routes, HTTP clients`
pub fn routes_for_clients(n: usize) -> String {
    match n {
        1 => "1 route, HTTP clients".to_owned(),
        n => format!("{n} routes, HTTP clients"),
    }
}

/// `the scheduler, 0 6 * * *`. The schedule is as the app wrote it.
pub fn the_scheduler(schedule: &str) -> String {
    format!("the scheduler, {schedule}")
}

/// `Accept 21 entries`
pub fn accept_entries(n: usize) -> String {
    match n {
        1 => "Accept 1 entry".to_owned(),
        n => format!("Accept {n} entries"),
    }
}

/// Not in the spec: what a name typed by hand turned out to be.
pub fn typed_verdict(typed: &mortise_core::Typed, module: &str) -> String {
    use mortise_core::Typed;
    match typed {
        Typed::Class { folder } if folder.is_empty() => "A class. Enter adds it.".to_owned(),
        Typed::Class { folder } => format!("A class in {folder}. Enter adds it."),
        Typed::Command => "A console command. Enter adds it.".to_owned(),
        Typed::Route { routes: 1 } => "1 route. Enter adds it.".to_owned(),
        Typed::Route { routes } => format!("{routes} routes. Enter adds it."),
        Typed::Listed => "It is in the list.".to_owned(),
        Typed::Unknown => format!("Nothing in {module} has this name."),
    }
}

// --- Runner drawer (S11.1) ---

pub const RUNNER: &str = "Runner";
pub const STOP: &str = "Stop";
pub const HIDE: &str = "Hide";
pub const SHOW: &str = "Show";
pub const RUN_AGAIN: &str = "Run again";
pub const COPY_OUTPUT: &str = "Copy output";
pub const HISTORY: &str = "History";
pub const NOTHING_RAN: &str =
    "No command has run yet. Output of generators and actions appears here.";
/// The keys that show and hide the drawer, on every platform (section 7).
pub const TOGGLE_RUNNER_KEYS: &str = "Ctrl `";
/// Not in the spec: the palette row that shows and hides the drawer, as "Theme: dark" is written.
pub const SHOW_OR_HIDE_RUNNER: &str = "Runner drawer: show or hide";
/// Not in the spec: the name of the handle at the top of the drawer.
pub const DRAWER_HEIGHT: &str = "Height of the runner drawer";
/// Not in the spec: a run whose output passed the limit of the history.
pub const OUTPUT_CUT: &str = "Output past 1 MB is not kept.";
/// Not in the spec: a stop through Sail or Docker reaches `docker` only.
pub const CONTAINER_MAY_RUN: &str = "PHP in the container may still run.";
/// The stop dialog of S10.3.
pub const STOP_BODY: &str =
    "A migration that stops half way can leave the database between two states.";
pub const KEEP_RUNNING: &str = "Keep running";
pub const STOP_COMMAND: &str = "Stop command";
/// Not in the spec: the reason of a start without a runner (design D1).
pub const NO_RUNNER_CHOSEN: &str = "no way to run PHP is chosen";

/// The keys of Stop: `Cmd .` on macOS, `Ctrl .` elsewhere.
pub fn stop_keys(mac: bool) -> String {
    format!("{} .", command_key(mac))
}

/// A time as the drawer writes it: `186 ms` under a second, else `2.4 s`.
pub fn run_time(elapsed: std::time::Duration) -> String {
    let ms = elapsed.as_millis();
    if ms < 1000 {
        format!("{ms} ms")
    } else {
        format!("{:.1} s", elapsed.as_secs_f64())
    }
}

/// `Running, 2.4 s`. Seconds with one decimal.
pub fn running_for(elapsed: std::time::Duration) -> String {
    format!("Running, {:.1} s", elapsed.as_secs_f64())
}

/// `Done in 186 ms`
pub fn done_in(elapsed: std::time::Duration) -> String {
    format!("Done in {}", run_time(elapsed))
}

/// `Exit code 1`. Not in the spec: a program a signal ended.
pub fn exit_code(code: Option<i32>) -> String {
    match code {
        Some(code) => format!("Exit code {code}"),
        None => "Ended by a signal".to_owned(),
    }
}

/// `Stopped after 3.1 s`
pub fn stopped_after(elapsed: std::time::Duration) -> String {
    format!("Stopped after {}", run_time(elapsed))
}

/// Not in the spec: the line under a run that the time limit stopped.
pub fn stopped_at_limit(limit: std::time::Duration) -> String {
    let minutes = (limit.as_secs() / 60).clamp(1, u64::from(u16::MAX)) as u16;
    format!("Stopped at the time limit of {}.", self::minutes(minutes))
}

/// Not in the spec: the sentence under a failed run (design D9). The sentence that
/// names a migration comes with the actions.
pub fn stopped_with(label: &str, code: Option<i32>) -> String {
    match code {
        Some(code) => format!("{label} stopped with exit code {code}."),
        None => format!("{label} was ended by a signal."),
    }
}

/// What the live region says when a run ends: `module:migrate Blog: Done in 186 ms`.
pub fn run_ended(label: &str, status: &str) -> String {
    format!("{label}: {status}")
}

/// `module:migrate --all is running`, the tooltip of a run button that is off.
pub fn is_running(label: &str) -> String {
    format!("{label} is running")
}

/// `Stop module:migrate Checkout?`
pub fn stop_title(label: &str) -> String {
    format!("Stop {label}?")
}

/// `5 runs today in shop-api`
pub fn runs_today(n: usize, app: &str) -> String {
    match n {
        1 => format!("1 run today in {app}"),
        n => format!("{n} runs today in {app}"),
    }
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

/// `The command did not start: "herd: command not found". Choose how PHP runs for shop-api in Settings.`
pub fn did_not_start(error: &mortise_core::StartError, app: &str) -> String {
    format!(
        "The command did not start: \"{}\". Choose how PHP runs for {app} in Settings.",
        start_reason(error)
    )
}

/// Not in the spec: the name of "Open settings" in the drawer, which opens the PHP
/// section, apart from the "Open settings" of the runtime box.
pub fn open_php_settings(app: &str) -> String {
    format!("Open settings, how PHP runs for {app}")
}

/// Not in the spec: the name of the step "Copy output" under a failure, apart from
/// the button of the bar.
pub fn copy_output_of(label: &str) -> String {
    format!("Copy output of {label}")
}

/// `module:migrate Checkout stopped with exit code 1 at 2026_11_20_...` (S11.1 c).
/// A program a signal ended keeps the sentence of [`stopped_with`].
pub fn failed_at(label: &str, code: Option<i32>, migration: &str) -> String {
    match code {
        Some(code) => format!(
            "{label} stopped with exit code {code} at {migration}. The error from the database \
             is quoted above. Fix the migration, or try a dry run to read the SQL."
        ),
        None => stopped_with(label, code),
    }
}

pub const OPEN_MIGRATION: &str = "Open migration file";

/// Not in the spec: Run again of a line Mortise asked about before it ran.
pub fn run_again_title(label: &str) -> String {
    format!("Run {label} again?")
}

/// Not in the spec.
pub const RUN_AGAIN_BODY: &str =
    "Mortise asked before this command ran last time. It runs the same line again.";
pub const DRY_RUN: &str = "Dry run";
pub const SHOW_OUTPUT: &str = "Show output";

/// Not in the spec: the name of the step "Dry run" in the drawer, apart from the
/// "Dry run" of the Actions tab.
pub fn dry_run_of(label: &str) -> String {
    format!("Dry run of {label}")
}

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

/// The drawer, under a run the app prohibits (design D4).
pub const PROHIBITED: &str = "The app prohibits this command, so the package refused to run it. Its message is quoted above.";

/// `Type Checkout to confirm`
pub fn type_to_confirm(name: &str) -> String {
    format!("Type {name} to confirm")
}

// ---------------------------------------------------------------------------
// Generate (S9.1 to S9.6)

/// The title of the Generate tab of a module: `Generate in Billing`.
pub fn generate_in(module: &str) -> String {
    format!("Generate in {module}")
}

/// The title of the new module form: `New module in shop-api`.
pub fn new_module_in(app: &str) -> String {
    format!("New module in {app}")
}

pub const NEW_MODULE: &str = "New module";
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
fn generator_noun(name: &str) -> String {
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
fn with_article(noun: &str) -> String {
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

/// The row of a generator in the palette: `Generate a model in Checkout`.
pub fn generate_row(name: &str, module: &str) -> String {
    format!(
        "Generate {} in {module}",
        with_article(&generator_noun(name))
    )
}

/// The group of the palette that lists generators.
pub const GENERATORS: &str = "Generators";
/// The hint of a generator row of the palette: Enter opens its form.
pub const FORM: &str = "Form";

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
    format!("{} name", generator_label(name))
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
        (n, 0) => format!("{}, none exist yet", files(n as u64)),
        (n, 1) => format!("{}, 1 exists already", files(n as u64)),
        (n, exist) => format!("{}, {exist} exist already", files(n as u64)),
    }
}

/// The head of the file list of `module:make` (S9.4): `Modules/Loyalty does not exist
/// yet`, or, with the module in the snapshot, `Modules/Loyalty exists with 23 files`.
pub fn module_folder_head(folder: &str, files_count: Option<u64>) -> String {
    match files_count {
        Some(n) => format!("{folder} exists with {}", files(n)),
        None => format!("{folder} does not exist yet"),
    }
}

/// The button of a file generator with a file list: `Create 3 files` (S9.2).
pub fn create_n_files(n: usize) -> String {
    format!("Create {}", files(n as u64))
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

/// The error of a class name: `A model name starts with a letter and has no spaces.
/// Write PaymentIntent, not "payment intent".` The example is the typed name as a
/// class name, when it makes one. Only the model sentence is in the spec.
pub fn class_name_error(generator: &str, typed: &str) -> String {
    let first = format!(
        "{} name starts with a letter and has no spaces.",
        capitalised(&with_article(&generator_noun(generator)))
    );
    let studly: String = typed
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '/' && c != '\\')
        .filter(|word| !word.is_empty())
        .map(capitalised)
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
        capitalised(&with_article(&generator_noun(generator)))
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
    format!("Created {done} of {}", files(total as u64))
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
    let mut text = format!("Created {done} of {} in {module}.", files(total as u64));
    match left {
        [] => {}
        [one] => text.push_str(&format!(" {one} already exists and was left as it is.")),
        many => text.push_str(&format!(
            " {} already exist and were left as they are.",
            and_list(many)
        )),
    }
    match failed {
        [] => {}
        [one] => text.push_str(&format!(
            " {one} was not written. The output is in the runner."
        )),
        many => text.push_str(&format!(
            " {} were not written. The output is in the runner.",
            and_list(many)
        )),
    }
    text
}

/// The done toast of a generator run: `Created 3 files in Billing.` A run that wrote
/// files again (not in the spec) says "Replaced".
pub fn created_files(n: usize, module: &str, replaced: bool) -> String {
    let verb = if replaced { "Replaced" } else { "Created" };
    format!("{verb} {} in {module}.", files(n as u64))
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
        many => format!("Replace the modules {}?", and_list(many)),
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
            files(n)
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
        many => format!("Created the modules {}.", and_list(many)),
    }
}

#[cfg(test)]
mod runner_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn times_are_written_as_in_the_drawings() {
        assert_eq!(done_in(Duration::from_millis(186)), "Done in 186 ms");
        assert_eq!(
            stopped_after(Duration::from_millis(3140)),
            "Stopped after 3.1 s"
        );
        assert_eq!(running_for(Duration::from_millis(2420)), "Running, 2.4 s");
        assert_eq!(running_for(Duration::from_millis(300)), "Running, 0.3 s");
    }
}
