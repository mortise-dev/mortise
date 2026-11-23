//! Interface text, from section 10 of the specification.
//!
//! Sentence case, no exclamation marks, no "please". An error says what happened,
//! then what to do next. Text that is not in section 10 is marked "not in the spec".

use std::path::Path;

use mortise_core::{AppSnapshot, LayoutKey, ReadError, Version};

pub const BRAND: &str = "Mortise";
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
/// Under 1100 px of window width.
pub const SEARCH_SHORT: &str = "Search or run";
pub const SORTED_BY_NAME: &str = "Sorted: by name";
pub const MODULES_ON: &str = "Modules on";
pub const MODULE_STATE: &str = "Module state";
pub const ON: &str = "On";
pub const OFF: &str = "Off";
/// Not in the spec: the state of a module when the app does not use the file activator.
pub const STATE_UNKNOWN: &str = "Unknown";

pub const NO_APPS_TITLE: &str = "No apps yet";
pub const NO_APPS_BODY: &str = "Add the folder of a Laravel app that uses nwidart/laravel-modules. \
    Mortise reads its modules from disk and changes nothing until you run a command.";
pub const NO_APP: &str = "No app";
pub const ADD_APP: &str = "Add app";
pub const ADD_APP_FOLDER: &str = "Add app folder";
pub const BACK: &str = "Back";
pub const CANCEL: &str = "Cancel";
pub const APP_FOLDER: &str = "App folder";
pub const APP_FOLDER_HINT: &str = "the folder that holds artisan";
pub const PATH_PLACEHOLDER: &str = "Path to the app, or drop the folder on this window";
pub const CHOOSE_FOLDER: &str = "Choose folder";
pub const CHOOSE_FOLDER_HINT: &str = "Opens the folder dialog of the operating system";
pub const READS_NOTE: &str = "Mortise reads composer.lock, the module.json files and \
    modules_statuses.json. It writes nothing into the app.";
pub const FOUND_TITLE: &str = "Found in this folder";
pub const FOUND_NOTE: &str = "read from disk, nothing run";
pub const SHOWN_TITLE: &str = "Shown in the rail as";
pub const SHOWN_NOTE: &str = "you can change both in Settings";
pub const NAME: &str = "Name";
pub const MONOGRAM: &str = "Monogram";
pub const MODULES: &str = "Modules";
pub const LARAVEL: &str = "Laravel";
pub const COPY_COMMAND: &str = "Copy command";
pub const CHECK_AGAIN: &str = "Check again";
pub const COPIED_COMMAND: &str = "Copied the command.";
pub const REQUIRE_PACKAGE: &str = "composer require nwidart/laravel-modules";
/// Not in the spec: the actuator line of "Found in this folder" when the app does not have it.
pub const NOT_INSTALLED: &str = "not installed";

/// `Add booking-engine`
pub fn add_named(name: &str) -> String {
    format!("Add {name}")
}

/// `Add booking-engine, read only`: the add button after the test of PHP failed (`b.addro`).
pub fn add_read_only(name: &str) -> String {
    format!("Add {name}, read only")
}

/// Not in the spec: the button of an app that is already in Mortise.
pub fn open_named(name: &str) -> String {
    format!("Open {name}")
}

/// `Added booking-engine. 5 modules read.`
pub fn added(name: &str, modules: usize) -> String {
    let read = if modules == 1 {
        "1 module read".to_owned()
    } else {
        format!("{modules} modules read")
    };
    format!("Added {name}. {read}.")
}

/// `shop-api is already in Mortise. Press Enter to open it.`
pub fn already_added(name: &str) -> String {
    format!("{name} is already in Mortise. Press Enter to open it.")
}

/// `5 in Modules/, 5 on`
pub fn modules_found(app: &AppSnapshot) -> (String, String, String) {
    let on = if app.layout.uses_file_activator() {
        format!(", {} on", app.modules_on())
    } else {
        String::new()
    };
    (
        format!("{} in ", app.modules.len()),
        format!("{}/", app.modules_dir_shown()),
        on,
    )
}

