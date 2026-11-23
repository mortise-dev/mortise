//! Interface text of adding an app (section 10 of the specification).

use mortise_core::AppSnapshot;

pub const BACK: &str = "Back";
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
pub const MODULES: &str = "Modules";
pub const LARAVEL: &str = "Laravel";
pub const CHECK_AGAIN: &str = "Check again";
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
pub const NO_REPORT_FOUND: &str = "none found, coverage shows as unknown";

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
        mortise_core::Git::NotARepository => {
            (crate::strings::NOT_A_REPOSITORY.to_owned(), String::new())
        }
        mortise_core::Git::NoGit => (crate::strings::GIT_NOT_FOUND.to_owned(), String::new()),
    }
}
pub const RUNTIME_OPTIONAL: &str = "Runtime, optional";
