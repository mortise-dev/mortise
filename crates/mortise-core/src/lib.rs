//! Reads a Laravel app that uses `nwidart/laravel-modules` from its files.
//!
//! Nothing here draws, runs PHP or writes into the app folder. The entry point is
//! [`read_app`], which turns a folder into an [`AppSnapshot`]: plain data that an
//! interface, a command line tool or a service can show.

mod app;
mod clover;
mod code;
mod composer;
mod editor;
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
mod settings;

pub use app::{
    AppSnapshot, CoverageReport, ReadError, ReadOptions, read_app, read_app_cached, read_app_with,
};
pub use clover::Coverage;
pub use code::{ALL_CODE, propose_codes, propose_monogram};
pub use composer::Version;
pub use editor::{Editor, EditorError, editor_arguments, open_in_editor};
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
pub use module::{KindCount, Manifest, Module, ModuleState, ReadCache};
pub use rules::{RULES_FILE, Rules, RulesError, RulesSource};
pub use settings::{AppEntry, General, IndexRows, Motion, Settings, SettingsError, ThemeChoice};
