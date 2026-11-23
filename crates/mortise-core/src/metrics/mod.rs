//! The metrics of a running app: the text format, the reads kept in memory, the place of a series
//! in a module and the figures per module. Nothing here is written to disk.

// Task 20.4 uses both and removes these two attributes.
#[allow(dead_code)]
pub(crate) mod prometheus;
#[allow(dead_code)]
pub(crate) mod sample;
