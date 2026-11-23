//! The commands of the package in a real app, through the runner Mortise uses.
//!
//! The test is ignored: it needs PHP, Composer and a Laravel app with
//! `nwidart/laravel-modules` and SQLite. `MORTISE_REAL_APP` names the folder of
//! that app. The job "Real PHP" in `.github/workflows/ci.yml` makes one. Run it with
//! `cargo test -p mortise-core --test real_php -- --ignored --test-threads=1`.
//! It writes into that app and leaves it as it found it. Its module has a name of
//! its own, which it removes also when it fails.
#![cfg(feature = "run")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mortise_core::{
    Action, Catalog, Context, FileOutcome, Filled, GeneratedFile, MODULE_MAKE, MigrationCount,
    Options, PhpSettings, Plan, RunEnd, RunEvent, RunnerKind, Runs, STOP_WAIT, Stream, Target,
    autoload_failed, failed_migration, generated_files, migration_count, read_app,
};

/// The module the test makes and deletes: `MortiseCheck`, the process id and the
/// time, so no other run or person holds the name. Letters and digits only, as
/// the module name rule of the package wants.
fn module() -> &'static str {
    static NAME: OnceLock<String> = OnceLock::new();
    NAME.get_or_init(|| {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        format!("MortiseCheck{}{}", std::process::id(), now.as_millis())
    })
}

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

/// Runs `args` to its end and gives its exit code and the lines of standard
/// output and of standard error, apart.
fn run_split(runs: &Runs, app: &Path, args: &[&str]) -> (i32, Vec<String>, Vec<String>) {
    let args: Vec<String> = args.iter().map(|word| word.to_string()).collect();
    let started = runs
        .start(app, &php(), &args, LIMIT)
        .unwrap_or_else(|error| panic!("{args:?} did not start: {error:?}"));
    let (mut out, mut err) = (Vec::new(), Vec::new());
    for event in started.wait(Instant::now() + LIMIT + Duration::from_secs(10)) {
        match event {
            RunEvent::Line {
                stream: Stream::Out,
                text,
            } => out.push(text),
            RunEvent::Line { text, .. } => err.push(text),
            RunEvent::Cut => panic!("{args:?}: the output passed the limit"),
            RunEvent::Ended(RunEnd::Exited {
                code: Some(code), ..
            }) => return (code, out, err),
            RunEvent::Ended(end) => panic!("{args:?} ended with {end:?}"),
        }
    }
    started.stop();
    panic!("{args:?} did not end");
}

/// The JSON document that a `--json` command wrote: the one line of standard output.
fn document(out: &[String], err: &[String]) -> serde_json::Value {
    assert_eq!(out.len(), 1, "stdout: {out:?}\nstderr: {err:?}");
    serde_json::from_str(&out[0]).unwrap_or_else(|error| panic!("{error}: {}", out[0]))
}

/// Every file under `folder` with its size and a hash of its contents.
fn listing(folder: &Path) -> BTreeMap<PathBuf, (u64, u64)> {
    use std::hash::{Hash, Hasher};
    let mut found = BTreeMap::new();
    let mut open = vec![folder.to_path_buf()];
    while let Some(next) = open.pop() {
        for entry in std::fs::read_dir(next).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                open.push(path);
            } else {
                let bytes = std::fs::read(&path).unwrap();
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                bytes.hash(&mut hasher);
                found.insert(path, (bytes.len() as u64, hasher.finish()));
            }
        }
    }
    found
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Rolls back and deletes [`module`] when it drops, also after a failed assertion.
/// The name is this run's own, so the test makes it before `module:make` runs,
/// and a failed `module:make` is cleaned up too. A module that is not there is
/// left alone.
struct Made<'a> {
    runs: &'a Runs,
    app: &'a Path,
    /// True once the test removed the module itself.
    removed: bool,
}

