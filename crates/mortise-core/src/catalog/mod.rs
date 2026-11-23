//! What Mortise can run in an app, as data: the actions of a module and the
//! generators of the package.
//!
//! Nothing here starts a process. [`plan`] gives the words of a command and says
//! whether Mortise asks first. [`Generator::words`] gives the words of a generator.

mod actions;
#[cfg(feature = "run")]
pub mod cache;
mod generators;

pub use actions::{
    Action, Ask, Context, OptionSet, Options, Plan, Target, asks_again, plan, refuses,
    typed_name_matches,
};
pub use generators::{
    Argument, Catalog, Filled, Generator, GeneratorOption, Group, MODULE_MAKE, NameRule,
    class_name_ok, module_name_ok, replace_words, shipped_mark,
};
