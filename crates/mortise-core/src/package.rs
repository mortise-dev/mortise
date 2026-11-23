//! What the package gives from 13.4.0 on: JSON documents on standard output.
//!
//! A `--json` command prints one document on one line, last. Its other messages go
//! to standard error. The types here read those documents. Unknown keys are ignored,
//! so a later package can add keys.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::catalog::Target;
use crate::composer::Version;
use crate::module::Module;
use crate::php;

/// True when the app's package gives P2 to P8: a released 13.4.0 or later.
///
/// The version must be `MAJOR.MINOR.PATCH` in ASCII digits. A dev build, a branch
/// alias or a pre-release may lack a feature, so it gets false.
pub fn gives_data(version: &Version) -> bool {
    let parts: Vec<&str> = version.full().split('.').collect();
    let [major, minor, patch] = parts.as_slice() else {
        return false;
    };
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    if !(digits(major) && digits(minor) && digits(patch)) {
        return false;
    }
    match (major.parse::<u64>(), minor.parse::<u64>()) {
        (Ok(major), Ok(minor)) => (major, minor) >= (13, 4),
        _ => false,
    }
}

/// Why a run gave no document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataError {
    /// Standard output held no line, or only empty lines.
    Missing,
    /// The last line is not the document: serde's message.
    Malformed(String),
}

/// The last non-empty line of standard output, read as `T`.
///
/// The exit code does not matter: a dry run with a kept file exits 1 with a whole document.
pub fn document<T: DeserializeOwned>(stdout: &[String]) -> Result<T, DataError> {
    let line = stdout
        .iter()
        .rev()
        .map(|line| line.trim())
        .find(|line| !line.is_empty())
        .ok_or(DataError::Missing)?;
    serde_json::from_str(line).map_err(|error| DataError::Malformed(error.to_string()))
}

/// The artisan words with `--json`, before a `--` when the words hold one.
pub fn with_json(words: Vec<String>) -> Vec<String> {
    with_flags(words, &["--json"])
}

/// The artisan words with `--dry-run --json`, before a `--` when the words hold one.
pub fn with_dry_run(words: Vec<String>) -> Vec<String> {
    with_flags(words, &["--dry-run", "--json"])
}

/// The words of a read of the migration state of `target`: `module:migrate-status
/// Checkout --json`, or `--all`. A module name that starts with `-` goes after `--`,
/// as [`crate::plan`] puts it.
pub fn migrate_status_args(target: &Target) -> Vec<String> {
    let mut words = vec!["module:migrate-status".to_string()];
    match target {
        Target::Module(name) if name.starts_with('-') => {
            words.extend(["--".to_string(), name.clone()]);
        }
        Target::Module(name) => words.push(name.clone()),
        Target::All => words.push("--all".to_string()),
    }
    with_json(words)
}

fn with_flags(mut words: Vec<String>, flags: &[&str]) -> Vec<String> {
    let at = words
        .iter()
        .position(|word| word == "--")
        .unwrap_or(words.len());
    words.splice(at..at, flags.iter().map(|flag| flag.to_string()));
    words
}

/// The document of a generator and of `module:make`, for a run and for a dry run.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GeneratorReport {
    /// The artisan command: `module:make-model`.
    pub command: String,
    pub dry_run: bool,
    /// The files in write order, with those of the generators it called.
    pub files: Vec<ReportedFile>,
    /// The error messages, as the package prints them.
    pub errors: Vec<String>,
}

impl GeneratorReport {
    /// True when the errors say that `composer dump-autoload` failed (`module:make`).
    pub fn autoload_failed(&self) -> bool {
        self.errors
            .iter()
            .any(|error| error.trim().ends_with("composer dump-autoload failed."))
    }
}

/// One file of a [`GeneratorReport`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReportedFile {
    /// Absolute, with `/` between the parts.
    pub path: String,
    /// The file was there before the run.
    pub exists: bool,
    pub action: FileAction,
    /// What the run did. A dry run has none.
    #[serde(default)]
    pub result: Option<FileResult>,
}

