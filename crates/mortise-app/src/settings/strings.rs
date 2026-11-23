//! Interface text of Settings (section 10 of the specification).

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

/// `No answer: service "app" is not running`, in the "Last test" column of S8.2.
pub fn no_answer(failure: &mortise_core::Failure) -> String {
    format!("No answer: {}", crate::strings::php_reason(failure))
}

/// One shortcut of the Keyboard section: what it does, its keys as written for
/// Windows and Linux, and where it works.
pub const SHORTCUTS: &[(&str, &str, &str)] = &[
    ("Open the command palette", "Ctrl K", "Anywhere"),
    // The drawer key is Ctrl on macOS too: Cmd ` cycles windows there (S7, Q9).
    (
        "Show or hide the runner drawer",
        "Ctrl `  or Ctrl J",
        "Anywhere",
    ),
    ("Collapse or open the module index", "Ctrl B", "Anywhere"),
    (
        "Select app 1 to 9 on the rail",
        "Ctrl Alt 1 to 9",
        "Anywhere",
    ),
    ("Add app", "Ctrl O", "Anywhere"),
    ("New module", "Ctrl N", "Anywhere"),
    ("Settings", "Ctrl ,", "Anywhere"),
    (
        "Read the app again, from disk and from the actuator",
        "Ctrl R",
        "Anywhere",
    ),
    ("Stop the running command", "Ctrl .", "Anywhere"),
    ("Close the topmost thing", "Esc", "Anywhere"),
    (
        "Move focus to the next or previous region",
        "F6  Shift F6",
        "Anywhere",
    ),
    (
        "Next or previous module",
        "J  K  or Down  Up",
        "Main window",
    ),
    ("First or last row of the index", "Home  End", "Main window"),
    ("Filter modules", "/  or Ctrl F", "Main window"),
    ("Change the order of the index", "S", "Main window"),
    (
        "Summary, Boundary, Generate, Actions, Metrics",
        "1  2  3  4  5",
        "Main window",
    ),
    (
        "Generate, with the generator filter focused",
        "G",
        "Main window",
    ),
    ("Findings list of the app", "F", "Main window"),
    ("Open the selected module in the editor", "E", "Main window"),
    ("Show the shortcuts", "?", "Main window"),
    (
        "Move focus into the workspace",
        "Enter  or Right",
        "Module index",
    ),
    (
        "Set the width of the index",
        "Left  Right",
        "Edge of the module index",
    ),
    ("Move an app on the rail", "Alt Up  Alt Down", "Rail"),
    ("Move in a table or list", "J  K  or Down  Up", "Workspace"),
    (
        "Change a segmented control or the tab",
        "Left  Right",
        "Workspace",
    ),
    ("Run the generator or the action", "Ctrl Enter", "Workspace"),
    ("Copy the command", "Ctrl Shift C", "Workspace"),
    (
        "Switch between Links and Public surface",
        "P",
        "Boundary, one module",
    ),
    (
        "Scan the public surface",
        "Ctrl Enter",
        "Boundary, Public surface",
    ),
    (
        "Add an entry to the proposal by hand",
        "A",
        "Boundary, Public surface",
    ),
    (
        "Move, run, close",
        "Up  Down  Enter  Esc",
        "Command palette",
    ),
    ("Search in the next app", "Tab", "Command palette"),
    (
        "Run the last command again",
        "Ctrl Shift R",
        "Runner drawer",
    ),
    ("Show the history", "Ctrl Shift H", "Runner drawer"),
    ("Copy the output", "Ctrl Shift O", "Runner drawer"),
    ("Press the focused button, cancel", "Enter  Esc", "Dialog"),
];

/// `3 of 12 shown.`
pub fn shown_of(shown: usize, total: usize) -> String {
    format!("{shown} of {total} shown.")
}

pub const SECTION_APPS: &str = "Apps";
pub const SECTION_GENERAL: &str = "General";
pub const SECTION_KEYBOARD: &str = "Keyboard";
pub const TITLE: &str = "Settings";
pub const SUBTITLE: &str = "Kept on this computer";
pub const ESC_KEY: &str = "Esc";
pub const THEME_HINT: &str = "System follows the operating system and changes with it";
pub const THEME: &str = "Theme";
pub const THEME_CHOICES: &[&str] = &["System", "Light", "Dark"];
pub const TEXT_SIZE_HINT: &str = "every size in the interface scales with it";
pub const TEXT_SIZE: &str = "Text size";
pub const TEXT_SIZE_CHOICES: &[&str] = &["100%", "110%", "125%", "150%"];
pub const MOTION_HINT: &str = "System follows the reduced motion setting";
pub const MOTION: &str = "Motion";
pub const MOTION_CHOICES: &[&str] = &["System", "Reduced", "Full"];
pub const ROWS_HINT: &str = "Automatic is two lines up to 20 modules, one line above";
pub const ROWS: &str = "Module index rows";
pub const ROWS_CHOICES: &[&str] = &["Automatic", "Two lines", "One line"];
pub const OPEN_FILES_WITH: &str = "Open files with";
pub const EDITOR_CHOICES: &[&str] = &["VS Code", "PhpStorm", "Zed", "Custom"];
pub const EDITOR_COMMAND: &str = "Editor command";
pub const CONTROLS_HINT: &str = "where the close, minimise and maximise buttons sit";
pub const CONTROLS: &str = "Window controls on Linux";
pub const CONTROLS_CHOICES: &[&str] = &["Right", "Left"];
pub const TITLE_BAR_HINT: &str = "The platform's own bar keeps what its window manager offers";
pub const TITLE_BAR: &str = "Title bar";
pub const TITLE_BAR_CHOICES: &[&str] = &["Mortise", "Platform"];
pub const COL_FOLDER: &str = "Folder";
pub const COL_VERSIONS: &str = "Versions";
pub const COL_ENVIRONMENTS: &str = "Environments";
pub const COVERAGE_LIMIT: &str = "Coverage limit";
pub const COVERAGE_HINT: &str = "found in phpunit.xml or the usual places";
pub const FIND_SHORTCUT: &str = "Find a shortcut";
pub const SHORTCUTS_NOTE: &str = "On macOS Ctrl is Cmd. Shortcuts are fixed in this version.";
pub const COL_DOES: &str = "Does";
pub const COL_KEYS: &str = "Keys";
pub const COL_WHERE: &str = "Where";
pub const KEYS_OR: &str = "or";

/// `Mortise 0.1.0`
pub fn version(number: &str) -> String {
    format!("Mortise {number}")
}

/// `Back to Apps`
pub fn back_to(place: &str) -> String {
    format!("Back to {place}")
}

/// `Remove shop-api from Mortise`
pub fn remove_app(name: &str) -> String {
    format!("Remove {name} from Mortise")
}

/// `80%`
pub fn percent(value: impl std::fmt::Display) -> String {
    format!("{value}%")
}
