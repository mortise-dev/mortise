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

/// Not in the spec: the toast when the editor of the user cannot start.
pub fn could_not_open_the_editor(error: &impl std::fmt::Display) -> String {
    format!("Could not open the editor: \"{error}\". Choose the editor command in Settings.")
}

/// Not in the spec: the title of the dialog that removes an app.
pub fn remove_app_title(name: &str) -> String {
    format!("Remove {name} from Mortise?")
}
/// Not in the spec: the body of the dialog that removes an app.
pub const REMOVE_APP_BODY: &str = "The folder and its files stay as they are. Mortise forgets the app, its environments and its saved credentials.";
/// Not in the spec: the button of the dialog that removes an app.
pub const REMOVE_APP: &str = "Remove app";

/// Not in the spec: the names of the macOS menu bar and its items.
pub const MENU_SETTINGS: &str = "Settings…";
pub const MENU_ADD_APP: &str = "Add App…";
pub const MENU_FILE: &str = "File";
pub const MENU_PALETTE: &str = "Command Palette";
pub const MENU_TOGGLE_INDEX: &str = "Collapse or Open the Module Index";
pub const MENU_READ_AGAIN: &str = "Read the App Again";
pub const MENU_THEME_SYSTEM: &str = "Theme: System";
pub const MENU_THEME_LIGHT: &str = "Theme: Light";
pub const MENU_THEME_DARK: &str = "Theme: Dark";
pub const MENU_VIEW: &str = "View";
pub const MENU_WINDOW: &str = "Window";
pub const MENU_SHORTCUTS: &str = "Keyboard Shortcuts";
pub const MENU_HELP: &str = "Help";

/// Not in the spec: a command asked for while Mortise replaces itself.
pub const INSTALLING_NO_COMMAND: &str =
    "Mortise is installing its newer version and restarts in a moment. Run the command after that.";
