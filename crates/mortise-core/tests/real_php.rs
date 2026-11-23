//! The commands of the package in a real app, through the runner Mortise uses.
//!
//! The test is ignored: it needs PHP, Composer and a Laravel app with
//! `nwidart/laravel-modules` and SQLite. `MORTISE_REAL_APP` names the folder of
//! that app. The job "Real PHP" in `.github/workflows/ci.yml` makes one. Run it with
//! `cargo test -p mortise-core --test real_php -- --ignored --test-threads=1`.
//! It writes into that app and leaves it as it found it. It refuses to run when the
//! app already has its module, and removes the module it made also when it fails.
#![cfg(feature = "run")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use mortise_core::{
    Action, Catalog, Context, FileOutcome, Filled, GeneratedFile, MODULE_MAKE, MigrationCount,
    Options, PhpSettings, Plan, RunEnd, RunEvent, RunnerKind, Runs, STOP_WAIT, Target,
    autoload_failed, failed_migration, generated_files, migration_count, read_app,
};

/// The module the test makes and deletes. A name no real app is likely to hold.
const MODULE: &str = "MortiseRealCheck";

/// The longest a command may take. `module:make` runs `composer dump-autoload`.
const LIMIT: Duration = Duration::from_secs(300);

fn app() -> PathBuf {
    match std::env::var_os("MORTISE_REAL_APP") {
        Some(folder) if !folder.is_empty() => PathBuf::from(folder),
        _ => panic!(
            "MORTISE_REAL_APP is not set. Set it to the folder of a Laravel app with \
             nwidart/laravel-modules. The job \"Real PHP\" in ci.yml shows how to make one."
        ),
    }
}

fn php() -> PhpSettings {
    PhpSettings {
        runner: Some(RunnerKind::Custom {
            line: "php artisan".into(),
        }),
        ..PhpSettings::default()
    }
}

/// Runs `args` to its end and gives the lines of both streams, as the app keeps
/// them. Fails unless the command exits with 0.
fn run(runs: &Runs, app: &Path, args: &[String]) -> Vec<String> {
    try_run(runs, app, args).unwrap_or_else(|error| panic!("{error}"))
}

/// [`run`] without a panic, for the clean up. The run is over when it returns.
fn try_run(runs: &Runs, app: &Path, args: &[String]) -> Result<Vec<String>, String> {
    let started = runs
        .start(app, &php(), args, LIMIT)
        .map_err(|error| format!("{args:?} did not start: {error:?}"))?;
    let mut lines = Vec::new();
    for event in started.wait(Instant::now() + LIMIT + Duration::from_secs(10)) {
        match event {
            RunEvent::Line { text, .. } => lines.push(text),
            RunEvent::Cut => return Err(format!("{args:?}: the output passed the limit")),
            RunEvent::Ended(RunEnd::Exited { code: Some(0), .. }) => return Ok(lines),
            RunEvent::Ended(end) => {
                return Err(format!(
                    "{args:?} ended with {end:?}:\n{}",
                    lines.join("\n")
                ));
            }
        }
    }
    started.stop();
    started.wait(Instant::now() + STOP_WAIT);
    Err(format!("{args:?} did not end:\n{}", lines.join("\n")))
}

/// Rolls back and deletes [`MODULE`] when it drops, also after a failed assertion.
/// The test makes it only after its own `module:make` exited with 0 and created
/// the `module.json` of the module.
struct Made<'a> {
    runs: &'a Runs,
    app: &'a Path,
    /// True once the test removed the module itself.
    removed: bool,
}

impl Drop for Made<'_> {
    fn drop(&mut self) {
        if self.removed {
            return;
        }
        for args in [action(Action::RollBack), action(Action::DeleteModule)] {
            if let Err(error) = try_run(self.runs, self.app, &args) {
                eprintln!("The clean up failed: {error}");
            }
        }
    }
}

fn module_dir(app: &Path) -> PathBuf {
    app.join("Modules").join(MODULE)
}

