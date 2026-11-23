//! What Mortise can run in a module, as data, and what the output of a run says.
//!
//! Nothing here starts a process. [`plan::plan`] gives the words of a command and says
//! whether Mortise asks first.

#[cfg(feature = "run")]
pub(crate) mod output;
pub(crate) mod plan;