impl Drop for Made<'_> {
    fn drop(&mut self) {
        if self.removed || !module_dir(self.app).exists() {
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
    app.join("Modules").join(module())
}

/// The words of an action on [`module`], outside production, as the app plans them.
fn action(action: Action) -> Vec<String> {
    let context = Context {
        production: false,
        users_on: &[],
        modules_folder_assumed: false,
    };
    match mortise_core::plan(
        action,
        &Target::Module(module().into()),
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
    snapshot.modules.iter().any(|found| found.name == module())
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
    let module = module();

    assert!(
        !has_module(&app) && !module_dir(&app).exists(),
        "The app already has a module {module}. The test does not change a module it did \
         not make: delete it, or use another app."
    );
    // The name is this run's own: from here on a failure removes the module.
    let mut made = Made {
        runs: &runs,
        app: &app,
        removed: false,
    };

    // `run` fails unless `module:make` exits with 0.
    let lines = run(
        &runs,
        &app,
        &generator(MODULE_MAKE, &[("name", module)], &[], None),
    );
    let files = generated_files(&lines, false);
    let manifest = format!("Modules/{module}/module.json");
    assert!(
        files
            .iter()
            .any(|file| file.path == manifest && file.outcome == FileOutcome::Created),
        "{files:?}"
    );
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
            Some(module),
        ),
    );
    let files = generated_files(&lines, false);
    assert_eq!(files.len(), 2, "{}", lines.join("\n"));
    assert_eq!(
        files[0].path,
        format!("Modules/{module}/app/Models/Widget.php")
    );
    assert!(
        files[1]
            .path
            .starts_with(&format!("Modules/{module}/database/migrations/"))
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

#[test]
#[ignore = "needs PHP and the app in MORTISE_REAL_APP"]
fn json_output_of_the_package_commands() {
    let app = app();
    let runs = Runs::new();
    let module = module();
    assert!(
        !has_module(&app) && !module_dir(&app).exists(),
        "The app already has a module {module}. The test does not change a module it did \
         not make: delete it, or use another app."
    );
    let mut made = Made {
        runs: &runs,
        app: &app,
        removed: false,
    };
    run(
        &runs,
        &app,
        &generator(MODULE_MAKE, &[("name", module)], &[], None),
    );
    let dir = module_dir(&app);

    // A dry run lists the files and writes nothing.
    let before = listing(&dir);
    let (code, out, err) = run_split(
        &runs,
        &app,
        &[
            "module:make-model",
            "Widget",
            module,
            "--migration",
            "--dry-run",
            "--json",
        ],
    );
    let json = document(&out, &err);
    assert_eq!(code, 0, "{err:?}");
    assert_eq!(json["dry_run"], true);
    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 2, "{json}");
    for file in files {
        assert!(file["path"].as_str().unwrap().contains(module), "{file}");
        assert_eq!(
            (&file["action"], &file["exists"]),
            (&"create".into(), &false.into())
        );
        assert!(file.get("result").is_none(), "{file}");
    }
    assert_eq!(listing(&dir), before);

    // The real run lists the same files as created.
    let (code, out, err) = run_split(
        &runs,
        &app,
        &[
            "module:make-model",
            "Widget",
            module,
            "--migration",
            "--json",
        ],
    );
    let json = document(&out, &err);
    assert_eq!(code, 0, "{err:?}");
    assert_eq!(json["dry_run"], false);
    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 2, "{json}");
    for file in files {
        assert_eq!(file["result"], "created", "{file}");
        assert!(
            Path::new(file["path"].as_str().unwrap()).is_file(),
            "{file}"
        );
    }

    // The migration waits, then it has run in batch 1 or later.
    let status = |runs: &Runs| {
        let (code, out, err) = run_split(runs, &app, &["module:migrate-status", module, "--json"]);
        assert_eq!(code, 0, "{err:?}");
        let json = document(&out, &err);
        let migrations = json["modules"][0]["migrations"].as_array().unwrap().clone();
        (json["migration_table"].clone(), migrations)
    };
    let (_, migrations) = status(&runs);
    assert_eq!(migrations.len(), 1, "{migrations:?}");
    assert_eq!(migrations[0]["ran"], false);
    run(&runs, &app, &action(Action::Migrate));
    let (table, migrations) = status(&runs);
    assert_eq!(table, true);
    assert_eq!(migrations[0]["ran"], true);
    assert!(migrations[0]["batch"].as_u64().unwrap() >= 1);
    run(&runs, &app, &action(Action::RollBack));

    // One key of the fr file is missing.
    write(
        &dir.join("lang/en/messages.php"),
        "<?php\n\nreturn ['welcome' => 'Welcome', 'bye' => 'Bye'];\n",
    );
    write(
        &dir.join("lang/fr/messages.php"),
        "<?php\n\nreturn ['welcome' => 'Bienvenue'];\n",
    );
    let (code, out, err) = run_split(&runs, &app, &["module:lang", module, "--json"]);
    let json = document(&out, &err);
    assert_eq!(code, 1, "{err:?}");
    assert_eq!(json["modules"][0]["checked"], true);
    assert_eq!(
        json["modules"][0]["locales"]["fr"]["missing_keys"]["messages.php"],
        serde_json::json!(["bye"])
    );

    // One test passes.
    write(
        &dir.join("tests/Unit/SumTest.php"),
        &format!(
            "<?php\n\nnamespace Modules\\{module}\\Tests\\Unit;\n\nuse PHPUnit\\Framework\\TestCase;\n\n\
             class SumTest extends TestCase\n{{\n    public function test_a_sum(): void\n    {{\n        \
             $this->assertSame(4, 2 + 2);\n    }}\n}}\n"
        ),
    );
    let (code, out, err) = run_split(&runs, &app, &["module:test", module, "--json"]);
    let json = document(&out, &err);
    assert_eq!(code, 0, "{err:?}");
    assert_eq!(json["passed"], 1, "{json}");
    assert_eq!(json["failed"], 0, "{json}");
    assert_eq!(json["modules"][0]["ran"], true);

    run(&runs, &app, &action(Action::DeleteModule));
    made.removed = true;
    assert!(!module_dir(&app).exists());
}
