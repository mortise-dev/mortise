//! Reads a Laravel app that uses `nwidart/laravel-modules` from its files.
//!
//! Nothing here draws or writes into the app folder. It runs PHP only when asked,
//! with the `run` feature (see [`test_php`]). The entry point is
//! [`read_app`], which turns a folder into an [`AppSnapshot`]: plain data that an
//! interface, a command line tool or a service can show.

mod actions;
mod app;
mod boundary;
mod catalog;
mod editor;
mod findings;
mod matcher;
mod modules;
#[cfg(feature = "run")]
mod output;
mod package;
mod php;
mod program;
mod run;
mod runtime;
mod settings;
mod surface;
#[cfg(feature = "testing")]
pub mod testing;

pub use actions::plan::{
    Action, Ask, Context, OptionSet, Options, Plan, asks_again, plan, refuses, typed_name_matches,
};
#[cfg(feature = "watch")]
pub use app::live::{LiveApp, LiveState};
pub use app::{
    AppSnapshot, CoverageReport, ReadError, ReadOptions, read_app, read_app_cached, read_app_env,
    read_app_with,
};
pub use boundary::links::{Cycle, Link, Reference, columns};
pub use boundary::rules::{RULES_FILE, Rules, RulesError, RulesSource};
#[cfg(feature = "run")]
pub use catalog::cache as catalog_cache;
pub use catalog::{
    Argument, Catalog, Filled, Generator, GeneratorOption, Group, MODULE_MAKE, NameRule,
    class_name_ok, model_flag_of, module_name_ok, replace_words, shipped_mark,
};
pub use editor::{Editor, EditorError, editor_arguments, open_in_editor};
pub use findings::{
    Check, CheckResult, DEFAULT_COVERAGE_LIMIT, Detail, Finding, FindingState, Findings, Severity,
    Todo,
};
pub use matcher::fuzzy;
pub use modules::clover::Coverage;
pub use modules::code::{ALL_CODE, propose_codes, propose_monogram};
pub use modules::composer::Version;
pub use modules::git::{CHURN_DAYS, Churn, Git};
pub use modules::kind::{OTHER, TESTS};
pub use modules::layout::{Layout, LayoutKey, read_layout};
pub use modules::module::{KindCount, Manifest, Module, ModuleState, Progress, ReadCache};
pub use modules::target::Target;
#[cfg(feature = "run")]
pub use output::{
    FailedMigration, FileOutcome, GeneratedFile, MigrationCount, app_relative, autoload_failed,
    failed_migration, generated_files, migration_count, prohibited, reported_files,
};
pub use package::{
    DataError, FileAction, FileResult, GeneratorReport, LangModule, LangReport, Locale,
    MigrationRow, MigrationStatus, ModuleMigrations, ModuleStatus, ModuleTests, PackageData,
    RanMigration, ReportedFile, TestReport, document, generator_gives_data, gives_data, has_option,
    migrate_status_args, migration_file, migrations_changed_by, whole_document, with_dry_run,
    with_json,
};
#[cfg(feature = "run")]
pub use run::ansi;
#[cfg(feature = "run")]
pub use run::history;
#[cfg(feature = "run")]
pub use run::history::OUTPUT_LIMIT;
#[cfg(feature = "run")]
pub use run::{
    CommandLine, Detected, Failure, Ran, Read, ReadEnd, RecentRun, Run, RunEnd, RunEvent, Runs,
    STOP_WAIT, StartError, Stream, TEST_LIMIT, TestOutcome, detect, herd_config_dir, recent_runs,
    stop_asks_first, test_php,
};
pub use run::{PhpSettings, RunnerKind};
#[cfg(feature = "actuator")]
pub use runtime::actuator::{
    Answer, Component, Info, Route, Runtime, Secret, Status, Task, ask, ask_status,
};
pub use runtime::environment::{Credentials, Environment};
#[cfg(feature = "actuator")]
pub use runtime::live::{
    LiveRuntime, Pace, RuntimeState, module_of, routes_by_module, routes_of, tasks_of,
};
pub use runtime::secrets::{Kept, Secrets};
pub use settings::{AppEntry, General, IndexRows, Motion, Settings, SettingsError, ThemeChoice};
pub use surface::{Proposal, ScanCounts, SurfaceEntry, SurfaceKind, SurfaceScan, Typed};
