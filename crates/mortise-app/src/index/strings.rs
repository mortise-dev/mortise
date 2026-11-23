//! Interface text of the module index (section 10 of the specification).

pub const SORTED_BY_NAME: &str = "Sorted: by name";

/// `38 files, turned off`
pub fn files_turned_off(n: u64) -> String {
    format!("{}, turned off", crate::strings::files(n))
}

/// `7 of 7 on`
pub fn of_on(on: usize, total: usize) -> String {
    format!("{on} of {total} on")
}

pub const FILTER_MODULES: &str = "Filter modules";

/// `84 files, 54% covered`
pub fn files_covered(n: u64, percent: Option<u8>) -> String {
    match percent {
        Some(percent) => format!("{}, {percent}% covered", crate::strings::files(n)),
        None => crate::strings::files(n),
    }
}

/// `1 of 7 modules`
pub fn of_modules(shown: usize, total: usize) -> String {
    format!("{shown} of {total} modules")
}

/// `No module matches "paym". Press Esc to clear the filter.`
pub fn no_module_matches(filter: &str) -> String {
    format!("No module matches \"{filter}\". Press Esc to clear the filter.")
}
pub const SORTED_PROBLEMS_FIRST: &str = "Sorted: problems first";
pub const CHANGE_THE_ORDER: &str = "Change the order";
pub const NEEDS_ATTENTION: &str = "Needs attention";