/// What a generator means to do with a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileAction {
    Create,
    Replace,
    /// The file exists and is not written over.
    Keep,
    /// The folder that `module:make --force` deletes first.
    Delete,
}

/// What a generator did with a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileResult {
    Created,
    Replaced,
    Exists,
    Deleted,
    Failed,
}

/// The document of `module:migrate-status --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MigrationStatus {
    pub modules: Vec<ModuleStatus>,
    /// False when the database has no migrations table. Every migration is then not ran.
    pub migration_table: bool,
}

/// The migrations of one module in a [`MigrationStatus`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModuleStatus {
    pub name: String,
    /// Sorted by name.
    pub migrations: Vec<MigrationRow>,
}

/// One migration of a [`ModuleStatus`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MigrationRow {
    /// The file name without `.php`.
    pub name: String,
    pub ran: bool,
    /// The batch of a ran migration.
    #[serde(default)]
    pub batch: Option<u32>,
}

/// The document of `module:test --json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TestReport {
    pub modules: Vec<ModuleTests>,
    pub passed: Option<u32>,
    pub failed: Option<u32>,
    pub skipped: Option<u32>,
    pub exit_code: Option<i32>,
}

/// The tests of one module in a [`TestReport`]. The counts are `None` when the
/// module has no tests folder or the runner wrote no report.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModuleTests {
    pub name: String,
    /// False for a module without a tests folder.
    pub ran: bool,
    pub passed: Option<u32>,
    pub failed: Option<u32>,
    pub skipped: Option<u32>,
    pub exit_code: Option<i32>,
}

/// The document of `module:lang --json`, and the report Mortise keeps of it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LangReport {
    /// By name once kept in [`PackageData`].
    pub modules: Vec<LangModule>,
}

/// The translations of one module in a [`LangReport`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LangModule {
    pub name: String,
    /// False when the module has fewer than two locales.
    pub checked: bool,
    /// By locale.
    pub locales: BTreeMap<String, Locale>,
}

/// What one locale of a module lacks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locale {
    /// Files of the other locales that this one has not.
    pub missing_files: Vec<String>,
    /// The missing keys, by file.
    pub missing_keys: BTreeMap<String, Vec<String>>,
}

/// What the package said about an app, from the reads the user started.
///
/// It lives in memory only: the database changes outside Mortise.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageData {
    /// The migration state, by module name.
    pub migrations: BTreeMap<String, ModuleMigrations>,
    /// Whether the database has a migrations table, from the last state read.
    pub migration_table: Option<bool>,
    /// The last translation report, by module.
    pub translations: Option<LangReport>,
}

/// The migration state of one module.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleMigrations {
    /// By name.
    pub ran: Vec<RanMigration>,
    /// The migrations that did not run, by name.
    pub waiting: Vec<String>,
    /// The tables the ran migrations create, read from their files by the read of the app.
    pub tables: Vec<String>,
    /// When the state was read, in milliseconds since 1970.
    pub read_ms: i64,
}

/// A migration that ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RanMigration {
    pub name: String,
    pub batch: u32,
}

impl ModuleMigrations {
    /// The ran migrations whose batch is the highest among them. A roll back takes these.
    pub fn last_batch(&self) -> Vec<&RanMigration> {
        let Some(last) = self.ran.iter().map(|ran| ran.batch).max() else {
            return Vec::new();
        };
        self.ran.iter().filter(|ran| ran.batch == last).collect()
    }
}

