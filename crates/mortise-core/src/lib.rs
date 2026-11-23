//! Reads a Laravel app that uses `nwidart/laravel-modules` from its files.
//!
//! Nothing here draws or writes into the app folder. It runs PHP only when asked,
//! with the `run` feature (see [`test_php`]). The entry point is
//! [`read_app`], which turns a folder into an [`AppSnapshot`]: plain data that an
//! interface, a command line tool or a service can show.

mod actions;
mod app;
mod boundary;
mod editor;
mod findings;
mod generate;
mod modules;
mod package;
mod palette;
mod php;
mod program;
mod run;
mod runtime;
mod settings;
mod surface;
#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "run")]
pub use actions::output::{
    FailedMigration, MigrationCount, failed_migration, migration_count, prohibited,
};
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
pub use editor::{Editor, EditorError, editor_arguments, open_in_editor};
pub use findings::{
    Check, CheckResult, DEFAULT_COVERAGE_LIMIT, Detail, Finding, FindingState, Findings, Severity,
    Todo,
};
#[cfg(feature = "run")]
pub use generate::cache as catalog_cache;
pub use generate::catalog::{
    Argument, Catalog, Filled, Generator, GeneratorOption, Group, MODULE_MAKE, NameRule,
    class_name_ok, generator_gives_data, model_flag_of, module_name_ok, replace_words,
    shipped_mark,
};
#[cfg(feature = "run")]
pub use generate::output::{
    FileOutcome, GeneratedFile, app_relative, autoload_failed, generated_files, reported_files,
};
pub use modules::clover::Coverage;
pub use modules::code::{ALL_CODE, propose_codes, propose_monogram};
pub use modules::composer::Version;
pub use modules::git::{CHURN_DAYS, Churn, Git};
pub use modules::kind::{OTHER, TESTS};
pub use modules::layout::{Layout, LayoutKey, read_layout};
pub use modules::module::{KindCount, Manifest, Module, ModuleState, Progress, ReadCache};
pub use modules::target::Target;
pub use package::{
    DataError, FileAction, FileResult, GeneratorReport, LangModule, LangReport, Locale,
    MigrationRow, MigrationStatus, ModuleMigrations, ModuleStatus, ModuleTests, PackageData,
    RanMigration, ReportedFile, TestReport, document, gives_data, has_option, migrate_status_args,
    migration_file, migrations_changed_by, whole_document, with_dry_run, with_json,
};
pub use palette::matcher::fuzzy;
#[cfg(feature = "run")]
pub use palette::recent::{Ran, RecentRun, recent_runs};
#[cfg(feature = "run")]
pub use run::ansi;
#[cfg(feature = "run")]
pub use run::history;
#[cfg(feature = "run")]
pub use run::history::OUTPUT_LIMIT;
#[cfg(feature = "run")]
pub use run::{
    CommandLine, Detected, Failure, Read, ReadEnd, Run, RunEnd, RunEvent, Runs, STOP_WAIT,
    StartError, Stream, TEST_LIMIT, TestOutcome, detect, herd_config_dir, stop_asks_first,
    test_php,
};
pub use run::{PhpSettings, RunnerKind};
#[cfg(feature = "actuator")]
pub use runtime::actuator::{
    Answer, Component, Info, Route, Runtime, Secret, Status, Task, ask, ask_status,
};
pub use runtime::environment::{Credentials, Environment};
#[cfg(feature = "actuator")]
pub use runtime::live::{
    LiveRuntime, Pace, RuntimeState, module_of, module_of_task, routes_by_module, routes_of,
    tasks_of,
};
#[cfg(feature = "actuator")]
pub use runtime::modules::{ModuleRuntime, RuntimeByModule, by_module};
pub use runtime::secrets::{Kept, Secrets};
pub use settings::{AppEntry, General, IndexRows, Motion, Settings, SettingsError, ThemeChoice};
pub use surface::{Proposal, ScanCounts, SurfaceEntry, SurfaceKind, SurfaceScan, Typed};
