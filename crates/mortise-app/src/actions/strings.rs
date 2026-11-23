//! Interface text of the Actions tab (section 10 of the specification).

pub const DRY_RUN_OPTION: &str = "Dry run: print the SQL, run none of it";
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
/// Not in the spec: the options of `module:test` (13.4, design D8).
pub const TESTS_FILTER: &str = "Filter, a test name or pattern";
pub const RUN_IN_PARALLEL: &str = "Run in parallel";

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

/// The title of the panel of an action: `Migrate Checkout`, `Migrate all modules`.
/// Not in the spec: the titles of Check translations, and of Run tests and Check
/// translations for "All modules".
pub fn action_title(action: mortise_core::Action, module: Option<&str>) -> String {
    use mortise_core::Action;
    match (action, module) {
        (Action::RunTests, Some(module)) => format!("Run tests of {module}"),
        (Action::CheckTranslations, Some(module)) => format!("Check translations of {module}"),
        (Action::CheckTranslations, None) => "Check translations of all modules".to_owned(),
        (action, None) => crate::strings::action_row(action, true).to_owned(),
        (action, Some(module)) => format!("{} {module}", crate::strings::action_button(action)),
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
        (Action::ResetMigrations, Some(module)) => {
            format!("Rolls back every migration of {module} that has run, newest first.")
        }
        (Action::RefreshMigrations, Some(module)) => format!(
            "Rolls back every migration of {module}, then runs them all again. The command has \
             no dry run."
        ),
        (Action::DeleteModule, Some(_)) => "Removes the module from the app. The command always \
                                            asks, so Mortise asks and then passes --force."
            .to_owned(),
        (Action::FreshMigrate, None) => "Runs db:wipe on the database, then the migrations of \
                                         the app, then those of every module that is on. The \
                                         command refuses a dry run."
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

/// Not in the spec: where the tests of "All modules" are on 13.4, after their count.
/// `module:test --all` takes every module, on or off.
pub const TESTS_IN_EVERY_MODULE: &str = "in every module, on or off.";

/// The toast of Run tests that passed: `31 tests passed in Checkout.` (C13). For
/// "All modules" the object is the count of modules that ran: not in the spec.
pub fn tests_passed(passed: u32, object: &str) -> String {
    format!("{} passed in {object}.", crate::strings::tests(passed))
}

/// Not in the spec: the toast of Run tests that failed, `3 of 31 tests failed in Checkout.`
pub fn tests_failed(failed: u32, total: u32, object: &str) -> String {
    format!(
        "{failed} of {} failed in {object}.",
        crate::strings::tests(total)
    )
}

/// Not in the spec: the toast of Run tests in a module without a tests folder.
pub fn no_tests_folder(module: &str) -> String {
    format!("{module} has no tests folder.")
}

/// The alert of Turn off (S10.2 a): `Shipping and Notifications use Checkout. ...`
/// Not in the spec: the sentence for one module.
pub fn turn_off_alert(users: &[&str], module: &str) -> String {
    let verb = if users.len() == 1 { "uses" } else { "use" };
    format!(
        "{} {verb} {module}. Mortise asks before it turns off a module that others use.",
        crate::strings::and_list(users)
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
            crate::strings::and_list(users)
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

/// The title of the state box of Migrate (S10.1, S10.2 f).
pub const WAITING: &str = "Waiting";
/// The title of the state box of Roll back (S10.2 c).
pub const LAST_BATCH: &str = "Last batch";
/// Not in the spec: the title of the state box of Reset and Refresh.
pub const RAN: &str = "Ran";
/// Not in the spec: the state box before a read.
pub const NOT_READ_YET: &str = "Not read yet.";
/// Not in the spec: the state box when the state was read for another connection.
pub const NOT_READ_FOR_CONNECTION: &str = "Not read for this connection.";
/// Not in the spec: the state box of "All modules" when some of the modules that
/// are on were read, not all.
pub const NOT_READ_FOR_EVERY_MODULE: &str = "Not read for every module yet.";
/// Not in the spec: the button that reads the migration state.
pub const READ_MIGRATION_STATE: &str = "Read migration state";
/// Not in the spec: the state box while its read runs.
pub const STATE_READING: &str = "Reading";
/// Not in the spec: the button that reads the migration state again.
pub const STATE_READ_AGAIN: &str = "Read again";
/// Not in the spec: the state box of Roll back, Reset and Refresh when nothing ran.
pub const NOTHING_RAN_YET: &str = "Nothing ran";
/// Not in the spec: the state box when the database has no migrations table.
pub const NO_MIGRATIONS_TABLE: &str = "The database has no migrations table yet.";

/// `1 migration`, `9 migrations` (S10.1).
pub fn migrations(n: usize) -> String {
    match n {
        1 => "1 migration".to_owned(),
        n => format!("{} migrations", crate::strings::count(n as u64)),
    }
}

/// `1 migration in 1 module` (S10.2 f). Not in the spec: the other counts.
pub fn migrations_in_modules(n: usize, modules: usize) -> String {
    match modules {
        1 => format!("{} in 1 module", migrations(n)),
        m => format!(
            "{} in {} modules",
            migrations(n),
            crate::strings::count(m as u64)
        ),
    }
}

/// The row detail of Roll back: `last batch, 1` (S10.1).
pub fn last_batch(n: usize) -> String {
    format!("last batch, {}", crate::strings::count(n as u64))
}

/// `Roll back 1 migration`, the button of the roll back dialog (S10.3). Not in the
/// spec: the other counts.
pub fn roll_back_counted(n: usize) -> String {
    format!("Roll back {}", migrations(n))
}

/// `Reset 9 migrations` (S10.5 a).
pub fn reset_counted(n: usize) -> String {
    format!("Reset {}", migrations(n))
}

/// `Refresh 9 migrations` (S10.5 b).
pub fn refresh_counted(n: usize) -> String {
    format!("Refresh {}", migrations(n))
}

/// Not in the spec: when the state was read, `Read at 14:05`.
pub fn state_read_at(clock: &str) -> String {
    format!("Read at {clock}")
}

/// Not in the spec, design D2: a state read that gave no document.
pub fn migration_state_failed(reason: &str) -> String {
    format!("Mortise could not read the migration state: \"{reason}\".")
}

/// `7 tables` (S10.4).
pub fn tables(n: usize) -> String {
    match n {
        1 => "1 table".to_owned(),
        n => format!("{} tables", crate::strings::count(n as u64)),
    }
}

/// ` and 2 more`, after the tables named in a row of "What is lost" (S10.4). Not in
/// the spec: after the migrations or tables a dialog lists, at most six.
pub fn and_more(n: usize) -> String {
    format!(" and {} more", crate::strings::count(n as u64))
}

/// `Every table and column created by the 9 migrations of Checkout that ran` (S10.4).
/// Not in the spec: the sentence for one migration.
pub fn reset_loses_counted(module: &str, ran: usize) -> String {
    match ran {
        1 => format!("Every table and column created by the 1 migration of {module} that ran"),
        n => format!(
            "Every table and column created by the {} migrations of {module} that ran",
            crate::strings::count(n as u64)
        ),
    }
}

/// `7 tables come back empty` (S10.4 a). Not in the spec: one table.
pub fn tables_come_back_empty(n: usize) -> String {
    match n {
        1 => "1 table comes back empty".to_owned(),
        n => format!("{} come back empty", tables(n)),
    }
}

/// `7 tables. Nothing outside Checkout is touched.` (S10.5 a)
pub fn tables_untouched(n: usize, module: &str) -> String {
    format!("{}. {}", tables(n), nothing_outside(module))
}

/// `7 tables. This command has no dry run.` (S10.5 b)
pub fn tables_no_dry_run(n: usize) -> String {
    format!("{}. {NO_DRY_RUN}", tables(n))
}

/// The row of the closed group in the list of actions.
pub const DESTRUCTIVE: &str = "Destructive";
/// The detail of the group row while it is open.
pub const GROUP_OPEN: &str = "open";
/// Not in the spec: the detail of the group row while it is closed.
pub const GROUP_CLOSED: &str = "closed";
/// The dim line under the group of a module.
pub const FRESH_IS_UNDER_ALL: &str = "Dropping all tables is under \"All modules\".";
pub const WHAT_IS_LOST: &str = "What is lost";
pub const CANNOT_BE_UNDONE: &str = "cannot be undone";
pub const WITH_ALL_ROWS: &str = "with all rows";
/// The value of the row of refresh. The spec has "7 tables come back empty"; the
/// count of tables needs the migration state (deviation S42).
pub const COME_BACK_EMPTY: &str = "come back empty";
pub const FRESH_LOSES_TABLES: &str = "Every table of the database, also those that no module owns";
pub const VIEWS_IF_CHOSEN: &str = "Views, if chosen below";
pub const TYPES_IF_CHOSEN: &str = "Types, if chosen below";
pub const POSTGRES_ONLY: &str = "Postgres only";
pub const ALSO_DROP_VIEWS: &str = "Also drop views";
pub const ALSO_DROP_TYPES: &str = "Also drop types";
/// The part of the folder row of delete before the folder, which is set in mono.
pub const THE_FOLDER: &str = "The folder ";
/// The part of the statuses row of delete before the file, which is set in mono.
pub const ITS_LINE_IN: &str = "Its line in ";
pub const UNCOMMITTED_CHANGES: &str = "Changes that are not committed";
/// Delete, when the app is not in a git repository or git cannot be asked.
pub const CANNOT_TELL_GIT: &str = "Mortise cannot tell whether these files are in git.";
/// Refresh in its dialog.
pub const NO_DRY_RUN: &str = "This command has no dry run.";

/// The title of the panel of a destructive action: `Reset the migrations of Checkout`,
/// `Drop all tables of shop-api and migrate`.
pub fn destructive_title(action: mortise_core::Action, module: &str, app: &str) -> String {
    use mortise_core::Action;
    match action {
        Action::ResetMigrations => format!("Reset the migrations of {module}"),
        Action::RefreshMigrations => format!("Refresh the migrations of {module}"),
        Action::FreshMigrate => format!("Drop all tables of {app} and migrate"),
        _ => format!("Delete the module {module}"),
    }
}

/// The first row of "What is lost" of reset. The spec counts the migrations, "the
/// 9 migrations"; the count needs the migration state (deviation S42).
pub fn reset_loses(module: &str) -> String {
    format!("Every table and column created by the migrations of {module} that ran")
}

/// The row of "What is lost" of refresh.
pub fn refresh_loses(module: &str) -> String {
    format!("Every row in the tables of {module}")
}

/// `Stays: the tables of Checkout in the database and the record of its migrations.`
pub fn stays(module: &str) -> String {
    format!("Stays: the tables of {module} in the database and the record of its migrations.")
}

/// `Shipping and Notifications use Checkout and will break.` Not in the spec: the
/// sentence for one module.
pub fn will_break(users: &[&str], module: &str) -> String {
    let verb = if users.len() == 1 { "uses" } else { "use" };
    format!(
        "{} {verb} {module} and will break.",
        crate::strings::and_list(users)
    )
}

/// The production alert of S10.4 d, and the tooltip of the run button it turns off.
pub fn production_refused(app: &str) -> String {
    format!(
        "{app} reports the environment production. Mortise does not run this command there. \
         Copy it and run it where you can watch the database."
    )
}

/// Not in the spec: why Delete module only copies its line when the modules folder
/// cannot be read from `config/modules.php`.
pub fn modules_folder_assumed(app: &str) -> String {
    format!(
        "Mortise cannot read where {app} keeps its modules, so the package could delete \
         another folder. Copy the command and run it where you can check the folder."
    )
}

/// The title of the dialog of a destructive action (S10.5).
pub fn destructive_question(action: mortise_core::Action, module: &str, app: &str) -> String {
    use mortise_core::Action;
    match action {
        Action::ResetMigrations => format!("Reset the migrations of {module}?"),
        Action::RefreshMigrations => format!("Refresh the migrations of {module}?"),
        Action::FreshMigrate => format!("Drop all tables of {app}?"),
        _ => format!("Delete the module {module}?"),
    }
}

/// The body of the dialog of reset. The spec names the tables below it; that needs
/// the migration state (deviation S42).
pub fn reset_body(module: &str) -> String {
    format!(
        "This rolls back every migration of {module} that has run. The tables and columns \
         they created are dropped, with every row in them."
    )
}

/// The line under the body of reset. The spec puts the count of tables before it.
pub fn nothing_outside(module: &str) -> String {
    format!("Nothing outside {module} is touched.")
}

pub fn refresh_body(module: &str) -> String {
    format!(
        "This rolls back every migration of {module} and runs them again. The tables come \
         back empty: every row in them is lost."
    )
}

/// The body of the dialog of fresh: `... of the 7 modules that are on. ...`
/// Not in the spec: the sentence for one module.
pub fn fresh_body(modules_on: usize) -> String {
    let modules = match modules_on {
        1 => "the 1 module that is on".to_owned(),
        n => format!("the {n} modules that are on"),
    };
    format!(
        "This drops every table of the database, not only those of modules, then runs the \
         migrations of the app and of {modules}. Every row in every table is lost. There is no \
         dry run."
    )
}

/// The body of the dialog of delete. `statuses` is the file that keeps the state of
/// the modules, when the app keeps it in a file. Not in the spec: the sentence
/// without that file, and the sentence for one module that uses it.
pub fn delete_body(folder: &str, files: u64, statuses: Option<&str>, users: &[&str]) -> String {
    let line = statuses.map_or(String::new(), |file| format!(" and its line in {file}"));
    let mut body = format!(
        "This removes the folder {folder} with its {}{line}. Its database tables stay.",
        crate::strings::files(files)
    );
    match users {
        [] => {}
        [one] => body.push_str(&format!(" {one} uses it.")),
        _ => body.push_str(&format!(" {} use it.", crate::strings::and_list(users))),
    }
    body
}

/// `12 of the files have changes that are not committed. Git cannot bring those back.`
/// Not in the spec: the sentence for one file.
pub fn uncommitted(n: u32) -> String {
    match n {
        1 => "1 of the files has changes that are not committed. Git cannot bring it back."
            .to_owned(),
        n => format!(
            "{} of the files have changes that are not committed. Git cannot bring those back.",
            crate::strings::count(u64::from(n))
        ),
    }
}

/// The done toasts of S10.5. Not in the spec: the toasts of a run that changed nothing,
/// and of fresh with one module.
pub fn reset_done(module: &str, rolled_back: usize) -> String {
    match rolled_back {
        0 => format!("Reset the migrations of {module}: nothing to roll back."),
        n => format!("Reset the migrations of {module}: {n} rolled back."),
    }
}

pub fn refreshed(module: &str, ran: usize) -> String {
    match ran {
        0 => format!("Refreshed the migrations of {module}: nothing ran again."),
        n => format!("Refreshed the migrations of {module}: {n} ran again."),
    }
}

pub fn dropped_all(app: &str, modules_on: usize) -> String {
    match modules_on {
        1 => format!("Dropped all tables of {app} and migrated 1 module."),
        n => format!("Dropped all tables of {app} and migrated {n} modules."),
    }
}

pub fn deleted_module(module: &str) -> String {
    format!("Deleted the module {module}.")
}

/// `1 seeder`, `5 seeders`
pub fn seeders(n: u32) -> String {
    if n == 1 {
        "1 seeder".to_owned()
    } else {
        format!("{} seeders", crate::strings::count(u64::from(n)))
    }
}

/// Not in the spec: a run of `module:lang --json` that gave no report (design D9).
pub fn translations_not_read(reason: &str) -> String {
    format!(
        "Mortise could not read the translation report: \"{reason}\". Findings show the last \
         report."
    )
}
