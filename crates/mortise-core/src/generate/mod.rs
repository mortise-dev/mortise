//! What Mortise can run in an app, as data: the generators of the package, and what
//! the output of a generator run says.
//!
//! Nothing here starts a process. [`catalog::Generator::words`] gives the words of a
//! generator.

#[cfg(feature = "run")]
pub mod cache;
pub(crate) mod catalog;
#[cfg(feature = "run")]
pub(crate) mod output;
