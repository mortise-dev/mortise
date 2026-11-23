//! What Mortise can run in an app, as data: the actions of a module.
//!
//! Nothing here starts a process. [`plan`] gives the words of a command and says
//! whether Mortise asks first.

mod actions;

pub use actions::{Action, Ask, Context, OptionSet, Options, Plan, Target, plan};
