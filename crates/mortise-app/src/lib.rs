//! The Mortise desktop app: the window, the theme and the screens.
//!
//! Everything that is not drawing lives in `mortise_core`.

pub mod actions;
pub mod app;
pub mod apps;
pub mod boundary;
pub mod chrome;
pub mod findings;
pub mod generate;
pub mod index;
#[cfg(target_os = "macos")]
pub mod menu;
pub mod palette;
pub mod rail;
pub mod runner;
pub mod runtime;
pub mod settings;
pub mod strings;
pub mod summary;
pub mod ui;
pub mod workspace;
