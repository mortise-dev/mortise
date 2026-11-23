//! What the output of a run says about migrations and about a refusal.
//!
//! Laravel prints one task line per migration: the name, dots, a time and `DONE`
//! or `FAIL`. The package prints `Running Migration <Name> Module` before the
//! migrations of each module. A roll back prints `Rollback: <name>` as a task
//! line, and a reset prints it as a line of its own. The reading depends on these
//! words, which Laravel and the package can change.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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

/// The file of a migration in the default folder of the package,
/// `database/migrations`, when it exists.
pub fn migration_file(module_dir: &Path, migration: &str) -> Option<PathBuf> {
    let path = module_dir
        .join("database")
        .join("migrations")
        .join(format!("{migration}.php"));
    path.is_file().then_some(path)
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

    #[test]
    fn a_migration_file_is_found_only_when_it_exists() {
        let billing = Path::new(FIXTURES).join("shop-api/Modules/Billing");
        let name = "2026_01_10_000000_create_invoices_table";
        assert_eq!(
            migration_file(&billing, name),
            Some(
                billing
                    .join("database/migrations")
                    .join(format!("{name}.php"))
            )
        );
        assert_eq!(
            migration_file(&billing, "2026_01_11_000000_create_refunds_table"),
            None
        );
    }
}
