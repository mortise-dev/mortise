//! Interface text of the command palette (section 10 of the specification).

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
            action => format!("{} {module}", crate::strings::action_button(action)),
        },
        None => crate::strings::action_row(action, true).to_owned(),
    }
}

/// The age of a recent action in the palette, in seconds since 1970: `ran 2 minutes ago`
/// (S12.2 a). The other units, such as `ran 8 s ago`, are not in the spec.
pub fn ran_ago(now: i64, then: i64) -> String {
    format!("ran {}", crate::strings::ago(now, then))
}

/// `Modules in shop-api`: the group of modules in the palette.
pub fn modules_in(app: &str) -> String {
    format!("Modules in {app}")
}

/// Not in the spec: the row of the palette that opens Settings on PHP.
pub const SETTINGS_PHP: &str = "Settings: PHP";

/// `Scan the public surface of Billing`
pub fn scan_the_surface_of(module: &str) -> String {
    format!("Scan the public surface of {module}")
}
/// Not in the spec: the palette row that shows and hides the drawer, as "Theme: dark" is written.
pub const SHOW_OR_HIDE_RUNNER: &str = "Runner drawer: show or hide";

/// The row of a generator in the palette: `Generate a model in Checkout`.
pub fn generate_row(name: &str, module: &str) -> String {
    format!(
        "Generate {} in {module}",
        crate::strings::with_article(&crate::strings::generator_noun(name))
    )
}

/// The group of the palette that lists generators.
pub const GENERATORS: &str = "Generators";
/// The hint of a generator row of the palette: Enter opens its form.
pub const FORM: &str = "Form";

/// The hint of a module row of the palette, and of the current row with no hint.
pub const OPEN: &str = "Open";
/// The name of the Option key on macOS.
pub const KEY_OPTION: &str = "Option";
/// The name of the Alt key on Windows and Linux.
pub const KEY_ALT: &str = "Alt";
/// The group of the recent rows.
pub const GROUP_RECENT: &str = "Recent";
/// The group of the rows that go to a screen.
pub const GROUP_GO_TO: &str = "Go to";
/// The group of the apps.
pub const GROUP_APPS: &str = "Apps";
/// Not in the spec: the row of the palette that sets the theme to follow the system.
pub const THEME_SYSTEM: &str = "Theme: follow the system";
/// Not in the spec: the row of the palette that sets the light theme.
pub const THEME_LIGHT: &str = "Theme: light";
/// Not in the spec: the row of the palette that sets the dark theme.
pub const THEME_DARK: &str = "Theme: dark";
/// The hint of the query field.
pub const SEARCH_HINT: &str = "Search modules, findings and apps";
/// The keys named in the footer of the palette.
pub const FOOTER_KEYS: &str = "Up and Down to move, Enter to run, Tab to switch app";

/// The text of a finding that reads internals: `3 files in Billing read Checkout internals`.
pub fn reads_internals(files: u64, module: &str, of: &str) -> String {
    format!(
        "{} in {module} read {of} internals",
        crate::strings::files(files)
    )
}

/// The text of the palette when no row matches.
pub fn nothing_matches(query: &str) -> String {
    format!(
        "Nothing matches \"{query}\". Try a module name, the name of an app, or a word of a finding."
    )
}
