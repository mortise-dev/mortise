//! The Mortise desktop app: the window, the theme and the screens.
//!
//! Everything that is not drawing lives in `mortise_core`.

pub mod actions_view;
pub mod add;
pub mod app;
pub mod boundary;
pub mod chrome;
pub mod dialog;
pub mod draw;
pub mod environment_form;
pub mod findings_view;
pub mod fonts;
pub mod index;
#[cfg(target_os = "macos")]
pub mod menu;
pub mod palette;
pub mod php_settings;
pub mod rail;
pub mod runner_view;
pub mod runtime_view;
pub mod settings_view;
pub mod strings;
pub mod summary;
pub mod surface_view;
pub mod theme;
pub mod toast;
pub mod workspace;
