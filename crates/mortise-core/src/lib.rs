//! Reads a Laravel app that uses `nwidart/laravel-modules` from its files.
//!
//! Nothing here draws, runs PHP or writes into the app folder. The entry point is
//! [`read_app`], which turns a folder into an [`AppSnapshot`]: plain data that an
//! interface, a command line tool or a service can show.

mod app;
mod code;
mod composer;
mod layout;
mod module;
mod php;

pub use app::{AppSnapshot, ReadError, read_app};
pub use code::{ALL_CODE, propose_codes};
pub use composer::Version;
pub use layout::{Layout, LayoutKey};
pub use module::{Manifest, Module, ModuleState};
