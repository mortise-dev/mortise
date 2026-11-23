//! The data of the command palette: fuzzy matching and the recent runs.

pub(crate) mod matcher;
#[cfg(feature = "run")]
pub(crate) mod recent;
