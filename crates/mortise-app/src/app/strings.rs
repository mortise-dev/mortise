//! Interface text of the window (section 10 of the specification).

use std::path::Path;

pub const BRAND: &str = "Mortise";
/// Under 1100 px of window width.
pub const SEARCH_SHORT: &str = "Search or run";

pub const NO_APPS_TITLE: &str = "No apps yet";
pub const NO_APP: &str = "No app";
pub const ADD_APP_FOLDER: &str = "Add app folder";

/// Not in the spec: the title when an app Mortise knows cannot be read any more.
pub fn cannot_read_title(name: &str) -> String {
    format!("{name} cannot be read")
}

/// Not in the spec: Mortise's own settings file is broken. The broken file is kept.
pub fn settings_invalid(kept_as: &Path) -> String {
    format!(
        "Mortise could not read its settings, so it starts without apps. The old file is kept as {}.",
        crate::strings::tilde(kept_as)
    )
}

pub const MINIMISE: &str = "Minimise";
pub const MAXIMISE: &str = "Maximise";
/// Not in the spec.
pub const RESTORE: &str = "Restore";
pub const CLOSE: &str = "Close";

/// `CHK is the code for Checkout`
pub fn code_is_for(code: &str, module: &str) -> String {
    format!("{code} is the code for {module}")
}
/// Not in the spec: the name of the edge of the index that sets its width.
pub const INDEX_WIDTH: &str = "Width of the module index";

/// What a screen reader says for "Open in editor" in the header of a module.
pub fn open_module_in_editor(module: &str) -> String {
    format!("Open {module} in the editor")
}
/// Not in the spec: the toast after "Copy path".
pub const COPIED_PATH: &str = "Copied the path.";
/// Not in the spec: the button of the dialog that asks before mortise.json is created.
pub const CREATE_RULES_FILE: &str = "Create mortise.json";

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
pub const BACK_TO_SUMMARY: &str = "Back to Summary";

/// `shop-api, 2 problems`
pub fn app_with_problems(app: &str, problem_count: usize) -> String {
    if problem_count == 0 {
        app.to_owned()
    } else {
        format!("{app}, {}", crate::strings::problems(problem_count))
    }
}

/// The header of the Actions tab: `Run with` and the keys.
pub const RUN_WITH: &str = "Run with";

/// The title of the Metrics tab while it cannot do its work (S7.4).
pub const METRICS_UNAVAILABLE_TITLE: &str = "Metrics are not in this version";
/// Not in the spec: the paragraph under the Metrics title.
pub const METRICS_UNAVAILABLE: &str = "Metrics show requests, response time, errors, jobs and database time per module, read from the prometheus endpoint of the running app. They arrive in a later release of Mortise.";
