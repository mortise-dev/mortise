//! What an action runs on.

/// What an action runs on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// One module, by its name.
    Module(String),
    /// Every module, through `--all`.
    All,
}
