//! How PHP runs for an app: the kind of runner, the command line, and starting it.
//!
//! A command line is a program and a list of arguments. It never goes through a shell.

#[cfg(feature = "run")]
mod detect;
#[cfg(feature = "run")]
pub use detect::{Detected, detect, herd_config_dir};
pub(crate) mod process;
