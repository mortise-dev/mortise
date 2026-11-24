//! The metrics of a running app: the text format, the reads kept in memory, the place of a series
//! in a module and the figures per module. Nothing here is written to disk.

pub(crate) mod figures;
pub(crate) mod place;
pub(crate) mod prometheus;
pub(crate) mod sample;
