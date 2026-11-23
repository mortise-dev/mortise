//! What the package gives from 13.4.0 on: JSON documents on standard output.
//!
//! A `--json` command prints one document on one line, last. Its other messages go
//! to standard error. The types here read those documents. Unknown keys are ignored,
//! so a later package can add keys.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, Target};
use crate::modules::composer::Version;
use crate::modules::module::Module;
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

/// True when the package ships the generator named `generator`: it is in the
/// catalog of 13.4.0 ([`Catalog::v13_4`]), `module:make` among them.
///
/// The package gives `--dry-run` and `--json` their meaning only for the options it
/// adds itself (`PlansGeneratorRun::addGeneratorRunOptions`). An app's own generator
/// may declare an option of that name and write its files anyway.
pub(crate) fn ships(generator: &str) -> bool {
    Catalog::v13_4().get(generator).is_some()
}

/// True when Mortise lists the files of `generator` by a dry run and reads the
/// result of its run from its answer (design D5, D6): the package gives data
/// ([`gives_data`]) and ships the generator ([`ships`]). An app's own generator
/// runs as before 13.4.
pub fn generator_gives_data(version: &Version, generator: &str) -> bool {
    gives_data(version) && ships(generator)
}

/// Why a run gave no document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataError {
    /// Standard output held no line, or only empty lines.
    Missing,
    /// The last line is not the document: serde's message.
    Malformed(String),
    /// The last line was longer than a line may be and only its start was kept, or
    /// the output passed its limit. A cut document is never read in part.
    Cut,
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

/// [`document`], unless the last non-empty line of `stdout` was cut (`cut`): then
/// [`DataError::Cut`], since the start of a line may parse as a whole document.
pub fn whole_document<T: DeserializeOwned>(stdout: &[String], cut: bool) -> Result<T, DataError> {
    if cut {
        return Err(DataError::Cut);
    }
    document(stdout)
}

/// True when the artisan words hold `option` before a `--`. A word after `--` is a
/// name, such as a module named `--json`.
pub fn has_option(words: &[String], option: &str) -> bool {
    words
        .iter()
        .take_while(|word| *word != "--")
        .any(|word| word == option)
}

/// The artisan words with `--json`, before a `--` when the words hold one. Words
/// that hold it already stay as they are.
pub fn with_json(words: Vec<String>) -> Vec<String> {
    with_flags(words, &["--json"])
}

/// The artisan words with `--dry-run --json`, before a `--` when the words hold one.
/// A flag the words hold already is not added again.
pub fn with_dry_run(words: Vec<String>) -> Vec<String> {
    with_flags(words, &["--dry-run", "--json"])
}

/// The words of a read of the migration state of `target` on `database`:
/// `module:migrate-status Checkout --database=testing --json`, or `--all`. An empty
/// `database` is the default connection and is left out. A module name that starts
/// with `-` goes after `--`, as [`crate::plan`] puts it.
pub fn migrate_status_args(target: &Target, database: &str) -> Vec<String> {
    let mut words = vec!["module:migrate-status".to_string()];
    let mut last = None;
    match target {
        Target::Module(name) if name.starts_with('-') => last = Some(name.clone()),
        Target::Module(name) => words.push(name.clone()),
        Target::All => words.push("--all".to_string()),
    }
    let database = database.trim();
    if !database.is_empty() {
        words.push(format!("--database={database}"));
    }
    if let Some(name) = last {
        words.extend(["--".to_string(), name]);
    }
    with_json(words)
}

