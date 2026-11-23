//! Reads a Laravel app that uses `nwidart/laravel-modules` from its files.
//!
//! Nothing here draws, runs PHP or writes into the app folder. The entry point is
//! [`read_app`], which turns a folder into an [`AppSnapshot`]: plain data that an
//! interface, a command line tool or a service can show.

#[cfg(feature = "actuator")]
mod actuator;
mod app;
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
    AppSnapshot, CoverageReport, ReadError, ReadOptions, read_app, read_app_cached, read_app_with,
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
pub use layout::{Layout, LayoutKey};
pub use links::{Cycle, Link, Reference, columns};
#[cfg(feature = "watch")]
pub use live::{LiveApp, LiveState};
pub use matcher::fuzzy;
pub use module::{KindCount, Manifest, Module, ModuleState, Progress, ReadCache};
pub use rules::{RULES_FILE, Rules, RulesError, RulesSource};
#[cfg(feature = "run")]
pub use run::{Detected, detect, herd_config_dir};
#[cfg(feature = "actuator")]
pub use runtime::{
    LiveRuntime, Pace, RuntimeState, module_of, routes_by_module, routes_of, tasks_of,
};
pub use secrets::{Kept, Secrets};
pub use settings::{AppEntry, General, IndexRows, Motion, Settings, SettingsError, ThemeChoice};
pub use surface::{Proposal, ScanCounts, SurfaceEntry, SurfaceKind, SurfaceScan, Typed};
