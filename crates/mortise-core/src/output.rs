//! What the output of a run says about migrations, a refusal and generated files.
//!
//! Laravel prints one task line per migration: the name, dots, a time and `DONE`
//! or `FAIL`. The package prints `Running Migration <Name> Module` before the
//! migrations of each module. A roll back prints `Rollback: <name>` as a task
//! line, and a reset prints it as a line of its own. The reading depends on these
//! words, which Laravel and the package can change.
//!
//! A generator prints `Generating file <path> ... <time> DONE` or `FAIL` for each
//! file. A `FAIL` is followed by an error line `File : <path> already exists.`
//! when the file is there. `module:make` prints `composer dump-autoload failed.`
//! when the autoload step fails.
//!
//! From 13.4.0 on a generator run with `--json` gives its files as a document
//! ([`GeneratorReport`]); [`reported_files`] reads it into the same rows.

use std::collections::BTreeSet;
use std::path::Path;

use crate::package::{FileResult, GeneratorReport};
use crate::run::ansi::plain;

/// The migration at which a run failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedMigration {
    /// The module of the last `Running Migration` line before the failure.
    pub module: Option<String>,
    /// The name of the migration, without `.php`.
    pub migration: String,
}

/// The first migration that failed, or `None` when no migration line failed.
pub fn failed_migration(lines: &[String]) -> Option<FailedMigration> {
    let mut module = None;
    for line in lines {
        let line = plain(line);
        if let Some(name) = running_module(&line) {
            module = Some(name.to_string());
        } else if let Some((migration, "FAIL")) = migration_line(&line) {
            return Some(FailedMigration {
                module,
                migration: migration.to_string(),
            });
        }
    }
    None
}

/// The sentence of Laravel's `Prohibitable` when an app prohibits a command.
const PROHIBITED: &str = "This command is prohibited from running in this environment.";

/// True when the output says that the app prohibits the command. Laravel's warn
/// component prints the sentence after ` WARN `.
pub fn prohibited(lines: &[String]) -> bool {
    lines
        .iter()
        .any(|line| plain(line).trim().ends_with(PROHIBITED))
}

/// The migrations that a run ran and rolled back.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MigrationCount {
    /// Migration lines that end with `DONE`, without `Rollback: `.
    pub ran: usize,
    /// Migrations rolled back, each name once.
    pub rolled_back: usize,
}

/// What the migration lines of a run say it ran and rolled back.
///
/// A rollback that ends with `FAIL` is not counted. A refresh prints its first
/// `Rollback: ` after its own task label, on the same line, so the name is
/// looked for anywhere in the line.
pub fn migration_count(lines: &[String]) -> MigrationCount {
    let mut ran = 0;
    let mut rolled_back = BTreeSet::new();
    for line in lines {
        let line = plain(line);
        if let Some(name) = rolled_back_name(&line) {
            if line.trim_end().rsplit(' ').next() != Some("FAIL") {
                rolled_back.insert(name.to_string());
            }
        } else if let Some((_, "DONE")) = migration_line(&line) {
            ran += 1;
        }
    }
    MigrationCount {
        ran,
        rolled_back: rolled_back.len(),
    }
}

/// What a generator run did with one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOutcome {
    /// The file is new.
    Created,
    /// The file existed and the run, which carried `--force`, wrote it again.
    Replaced,
    /// The file exists and the run did not write it.
    Left,
    /// The file was not written for another reason.
    Failed,
}

/// One file of a generator run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFile {
    /// The path as the output prints it.
    pub path: String,
    pub outcome: FileOutcome,
}

/// The files of a generator run, in output order. `force` is true when the run
/// carried `--force`.
///
/// The exit code does not say what happened to each file: `module:make-model`
/// exits 0 when it leaves its policy.
pub fn generated_files(lines: &[String], force: bool) -> Vec<GeneratedFile> {
    let mut files: Vec<GeneratedFile> = Vec::new();
    for line in lines {
        let line = plain(line);
        if let Some((path, done)) = generating_line(&line) {
            let outcome = match (done, force) {
                (true, true) => FileOutcome::Replaced,
                (true, false) => FileOutcome::Created,
                (false, _) => FileOutcome::Failed,
            };
            files.push(GeneratedFile {
                path: path.to_string(),
                outcome,
            });
        } else if let Some(path) = exists_path(&line) {
            let failed = files
                .iter_mut()
                .rev()
                .find(|file| file.path == path && file.outcome == FileOutcome::Failed);
            match failed {
                Some(file) => file.outcome = FileOutcome::Left,
                None => files.push(GeneratedFile {
                    path: path.to_string(),
                    outcome: FileOutcome::Left,
                }),
            }
        }
    }
    files
}

