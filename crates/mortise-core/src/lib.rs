//! Reads a Laravel app that uses `nwidart/laravel-modules` from its files.
//!
//! Nothing here draws or writes into the app folder. It runs PHP only when asked,
//! with the `run` feature (see [`test_php`]). The entry point is
//! [`read_app`], which turns a folder into an [`AppSnapshot`]: plain data that an
//! interface, a command line tool or a service can show.

#[cfg(feature = "actuator")]
mod actuator;
mod app;
mod catalog;
mod clover;
mod code;
mod composer;
mod editor;
mod environment;
mod findings;
mod git;
mod kind;
mod layout;
mod links;
#[cfg(feature = "watch")]
mod live;
mod matcher;
mod module;
#[cfg(feature = "run")]
mod output;
mod package;
mod php;
mod rules;
mod run;
#[cfg(feature = "actuator")]
mod runtime;
mod secrets;
mod settings;
mod surface;
#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "actuator")]
pub use actuator::{
    Answer, Component, Info, Route, Runtime, Secret, Status, Task, ask, ask_status,
};
pub use app::{
    AppSnapshot, CoverageReport, ReadError, ReadOptions, read_app, read_app_cached, read_app_env,
    read_app_with,
};
pub use catalog::{
    Action, Argument, Ask, Catalog, Context, Filled, Generator, GeneratorOption, Group,
    MODULE_MAKE, NameRule, OptionSet, Options, Plan, Target, asks_again, class_name_ok,
    module_name_ok, plan, refuses, replace_words, shipped_for, typed_name_matches,
};
pub use clover::Coverage;
pub use code::{ALL_CODE, propose_codes, propose_monogram};
pub use composer::Version;
pub use editor::{Editor, EditorError, editor_arguments, open_in_editor};
pub use environment::{Credentials, Environment};
pub use findings::{
    Check, CheckResult, DEFAULT_COVERAGE_LIMIT, Detail, Finding, FindingState, Findings, Severity,
};
pub use git::{CHURN_DAYS, Churn, Git};
pub use kind::{OTHER, TESTS};
pub use layout::{Layout, LayoutKey, read_layout};
pub use links::{Cycle, Link, Reference, columns};
#[cfg(feature = "watch")]
pub use live::{LiveApp, LiveState};
pub use matcher::fuzzy;
pub use module::{KindCount, Manifest, Module, ModuleState, Progress, ReadCache};
#[cfg(feature = "run")]
pub use output::{
    FailedMigration, FileOutcome, GeneratedFile, MigrationCount, autoload_failed, failed_migration,
    generated_files, migration_count, prohibited,
};
pub use package::{
    DataError, FileAction, FileResult, GeneratorReport, LangModule, LangReport, Locale,
    MigrationRow, MigrationStatus, ModuleMigrations, ModuleStatus, ModuleTests, PackageData,
    RanMigration, ReportedFile, TestReport, document, gives_data, migration_file, with_dry_run,
    with_json,
};
pub use rules::{RULES_FILE, Rules, RulesError, RulesSource};
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
pub use runtime::{
    LiveRuntime, Pace, RuntimeState, module_of, routes_by_module, routes_of, tasks_of,
};
pub use secrets::{Kept, Secrets};
pub use settings::{AppEntry, General, IndexRows, Motion, Settings, SettingsError, ThemeChoice};
pub use surface::{Proposal, ScanCounts, SurfaceEntry, SurfaceKind, SurfaceScan, Typed};
