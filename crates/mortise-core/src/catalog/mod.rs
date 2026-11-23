//! What Mortise can run in an app, as data: the generators of the package.
//!
//! Nothing here starts a process. [`Generator::words`] gives the words of a generator.

#[cfg(feature = "run")]
pub mod cache;
mod generators;

pub use generators::{
    Argument, Catalog, Filled, Generator, GeneratorOption, Group, MODULE_MAKE, NameRule,
    class_name_ok, model_flag_of, module_name_ok, replace_words, shipped_mark,
};