/// The words of an action on [`MODULE`], outside production, as the app plans them.
fn action(action: Action) -> Vec<String> {
    let context = Context {
        production: false,
        users_on: &[],
        modules_folder_assumed: false,
    };
    match mortise_core::plan(
        action,
        &Target::Module(MODULE.into()),
        &Options::default(),
        &context,
    ) {
        Plan::Run { args } | Plan::Ask { args, .. } => args,
        plan => panic!("{action:?}: {plan:?}"),
    }
}

/// The words of a generator form with these arguments and flags.
fn generator(
    name: &str,
    arguments: &[(&str, &str)],
    flags: &[&str],
    module: Option<&str>,
) -> Vec<String> {
    let filled = Filled {
        arguments: arguments
            .iter()
            .map(|(name, text)| (name.to_string(), text.to_string()))
            .collect::<BTreeMap<_, _>>(),
        flags: flags.iter().map(|flag| flag.to_string()).collect(),
        ..Filled::default()
    };
    Catalog::v13_3().get(name).unwrap().words(&filled, module)
}

fn has_module(app: &Path) -> bool {
    let snapshot = read_app(app).unwrap();
    snapshot.modules.iter().any(|module| module.name == MODULE)
}

/// Asserts that every file of a generator run is new and is in the app.
fn all_created(app: &Path, files: &[GeneratedFile]) {
    for file in files {
        assert_eq!(file.outcome, FileOutcome::Created, "{file:?}");
        assert!(app.join(&file.path).is_file(), "{file:?} is not in the app");
    }
}

#[test]
#[ignore = "needs PHP and the app in MORTISE_REAL_APP"]
fn make_generate_migrate_and_delete_a_module() {
    let app = app();
    let runs = Runs::new();

    assert!(
        !has_module(&app) && !module_dir(&app).exists(),
        "The app already has a module {MODULE}. The test does not change a module it did \
         not make: delete it, or use another app."
    );

    // `run` fails unless `module:make` exits with 0.
    let lines = run(
        &runs,
        &app,
        &generator(MODULE_MAKE, &[("name", MODULE)], &[], None),
    );
    let files = generated_files(&lines, false);
    let manifest = format!("Modules/{MODULE}/module.json");
    assert!(
        files
            .iter()
            .any(|file| file.path == manifest && file.outcome == FileOutcome::Created),
        "{files:?}"
    );
    // This run made the module: from here on a failure removes it.
    let mut made = Made {
        runs: &runs,
        app: &app,
        removed: false,
    };
    all_created(&app, &files);
    assert!(!autoload_failed(&lines), "{}", lines.join("\n"));
    assert!(has_module(&app));

    let lines = run(
        &runs,
        &app,
        &generator(
            "module:make-model",
            &[("model", "Widget")],
            &["migration"],
            Some(MODULE),
        ),
    );
    let files = generated_files(&lines, false);
    assert_eq!(files.len(), 2, "{}", lines.join("\n"));
    assert_eq!(
        files[0].path,
        format!("Modules/{MODULE}/app/Models/Widget.php")
    );
    assert!(
        files[1]
            .path
            .starts_with(&format!("Modules/{MODULE}/database/migrations/"))
            && files[1].path.ends_with("_create_widgets_table.php"),
        "{files:?}"
    );
    all_created(&app, &files);

    let lines = run(&runs, &app, &action(Action::Migrate));
    assert_eq!(failed_migration(&lines), None, "{}", lines.join("\n"));
    assert_eq!(
        migration_count(&lines),
        MigrationCount {
            ran: 1,
            rolled_back: 0
        },
        "{}",
        lines.join("\n")
    );

    // The roll back leaves the database as it was, so the test runs again.
    let lines = run(&runs, &app, &action(Action::RollBack));
    assert_eq!(
        migration_count(&lines),
        MigrationCount {
            ran: 0,
            rolled_back: 1
        },
        "{}",
        lines.join("\n")
    );

    let delete = action(Action::DeleteModule);
    assert!(delete.contains(&"--force".to_string()), "{delete:?}");
    run(&runs, &app, &delete);
    made.removed = true;
    assert!(!has_module(&app));
    assert!(!module_dir(&app).exists());
}