/// Not in the spec: the title when an app Mortise knows cannot be read any more.
pub fn cannot_read_title(name: &str) -> String {
    format!("{name} cannot be read")
}

/// Not in the spec: Mortise's own settings file is broken. The broken file is kept.
pub fn settings_invalid(kept_as: &Path) -> String {
    format!(
        "Mortise could not read its settings, so it starts without apps. The old file is kept as {}.",
        tilde(kept_as)
    )
}

pub const MINIMISE: &str = "Minimise";
pub const MAXIMISE: &str = "Maximise";
/// Not in the spec.
pub const RESTORE: &str = "Restore";
pub const CLOSE: &str = "Close";
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

/// `CHK is the code for Checkout`
pub fn code_is_for(code: &str, module: &str) -> String {
    format!("{code} is the code for {module}")
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
/// Not in the spec: the name of the edge of the index that sets its width.
pub const INDEX_WIDTH: &str = "Width of the module index";
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
pub const NO_REPORT_FOUND: &str = "none found, coverage shows as unknown";
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

/// `branch main`
pub fn git_found(git: &mortise_core::Git) -> (String, String) {
    match git {
        mortise_core::Git::Repository {
            branch: Some(branch),
            ..
        } => ("branch ".to_owned(), branch.clone()),
        // Not in the spec: a commit that is checked out without a branch.
        mortise_core::Git::Repository {
            branch: None,
            commit: Some(commit),
        } => ("at ".to_owned(), commit.clone()),
        mortise_core::Git::Repository { .. } => ("no commit yet".to_owned(), String::new()),
        mortise_core::Git::NotARepository => (NOT_A_REPOSITORY.to_owned(), String::new()),
        mortise_core::Git::NoGit => (GIT_NOT_FOUND.to_owned(), String::new()),
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

/// What a screen reader says for "Open in editor" in the header of a module.
pub fn open_module_in_editor(module: &str) -> String {
    format!("Open {module} in the editor")
}
/// Not in the spec: the toast after "Copy path".
pub const COPIED_PATH: &str = "Copied the path.";
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
/// Not in the spec: the button of the dialog that asks before mortise.json is created.
pub const CREATE_RULES_FILE: &str = "Create mortise.json";

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

/// Not in the spec: the dialog before the rules file is created.
pub fn create_rules_title(app: &str) -> String {
    format!("Create mortise.json in {app}?")
}

/// Not in the spec.
pub fn create_rules_body(app: &str) -> String {
    format!(
        "Mortise keeps the boundary rules of {app} in mortise.json in the app folder, so a team \
         shares them. It is the only file Mortise writes into the app without a command."
    )
}

/// Not in the spec: mortise.json could not be written. The system's message is quoted.
pub fn rules_not_saved(error: &mortise_core::RulesError) -> String {
    format!("The rules were not saved: {error}.")
}

/// Not in the spec: mortise.json is there but cannot be read as rules.
pub fn rules_invalid(message: &str) -> String {
    format!("mortise.json is not valid: {message}. The default rules apply until it is corrected.")
}

pub const FINDINGS: &str = "Findings";
pub const SORTED_PROBLEMS_FIRST: &str = "Sorted: problems first";
pub const CHANGE_THE_ORDER: &str = "Change the order";
pub const NEEDS_ATTENTION: &str = "Needs attention";
pub const BACK_TO_SUMMARY: &str = "Back to Summary";
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

/// `shop-api, 2 problems`
pub fn app_with_problems(app: &str, problem_count: usize) -> String {
    if problem_count == 0 {
        app.to_owned()
    } else {
        format!("{app}, {}", problems(problem_count))
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
    }
}

/// The name of a check in the Findings list: `Rule: no circular use`.
pub fn check_name(check: mortise_core::Check, app: &AppSnapshot) -> String {
    use mortise_core::Check;
    match check {
        Check::NoCircularUse => "Rule: no circular use".to_owned(),
        Check::PublicSurfaceOnly => format!("Rule: {} only", list_of(&app.rules.public_folders)),
        Check::CoverageLimit => "Coverage limit".to_owned(),
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
    }
}

/// What a check looked at: `7 modules`, `7 entries`.
pub fn checked(result: &mortise_core::CheckResult) -> String {
    match result.check {
        mortise_core::Check::NoStaleStatus if result.checked == 1 => "1 entry".to_owned(),
        mortise_core::Check::NoStaleStatus => format!("{} entries", result.checked),
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
pub const RUNTIME_OPTIONAL: &str = "Runtime, optional";
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

/// The header of the Actions tab: `Run with` and the keys.
pub const RUN_WITH: &str = "Run with";
pub const COMMAND: &str = "Command";
pub const DRY_RUN_OPTION: &str = "Dry run: print the SQL, change nothing";
pub const SEED_AFTERWARDS: &str = "Seed afterwards";
pub const DATABASE_CONNECTION: &str = "Database connection";
pub const SEEDER_CLASS: &str = "Seeder class";
/// The placeholder of the database field.
pub const DEFAULT_CONNECTION: &str = "default";
pub const ROLL_BACK_HINT: &str = "Destructive. Mortise asks before it runs. The command has no \
                                  option to choose how many batches.";
/// The placeholder in the line of Run tests (S10.2 e).
pub const TESTS_PLACEHOLDER: &str = "[the package's command for the tests of one module]";
/// Not in the spec: why Run tests and its Copy command are off (design D4).
pub const NO_TESTS_COMMAND: &str = "The package has no command for the tests of one module yet.";

/// The hint of the database field: Migrate says what an empty field means, Seed
/// names the option only.
pub fn database_hint(action: mortise_core::Action) -> &'static str {
    match action {
        mortise_core::Action::Seed => "--database",
        _ => "--database, empty for the default",
    }
}

/// The hint of the seeder field. Not in the spec for "All modules".
pub fn class_hint(all: bool) -> &'static str {
    if all {
        "--class, empty for the seeder of each module"
    } else {
        "--class, empty for the module seeder"
    }
}

/// `CheckoutDatabaseSeeder`, the seeder the package runs when the field is empty.
pub fn module_seeder(module: &str) -> String {
    format!("{module}DatabaseSeeder")
}

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

/// The title of the panel of an action: `Migrate Checkout`, `Migrate all modules`.
/// Not in the spec: the titles of Check translations, and of Run tests and Check
/// translations for "All modules".
pub fn action_title(action: mortise_core::Action, module: Option<&str>) -> String {
    use mortise_core::Action;
    match (action, module) {
        (Action::RunTests, Some(module)) => format!("Run tests of {module}"),
        (Action::CheckTranslations, Some(module)) => format!("Check translations of {module}"),
        (Action::CheckTranslations, None) => "Check translations of all modules".to_owned(),
        (action, None) => action_row(action, true).to_owned(),
        (action, Some(module)) => format!("{} {module}", action_button(action)),
    }
}

/// The sentence under the title of the panel. Not in the spec: the sentences of
/// Check translations, and of Seed for "All modules". Run tests has its own.
pub fn action_text(action: mortise_core::Action, module: Option<&str>) -> String {
    use mortise_core::Action;
    match (action, module) {
        (Action::TurnOff, Some(module)) => format!(
            "{module} is on. The app stops loading its providers, routes and migrations. Its \
             files stay."
        ),
        (Action::TurnOn, Some(module)) => {
            format!("{module} is off. Turning it on does not run its migrations.")
        }
        (Action::Migrate, Some(module)) => {
            format!("Runs the migrations of {module} that have not run yet.")
        }
        (Action::Migrate, None) => "Runs the waiting migrations of every module that is on, in \
                                    the package's order."
            .to_owned(),
        (Action::RollBack, Some(module)) => {
            format!("Runs the down method of the last migration batch of {module}.")
        }
        (Action::Seed, Some(module)) => format!(
            "Runs the seeders of {module}. Rows are added; running it twice can add them twice."
        ),
        (Action::Seed, None) => "Runs the seeders of every module that is on. Rows are added; \
                                 running it twice can add them twice."
            .to_owned(),
        (Action::CheckTranslations, Some(module)) => format!(
            "Lists the translation keys that a language of {module} lacks. The output shows in \
             the runner."
        ),
        (Action::CheckTranslations, None) => "Lists the translation keys that a language of \
                                              each module lacks. The output shows in the runner."
            .to_owned(),
        _ => String::new(),
    }
}

/// The sentence of Run tests after its count and folder (S10.2 e).
pub const TESTS_TEXT: &str = "The package will provide a command that runs the tests of one \
                              module. Its name and options are not decided, so the line below \
                              has a placeholder and no options are drawn.";

/// Not in the spec: where the tests of "All modules" are, after their count.
pub const TESTS_IN_MODULES_ON: &str = "in the modules that are on.";

/// The alert of Turn off (S10.2 a): `Shipping and Notifications use Checkout. ...`
/// Not in the spec: the sentence for one module.
pub fn turn_off_alert(users: &[&str], module: &str) -> String {
    let verb = if users.len() == 1 { "uses" } else { "use" };
    format!(
        "{} {verb} {module}. Mortise asks before it turns off a module that others use.",
        and_list(users)
    )
}

/// The dialog of Turn off (S10.3). Not in the spec: the sentence for one module.
pub fn turn_off_title(module: &str) -> String {
    format!("Turn off {module}?")
}

pub fn turn_off_body(users: &[&str], module: &str) -> String {
    match users {
        [one] => format!("{one} uses it. It stays on and can fail where it calls {module}."),
        _ => format!(
            "{} use it. They stay on and can fail where they call {module}.",
            and_list(users)
        ),
    }
}

pub fn turn_off_confirm(module: &str) -> String {
    format!("Turn off {module}")
}

/// The dialog of Roll back (S10.3). The button has no count without the migration
/// state (deviation S41).
pub fn roll_back_title(object: &str) -> String {
    format!("Roll back {object}?")
}

pub fn roll_back_body(object: &str) -> String {
    format!(
        "This runs the down method of the last migration batch of {object}. Data in the \
         columns and tables it drops is lost."
    )
}

/// The question before migrate or seed in production. Not in the spec: the title
/// and the button of seed.
pub fn production_title(action: mortise_core::Action, object: &str) -> String {
    match action {
        mortise_core::Action::Seed => format!("Seed {object} in production?"),
        _ => format!("Migrate {object} in production?"),
    }
}

pub fn production_body(app: &str) -> String {
    format!(
        "{app} reports the environment production. The command runs with --force on that \
         database."
    )
}

pub fn production_confirm(action: mortise_core::Action) -> &'static str {
    match action {
        mortise_core::Action::Seed => "Seed in production",
        _ => "Migrate in production",
    }
}

/// What an action names in a toast or a dialog for "All modules".
pub const ALL_MODULES_OBJECT: &str = "all modules";

/// The done toasts of C13. Not in the spec: the toasts of a run that changed nothing.
pub fn turned_off(object: &str) -> String {
    format!("Turned off {object}.")
}

pub fn turned_on(object: &str) -> String {
    format!("Turned on {object}.")
}

pub fn migrated(object: &str, done: usize) -> String {
    match done {
        0 => format!("Migrated {object}: nothing to migrate."),
        1 => format!("Migrated {object}: 1 migration ran."),
        n => format!("Migrated {object}: {n} migrations ran."),
    }
}

pub fn rolled_back(object: &str, done: usize) -> String {
    match done {
        0 => format!("Rolled back {object}: nothing to roll back."),
        1 => format!("Rolled back {object}: 1 migration."),
        n => format!("Rolled back {object}: {n} migrations."),
    }
}

pub fn seeded(object: &str) -> String {
    format!("Seeded {object}.")
}

/// Not in the spec: the failed toast of a run that was stopped.
pub fn was_stopped(label: &str) -> String {
    format!("{label} was stopped.")
}

/// `1 seeder`, `5 seeders`
pub fn seeders(n: u32) -> String {
    if n == 1 {
        "1 seeder".to_owned()
    } else {
        format!("{} seeders", count(u64::from(n)))
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