impl PackageData {
    /// Takes in a state read of `target`, read at `read_ms`. A module of the document
    /// replaces its entry. Under `--all`, a module in `modules_on` that the document
    /// does not hold is removed, because the package lists every module that is on.
    pub fn merge_migrations(
        &mut self,
        status: MigrationStatus,
        target: &Target,
        modules_on: &[&str],
        read_ms: i64,
    ) {
        if *target == Target::All {
            let listed: BTreeSet<&str> = status
                .modules
                .iter()
                .map(|module| module.name.as_str())
                .collect();
            self.migrations.retain(|name, _| {
                listed.contains(name.as_str()) || !modules_on.contains(&name.as_str())
            });
        }
        for module in status.modules {
            let mut entry = ModuleMigrations {
                read_ms,
                ..ModuleMigrations::default()
            };
            for migration in module.migrations {
                if migration.ran {
                    entry.ran.push(RanMigration {
                        name: migration.name,
                        batch: migration.batch.unwrap_or_default(),
                    });
                } else {
                    entry.waiting.push(migration.name);
                }
            }
            entry.ran.sort_by(|a, b| a.name.cmp(&b.name));
            entry.waiting.sort();
            self.migrations.insert(module.name, entry);
        }
        self.migration_table = Some(status.migration_table);
    }

    /// Takes in a translation report. A module of the report replaces its entry, so a
    /// check of one module keeps the others.
    pub fn merge_translations(&mut self, report: LangReport) {
        let kept = self.translations.get_or_insert_default();
        for module in report.modules {
            kept.modules.retain(|old| old.name != module.name);
            kept.modules.push(module);
        }
        kept.modules.sort_by(|a, b| a.name.cmp(&b.name));
    }

    /// This data with the tables of each module's ran migrations read from their files.
    pub(crate) fn with_tables(&self, modules: &[Module]) -> PackageData {
        let mut data = self.clone();
        for (name, state) in &mut data.migrations {
            state.tables.clear();
            let Some(module) = modules.iter().find(|module| &module.name == name) else {
                continue;
            };
            for ran in &state.ran {
                let source = migration_file(&module.dir, &ran.name)
                    .and_then(|path| std::fs::read_to_string(path).ok());
                for table in source
                    .as_deref()
                    .map(php::created_tables)
                    .unwrap_or_default()
                {
                    if !state.tables.contains(&table) {
                        state.tables.push(table);
                    }
                }
            }
        }
        data
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

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures");

    fn version(raw: &str, alias: Option<&str>) -> Version {
        Version {
            raw: raw.into(),
            alias: alias.map(String::from),
        }
    }

    /// The lines of the standard output of a recording in `fixtures/output`.
    fn stdout(name: &str) -> Vec<String> {
        let path = Path::new(FIXTURES)
            .join("output")
            .join(format!("{name}.json"));
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        json["stdout"]
            .as_str()
            .unwrap()
            .lines()
            .map(String::from)
            .collect()
    }

    fn words(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| word.to_string()).collect()
    }

    #[test]
    fn only_a_released_13_4_or_later_gives_data() {
        let cases = [
            (version("v13.4.0", None), true),
            (version("13.4.0", None), true),
            (version("13.3.9", None), false),
            (version("14.0.0", None), true),
            (version("13.4.0-RC1", None), false),
            (version("dev-master", Some("13.x-dev")), false),
        ];
        for (version, gives) in cases {
            assert_eq!(gives_data(&version), gives, "{}", version.raw);
        }
    }

