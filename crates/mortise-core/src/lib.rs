//! Reads a Laravel app that uses `nwidart/laravel-modules` from its files.
//!
//! Nothing here draws, runs PHP or writes into the app folder. The entry point is
//! [`read_app`], which turns a folder into an [`AppSnapshot`]: plain data that an
//! interface, a command line tool or a service can show.

mod app;
mod code;
mod composer;
mod layout;
#[cfg(feature = "watch")]
mod live;
mod module;
mod php;
mod settings;

pub use app::{AppSnapshot, ReadError, read_app, read_app_with};
pub use code::{ALL_CODE, propose_codes, propose_monogram};
pub use composer::Version;
pub use layout::{Layout, LayoutKey};
#[cfg(feature = "watch")]
pub use live::{LiveApp, LiveState};
pub use module::{Manifest, Module, ModuleState};
pub use settings::{AppEntry, Settings, SettingsError};