/// The files of a generator run from its document (P5), in write order. The folder
/// that `module:make --force` deletes is not a file. A path is made relative to the
/// app folder `root`, as the output lines print it, through [`app_relative`] with the
/// modules folder `modules`; a path it cannot place stays as printed.
pub fn reported_files(
    report: &GeneratorReport,
    root: &Path,
    modules: &str,
    names: &[&str],
    folders: &[&str],
) -> Vec<GeneratedFile> {
    report
        .files
        .iter()
        .filter_map(|file| {
            let outcome = match file.result? {
                FileResult::Created => FileOutcome::Created,
                FileResult::Replaced => FileOutcome::Replaced,
                FileResult::Exists => FileOutcome::Left,
                FileResult::Failed => FileOutcome::Failed,
                FileResult::Deleted => return None,
            };
            let path = app_relative(root, modules, names, folders, &file.path)
                .unwrap_or_else(|| file.path.clone());
            Some(GeneratedFile { path, outcome })
        })
        .collect()
}

/// A path the package printed, relative to the app folder `root` with `/` between its
/// parts, or `None` when Mortise cannot place it. The package prints absolute paths
/// with `/` on every platform, from the real folder of the app, so `root` is compared
/// as given and with its links resolved.
///
/// PHP in a container (Sail, Docker) prints the folder of the container, such as
/// `/var/www/html/Modules/Billing/app/Models/Invoice.php`. A path outside `root` is
/// then read from a part `/<modules>/<Module>/` on, where `modules` is the modules
/// folder relative to the app (`Modules`): `Modules/Billing/app/Models/Invoice.php`.
/// The container's folder may hold that name too (`/srv/Modules/shop-api/Modules/...`),
/// and so may the class name (`.../Models/Admin/Modules/Billing/Invoice.php`). The
/// first part counts whose `<Module>` is one of `names`, the modules the run writes
/// in: its own module, or those `module:make` asks for (the package makes them
/// studly: case, `-` and `_` do not count). Else the first part whose `<Module>` is
/// one of `folders`, the folder names of the app's modules, counts, and with none of
/// them, the last part.
pub fn app_relative(
    root: &Path,
    modules: &str,
    names: &[&str],
    folders: &[&str],
    printed: &str,
) -> Option<String> {
    let printed = printed.replace('\\', "/");
    [Some(root.to_path_buf()), root.canonicalize().ok()]
        .into_iter()
        .flatten()
        .find_map(|root| {
            let root = root.to_string_lossy().replace('\\', "/");
            // A resolved path on Windows starts with `\\?\`.
            let root = root.strip_prefix("//?/").unwrap_or(&root);
            let rest = printed.strip_prefix(root.trim_end_matches('/'))?;
            let rest = rest.strip_prefix('/')?;
            (!rest.is_empty()).then(|| rest.to_owned())
        })
        .or_else(|| from_modules(modules, names, folders, &printed))
}

/// The part of `printed` from a `/<modules>/<Module>/` on, when a file follows: the
/// first whose `<Module>` is one of `names`, else the first whose `<Module>` is one of
/// `folders`, else the last.
fn from_modules(modules: &str, names: &[&str], folders: &[&str], printed: &str) -> Option<String> {
    let modules = modules.trim_matches('/');
    if modules.is_empty() {
        return None;
    }
    let part = format!("/{modules}/");
    let found: Vec<(&str, &str)> = printed
        .char_indices()
        .filter(|(at, _)| printed[*at..].starts_with(&part))
        .filter_map(|(at, _)| {
            let rest = &printed[at + 1..];
            let (module, file) = rest[modules.len() + 1..].split_once('/')?;
            (!module.is_empty() && !file.is_empty()).then_some((module, rest))
        })
        .collect();
    let studly = |name: &str| {
        name.chars()
            .filter(|c| *c != '-' && *c != '_')
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>()
    };
    let asked: Vec<String> = names.iter().map(|name| studly(name)).collect();
    let first = |known: &dyn Fn(&str) -> bool| found.iter().find(|(module, _)| known(module));
    first(&|module| asked.contains(&studly(module)))
        .or_else(|| first(&|module| folders.contains(&module)))
        .or(found.last())
        .map(|(_, rest)| (*rest).to_owned())
}

/// True when the output says that `composer dump-autoload` failed.
pub fn autoload_failed(lines: &[String]) -> bool {
    lines.iter().any(|line| {
        plain(line)
            .trim()
            .ends_with("composer dump-autoload failed.")
    })
}

/// The path and the result (`true` for `DONE`) of a `Generating file` line. The
/// path ends before the dots and the time.
fn generating_line(line: &str) -> Option<(&str, bool)> {
    let rest = line.trim().strip_prefix("Generating file ")?;
    let (rest, end) = rest.trim_end().rsplit_once(' ')?;
    let done = match end {
        "DONE" => true,
        "FAIL" => false,
        _ => return None,
    };
    let rest = rest.trim_end();
    let rest = match rest.rsplit_once(' ') {
        Some((before, time)) if is_time(time) => before,
        _ => rest,
    };
    let path = rest.trim_end_matches(['.', ' ']);
    (!path.is_empty()).then_some((path, done))
}

