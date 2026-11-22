//! Interface text, from section 10 of the specification.
//!
//! Sentence case, no exclamation marks, no "please". An error says what happened,
//! then what to do next. Text that is not in section 10 is marked "not in the spec".

use std::path::Path;

use mortise_core::{AppSnapshot, LayoutKey, ReadError, Version};

pub const BRAND: &str = "Mortise";
pub const ALL_MODULES: &str = "All modules";
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
        Some(rest) => format!("~/{}", rest.display()),
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
pub fn assumed(app: &AppSnapshot) -> Option<String> {
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
            " If the modules of {} are not in {}, this list is wrong.",
            app.name,
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
pub fn state_not_in_a_file(app: &AppSnapshot) -> String {
    format!(
        "{} keeps the state of its modules with the activator \"{}\", not in a file. \
         Mortise reads files only, so it cannot tell which modules are on.",
        app.name, app.layout.activator
    )
}

/// Not in the spec: an app whose modules folder holds no module.
pub fn no_modules(app: &AppSnapshot) -> String {
    format!(
        "{} has no modules in {}.",
        app.name,
        app.modules_dir_shown()
    )
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
