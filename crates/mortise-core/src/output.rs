//! What the output of a run says about migrations.
//!
//! Laravel prints one task line per migration: the name, dots, a time and `DONE`
//! or `FAIL`. The package prints `Running Migration <Name> Module` before the
//! migrations of each module. The reading depends on these words, which Laravel
//! can change.

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

/// The number of migration lines that end with `DONE`.
pub fn migrations_done(lines: &[String]) -> usize {
    lines
        .iter()
        .filter(|line| matches!(migration_line(&plain(line)), Some((_, "DONE"))))
        .count()
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
/// spaces, with a name of the form `dddd_dd_dd_dddddd_`, and ends with `DONE`
/// or `FAIL`. A rollback puts `Rollback: ` before the name.
fn migration_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    let line = line.strip_prefix("Rollback: ").unwrap_or(line);
    let name = line.split(' ').next()?;
    let end = line.rsplit(' ').next()?;
    const SHAPE: &[u8] = b"dddd_dd_dd_dddddd_";
    let dated = name.len() > SHAPE.len()
        && name.bytes().zip(SHAPE).all(|(byte, &shape)| match shape {
            b'd' => byte.is_ascii_digit(),
            _ => byte == shape,
        });
    (dated && matches!(end, "DONE" | "FAIL")).then_some((name, end))
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
        assert_eq!(migrations_done(&lines), 0);
    }

    #[test]
    fn a_successful_migrate_counts_its_migration() {
        let lines = recording("module-migrate-one-migration");
        assert_eq!(migrations_done(&lines), 1);
        assert_eq!(failed_migration(&lines), None);
        assert_eq!(migrations_done(&recording("module-migrate-rollback")), 1);
    }

    #[test]
    fn a_plain_failure_names_no_migration() {
        assert_eq!(failed_migration(&recording("module-failing")), None);
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