/// A time such as `0.24ms` or `1s`.
fn is_time(word: &str) -> bool {
    word.starts_with(|c: char| c.is_ascii_digit()) && word.ends_with('s')
}

/// The path of a `File : <path> already exists.` line.
fn exists_path(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once("File : ")?;
    let path = rest.trim_end().strip_suffix(" already exists.")?;
    (!path.is_empty()).then_some(path)
}

/// The module of a `Running Migration <Name> Module` line.
fn running_module(line: &str) -> Option<&str> {
    let mut words = line.trim().strip_prefix("Running Migration ")?.split(' ');
    let name = words.next()?;
    (words.next() == Some("Module")).then_some(name)
}

/// The name and the last word of a migration line. The line starts, after
/// spaces, with a migration name, and ends with `DONE` or `FAIL`. A rollback
/// puts `Rollback: ` before the name.
fn migration_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    let line = line.strip_prefix("Rollback: ").unwrap_or(line);
    let name = line.split(' ').next()?;
    let end = line.rsplit(' ').next()?;
    (migration_name(name) && matches!(end, "DONE" | "FAIL")).then_some((name, end))
}

/// The migration name after `Rollback: ` in a line.
fn rolled_back_name(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once("Rollback: ")?;
    let name = rest.split(' ').next()?.trim_end();
    migration_name(name).then_some(name)
}

