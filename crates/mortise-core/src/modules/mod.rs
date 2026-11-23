//! Reading the modules of an app: each module, its layout, its kinds of file,
//! its Composer version, its coverage, its Git churn and its code.

pub(crate) mod clover;
pub(crate) mod code;
pub(crate) mod composer;
pub(crate) mod git;
pub(crate) mod kind;
pub(crate) mod layout;
pub(crate) mod module;