    #[test]
    fn documents_are_read_from_the_last_line_of_standard_output() {
        // A kept file exits 1 with a whole document.
        let dry_run: GeneratorReport =
            document(&stdout("module-make-model-policy-exists-dry-run")).unwrap();
        assert!(dry_run.dry_run);
        assert_eq!(dry_run.files.len(), 3);
        let policy = &dry_run.files[2];
        assert!(policy.exists && policy.path.ends_with("/Policies/InvoicePolicy.php"));
        assert_eq!((policy.action, policy.result), (FileAction::Keep, None));
        assert_eq!(dry_run.errors.len(), 1);

        let run: GeneratorReport =
            document(&stdout("module-make-model-policy-exists-json")).unwrap();
        assert_eq!(run.files[0].result, Some(FileResult::Created));
        assert_eq!(run.files[2].result, Some(FileResult::Exists));

        let status: MigrationStatus = document(&stdout("module-migrate-status-waiting")).unwrap();
        assert!(status.migration_table);
        let checkout = &status.modules[0].migrations;
        assert_eq!((checkout[0].ran, checkout[0].batch), (true, Some(1)));
        assert_eq!((checkout[1].ran, checkout[1].batch), (false, None));

        let tests: TestReport = document(&stdout("module-test-all")).unwrap();
        assert_eq!(
            (tests.passed, tests.failed, tests.exit_code),
            (Some(1), Some(1), Some(1))
        );
        assert_eq!(tests.modules[1].failed, Some(1));

        let lang: LangReport = document(&stdout("module-lang-all-json")).unwrap();
        assert!(!lang.modules[0].checked && lang.modules[0].locales.is_empty());
        assert_eq!(
            lang.modules[1].locales["fr"].missing_keys["messages.php"],
            ["tracking_code", "order_placed"]
        );

        // A line of a broken setup before the document does not matter.
        let lines = words(&["Warning: something", r#"{"modules":[]}"#, "  "]);
        assert_eq!(document::<LangReport>(&lines), Ok(LangReport::default()));
        assert_eq!(
            document::<LangReport>(&words(&["", " "])),
            Err(DataError::Missing)
        );
        assert!(matches!(
            document::<LangReport>(&words(&["Command \"module:lang\" is not defined."])),
            Err(DataError::Malformed(_))
        ));
    }

    #[test]
    fn json_goes_before_a_double_dash() {
        assert_eq!(
            with_json(words(&["module:make-model", "Invoice", "Billing"])),
            words(&["module:make-model", "Invoice", "Billing", "--json"])
        );
        assert_eq!(
            with_dry_run(words(&["module:make", "--", "--force"])),
            words(&["module:make", "--dry-run", "--json", "--", "--force"])
        );
        assert_eq!(
            migrate_status_args(&Target::Module("-Odd".into())),
            words(&["module:migrate-status", "--json", "--", "-Odd"])
        );
    }

    #[test]
    fn a_read_replaces_the_modules_it_holds() {
        let mut data = PackageData::default();
        let all: MigrationStatus = document(&stdout("module-migrate-status-all")).unwrap();
        data.merge_migrations(all, &Target::All, &["Billing", "Checkout", "Loyalty"], 1);
        let checkout = &data.migrations["Checkout"];
        assert_eq!(
            checkout.waiting,
            ["2026_11_23_120000_create_order_items_table"]
        );
        assert_eq!(
            checkout.last_batch()[0].name,
            "2026_11_23_110149_create_orders_table"
        );
        assert_eq!(data.migrations["Billing"].waiting.len(), 2);

        // A read of two modules keeps the third, and says there is no table.
        let two: MigrationStatus = document(&stdout("module-migrate-status-no-table")).unwrap();
        data.merge_migrations(two, &Target::Module("Checkout".into()), &[], 2);
        assert_eq!(data.migration_table, Some(false));
        assert_eq!(data.migrations["Checkout"].ran, []);
        assert_eq!(data.migrations["Checkout"].read_ms, 2);
        assert_eq!(data.migrations["Loyalty"].read_ms, 1);

        // Under --all a module that is on and not listed is gone. One that is off stays.
        let none = MigrationStatus {
            modules: Vec::new(),
            migration_table: true,
        };
        data.merge_migrations(none, &Target::All, &["Checkout", "Loyalty"], 3);
        assert_eq!(data.migrations.keys().collect::<Vec<_>>(), ["Billing"]);

        // A report of one module keeps the others.
        data.merge_translations(document(&stdout("module-lang-all-json")).unwrap());
        let mut one: LangReport = document(&stdout("module-lang-all-json")).unwrap();
        one.modules.retain(|module| module.name == "Checkout");
        one.modules[0].locales.clear();
        data.merge_translations(one);
        let kept = &data.translations.as_ref().unwrap().modules;
        let names: Vec<&str> = kept.iter().map(|module| module.name.as_str()).collect();
        assert_eq!(names, ["Billing", "Checkout", "Loyalty"]);
        assert!(kept[1].locales.is_empty());
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