/// The migration state that a run of the artisan words `args` may have changed, when
/// it ends, whatever its exit code (design D7): a migrate, roll back, reset, refresh
/// or fresh that is not a dry run (`--pretend`). Every module for `--all`, for a fresh
/// or for words that name no module; else the module the words name, the second word
/// or the word after `--`. `None` for other words.
pub fn migrations_changed_by(args: &[String]) -> Option<Target> {
    const CHANGE: [&str; 5] = [
        "module:migrate",
        "module:migrate-rollback",
        "module:migrate-reset",
        "module:migrate-refresh",
        "module:migrate-fresh",
    ];
    let command = args
        .first()
        .filter(|word| CHANGE.contains(&word.as_str()))?;
    if has_option(args, "--pretend") {
        return None;
    }
    let at = args.iter().position(|word| word == "--");
    let options = &args[..at.unwrap_or(args.len())];
    let module = match at {
        Some(at) => args.get(at + 1),
        None => options.get(1).filter(|word| !word.starts_with('-')),
    };
    match module {
        Some(name) if command != "module:migrate-fresh" && !has_option(args, "--all") => {
            Some(Target::Module(name.clone()))
        }
        _ => Some(Target::All),
    }
}

fn with_flags(mut words: Vec<String>, flags: &[&str]) -> Vec<String> {
    let at = words
        .iter()
        .position(|word| word == "--")
        .unwrap_or(words.len());
    let added: Vec<String> = flags
        .iter()
        .filter(|flag| !has_option(&words, flag))
        .map(|flag| flag.to_string())
        .collect();
    words.splice(at..at, added);
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
    /// The migration state, by module name, of [`Self::migration_connection`].
    pub migrations: BTreeMap<String, ModuleMigrations>,
    /// The connection the migration state was read for (`--database`). Empty for
    /// the default connection.
    pub migration_connection: String,
    /// The modules that were on at the last read of every module (`--all`), while
    /// that read holds: nothing was forgotten after it. Such a module with no entry
    /// was not in its answer. Empty without such a read.
    pub migrations_of_all: BTreeSet<String>,
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
    /// The migration state when it was read for `connection` (empty for the default).
    /// A state of another connection says nothing about this one.
    pub fn migrations_for(&self, connection: &str) -> Option<&BTreeMap<String, ModuleMigrations>> {
        (self.migration_connection == connection.trim()).then_some(&self.migrations)
    }

    /// True when the state on `connection` says something of every module in `modules_on`:
    /// each has an entry, or was on at a read of every module that holds since (design D7).
    /// A count over the modules that are on is then whole.
    pub fn migrations_of_every(&self, connection: &str, modules_on: &[&str]) -> bool {
        self.migrations_for(connection).is_some_and(|migrations| {
            modules_on.iter().all(|name| {
                migrations.contains_key(*name) || self.migrations_of_all.contains(*name)
            })
        })
    }

    /// Forgets the migration state of `target`, whatever connection it was read for:
    /// every module for [`Target::All`], else the module's entry, by name in any case.
    /// A "no migrations table" answer goes too: the run may have made the table.
    /// A read of every module no longer covers them all. True when something was forgotten.
    pub fn forget_migrations(&mut self, target: &Target) -> bool {
        let before = (
            self.migrations.len(),
            self.migration_table,
            self.migrations_of_all.len(),
        );
        match target {
            Target::All => self.migrations.clear(),
            Target::Module(name) => self
                .migrations
                .retain(|module, _| !module.eq_ignore_ascii_case(name)),
        }
        if self.migration_table == Some(false) {
            self.migration_table = None;
        }
        self.migrations_of_all.clear();
        before
            != (
                self.migrations.len(),
                self.migration_table,
                self.migrations_of_all.len(),
            )
    }

    /// Takes in a state read of `target` on `connection`, read at `read_ms`. A read of
    /// another connection than the state's replaces the whole state. A module of the
    /// document replaces its entry. Under `--all`, a module in `modules_on` that the
    /// document does not hold is removed, because the package lists every module that is on.
    pub fn merge_migrations(
        &mut self,
        status: MigrationStatus,
        target: &Target,
        connection: &str,
        modules_on: &[&str],
        read_ms: i64,
    ) {
        let connection = connection.trim();
        if self.migration_connection != connection {
            self.migrations.clear();
            self.migration_connection = connection.to_string();
            self.migrations_of_all.clear();
        }
        if *target == Target::All {
            self.migrations_of_all = modules_on.iter().map(|name| name.to_string()).collect();
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

        // Only a generator the package ships gets the dry run and the answer: an
        // app's own may declare `--dry-run` and write anyway.
        let v13_4 = version("13.4.0", None);
        assert!(generator_gives_data(&v13_4, "module:make"));
        assert!(generator_gives_data(&v13_4, "module:make-health-indicator"));
        assert!(!generator_gives_data(&v13_4, "app:make-widget"));
        assert!(!generator_gives_data(&v13_4, "module:make-widget"));
        assert!(!generator_gives_data(
            &version("13.3.0", None),
            "module:make-model"
        ));
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
            migrate_status_args(&Target::Module("-Odd".into()), " testing "),
            words(&[
                "module:migrate-status",
                "--database=testing",
                "--json",
                "--",
                "-Odd"
            ])
        );
        // Once only, and a module named `--json` after `--` is not the option.
        assert_eq!(
            with_json(words(&["module:make-model", "Invoice", "--json"])),
            words(&["module:make-model", "Invoice", "--json"])
        );
        assert_eq!(
            with_json(words(&["module:make-model", "Invoice", "--", "--json"])),
            words(&["module:make-model", "Invoice", "--json", "--", "--json"])
        );
    }

    #[test]
    fn a_migration_command_changes_the_state_of_its_target() {
        let changed = |line: &[&str]| migrations_changed_by(&words(line));
        let checkout = Some(Target::Module("Checkout".into()));
        assert_eq!(
            changed(&["module:migrate", "Checkout", "--database=testing"]),
            checkout
        );
        assert_eq!(
            changed(&["module:migrate-rollback", "--", "Checkout"]),
            checkout
        );
        assert_eq!(
            changed(&["module:migrate-refresh", "--all"]),
            Some(Target::All)
        );
        assert_eq!(
            changed(&["module:migrate-fresh", "--force"]),
            Some(Target::All)
        );
        assert_eq!(
            changed(&["module:migrate-reset", "--force"]),
            Some(Target::All)
        );
        assert_eq!(changed(&["module:migrate", "Checkout", "--pretend"]), None);
        assert_eq!(changed(&["module:migrate-status", "Checkout"]), None);
        assert_eq!(changed(&["module:seed", "Checkout"]), None);
    }

    #[test]
    fn a_read_replaces_the_modules_it_holds() {
        let mut data = PackageData::default();
        let all: MigrationStatus = document(&stdout("module-migrate-status-all")).unwrap();
        data.merge_migrations(
            all,
            &Target::All,
            "",
            &["Billing", "Checkout", "Loyalty", "Shipping"],
            1,
        );
        // The read of every module covers a module that was on and is not in its answer,
        // not one turned on after it.
        assert!(data.migrations_of_every("", &["Checkout", "Shipping"]));
        assert!(!data.migrations_of_every("", &["Checkout", "Catalog"]));
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
        data.merge_migrations(two, &Target::Module("Checkout".into()), "", &[], 2);
        assert_eq!(data.migration_table, Some(false));
        assert_eq!(data.migrations["Checkout"].ran, []);
        assert_eq!(data.migrations["Checkout"].read_ms, 2);
        assert_eq!(data.migrations["Loyalty"].read_ms, 1);

        // Under --all a module that is on and not listed is gone. One that is off stays.
        let none = MigrationStatus {
            modules: Vec::new(),
            migration_table: true,
        };
        data.merge_migrations(none.clone(), &Target::All, "", &["Checkout", "Loyalty"], 3);
        assert_eq!(data.migrations.keys().collect::<Vec<_>>(), ["Billing"]);

        // A run forgets its module whatever the connection, and a "no table" answer.
        data.migration_table = Some(false);
        assert!(data.forget_migrations(&Target::Module("billing".into())));
        assert!(!data.migrations_of_every("", &["Catalog"]));
        assert!(data.migrations.is_empty() && data.migration_table.is_none());
        assert!(!data.forget_migrations(&Target::All));

        // The state is of one connection: a read of another replaces it all.
        assert!(data.migrations_for("testing").is_none());
        data.merge_migrations(none, &Target::Module("Checkout".into()), "testing", &[], 4);
        assert!(data.migrations_for("").is_none());
        assert_eq!(data.migrations_for(" testing").map(BTreeMap::len), Some(0));

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