/// A name of the form `dddd_dd_dd_dddddd_` and more.
fn migration_name(name: &str) -> bool {
    const SHAPE: &[u8] = b"dddd_dd_dd_dddddd_";
    name.len() > SHAPE.len()
        && name.bytes().zip(SHAPE).all(|(byte, &shape)| match shape {
            b'd' => byte.is_ascii_digit(),
            _ => byte == shape,
        })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures");

    /// The lines of the stdout of a recording in `fixtures/output`.
    fn recording(name: &str) -> Vec<String> {
        let path = Path::new(FIXTURES)
            .join("output")
            .join(format!("{name}.json"));
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        json["stdout"]
            .as_str()
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn a_failing_migrate_names_its_migration_and_module() {
        let lines = recording("module-migrate-failing-migration");
        assert_eq!(
            failed_migration(&lines),
            Some(FailedMigration {
                module: Some("Shipping".to_string()),
                migration: "2026_11_22_140500_add_tracking_code_to_shipments_table".to_string(),
            })
        );
        assert_eq!(migration_count(&lines), MigrationCount::default());
    }

    #[test]
    fn a_run_counts_what_ran_and_what_rolled_back() {
        let lines = recording("module-migrate-one-migration");
        assert_eq!(failed_migration(&lines), None);
        // The recording, the migrations that ran, those rolled back.
        let table = [
            ("module-migrate-one-migration", 1, 0),
            // A roll back prints a task line: `Rollback: <name> ... DONE`.
            ("module-migrate-rollback", 0, 1),
            // A reset prints `Rollback: <name>` lines with no `DONE`.
            ("module-migrate-reset", 0, 3),
            // A dry run prints the names with no `DONE`.
            ("module-migrate-reset-pretend", 0, 0),
            // A refresh glues its first `Rollback: ` to its own task label.
            ("module-migrate-refresh", 3, 3),
            // A fresh runs the migrations of the app too.
            ("module-migrate-fresh-all", 8, 0),
        ];
        for (name, ran, rolled_back) in table {
            assert_eq!(
                migration_count(&recording(name)),
                MigrationCount { ran, rolled_back },
                "{name}"
            );
        }
    }

    #[test]
    fn a_plain_failure_names_no_migration() {
        let lines = recording("module-failing");
        assert_eq!(failed_migration(&lines), None);
        assert!(!prohibited(&lines));
    }

    #[test]
    fn a_prohibited_command_reads_as_prohibited() {
        let lines = recording("module-migrate-reset-prohibited");
        assert!(prohibited(&lines));
        assert_eq!(failed_migration(&lines), None);
    }

    fn files(name: &str, force: bool) -> Vec<(String, FileOutcome)> {
        generated_files(&recording(name), force)
            .into_iter()
            .map(|file| (file.path, file.outcome))
            .collect()
    }

    #[test]
    fn a_generator_run_creates_its_files() {
        let created = |path: &str| (path.to_string(), FileOutcome::Created);
        assert_eq!(
            files("module-make-model-migration-policy", false),
            vec![
                created("Modules/Billing/app/Models/PaymentIntent.php"),
                created(
                    "Modules/Billing/database/migrations/2026_11_23_071900_create_payment_intents_table.php"
                ),
                created("Modules/Billing/app/Policies/PaymentIntentPolicy.php"),
            ]
        );
        let module = files("module-make", false);
        assert_eq!(module[0], created("Modules/Blog/module.json"));
        assert!(
            module
                .iter()
                .all(|(_, outcome)| *outcome == FileOutcome::Created)
        );
        assert_eq!(files("module-make-api", false).len(), 16);
    }

    #[test]
    fn a_file_that_exists_is_left() {
        // The model run exits 0 and leaves only its policy.
        let outcomes: Vec<_> = files("module-make-model-policy-exists", false)
            .into_iter()
            .map(|(_, outcome)| outcome)
            .collect();
        assert_eq!(
            outcomes,
            [
                FileOutcome::Created,
                FileOutcome::Created,
                FileOutcome::Left
            ]
        );
        assert_eq!(
            files("module-make-controller-exists", false),
            vec![(
                "Modules/Blog/app/Http/Controllers/PostController.php".to_string(),
                FileOutcome::Left
            )]
        );
    }

    #[test]
    fn a_forced_run_replaces_its_file() {
        assert_eq!(
            files("module-make-policy-force", true),
            vec![(
                "Modules/Billing/app/Policies/InvoicePolicy.php".to_string(),
                FileOutcome::Replaced
            )]
        );
    }

    #[test]
    fn a_report_gives_the_files_relative_to_the_app() {
        let root = Path::new("/tmp/scratch-app");
        let read = |name: &str| {
            let report: GeneratorReport = crate::package::document(&recording(name)).unwrap();
            reported_files(&report, root, "Modules", &[], &[])
                .into_iter()
                .map(|file| (file.path, file.outcome))
                .collect::<Vec<_>>()
        };
        let created = read("module-make-model-json");
        assert_eq!(
            created[0],
            (
                "Modules/Billing/app/Models/PaymentIntent.php".to_string(),
                FileOutcome::Created
            )
        );
        assert!(
            created
                .iter()
                .all(|(_, outcome)| *outcome == FileOutcome::Created)
        );
        // The answer says the policy exists, as the lines do.
        let left = read("module-make-model-policy-exists-json");
        assert_eq!(
            left[2],
            (
                "Modules/Billing/app/Policies/InvoicePolicy.php".to_string(),
                FileOutcome::Left
            )
        );
        assert_eq!(
            app_relative(
                Path::new("/tmp/other"),
                "Modules",
                &[],
                &[],
                "/tmp/scratch-app/x.php"
            ),
            None
        );
        // PHP in a container prints the container's folder: the path is read from
        // the modules folder of the app's layout on.
        let container = |modules: &str, printed: &str| {
            app_relative(
                Path::new("/Users/me/shop"),
                modules,
                &[],
                &["Billing"],
                printed,
            )
        };
        assert_eq!(
            container(
                "Modules",
                "/var/www/html/Modules/Billing/app/Models/Invoice.php"
            )
            .as_deref(),
            Some("Modules/Billing/app/Models/Invoice.php")
        );
        assert_eq!(
            container("src/Domains", "/app/src/Domains/Billing/Invoice.php").as_deref(),
            Some("src/Domains/Billing/Invoice.php")
        );
        assert_eq!(container("Modules", "/var/www/html/Modules/x.php"), None);
        // A container folder that holds the name of the modules folder: the part of a
        // module of the app counts.
        assert_eq!(
            container(
                "Modules",
                "/srv/Modules/shop-api/Modules/Billing/app/Models/Invoice.php"
            )
            .as_deref(),
            Some("Modules/Billing/app/Models/Invoice.php")
        );
        // A new module comes before a known one: `module:make loyalty` placed in a
        // container folder named after Billing.
        assert_eq!(
            app_relative(
                Path::new("/Users/me/shop"),
                "Modules",
                &["loyalty"],
                &["Billing"],
                "/srv/Modules/Billing/Modules/Loyalty/module.json"
            )
            .as_deref(),
            Some("Modules/Loyalty/module.json")
        );
        // A class name that repeats the module's part: the first part of the run's
        // own module counts.
        assert_eq!(
            app_relative(
                Path::new("/Users/me/shop"),
                "Modules",
                &["Billing"],
                &["Billing"],
                "/var/www/html/Modules/Billing/app/Models/Admin/Modules/Billing/Invoice.php"
            )
            .as_deref(),
            Some("Modules/Billing/app/Models/Admin/Modules/Billing/Invoice.php")
        );
    }

    #[test]
    fn a_failed_autoload_is_read_from_its_error_line() {
        assert!(!autoload_failed(&recording("module-make-api")));
        assert!(autoload_failed(&[
            "  \x1b[37;41m ERROR \x1b[39;49m composer dump-autoload failed.  ".to_string()
        ]));
    }
}
