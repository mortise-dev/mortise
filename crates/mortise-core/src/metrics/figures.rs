//! The figures of the plain table: requests and jobs per minute for each module.

use std::collections::HashMap;
use std::time::Duration;

use crate::app::AppSnapshot;
use crate::metrics::place::Placing;
use crate::metrics::sample::{Change, Subject};

/// The span of the table of milestone 20.
pub const LAST_MINUTE: Duration = Duration::from_secs(60);

/// What one row of the table counts, per minute.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rates {
    pub requests_per_minute: f64,
    pub jobs_per_minute: f64,
}

/// One module of the app and its rates.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleFigures {
    pub module: String,
    pub rates: Rates,
}

/// The rates per module over a span.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Figures {
    /// The time the rates cover. `None` before two reads of one run lie in the span.
    pub covered: Option<Duration>,
    /// One row per module of the app: most requests first, then by name.
    pub modules: Vec<ModuleFigures>,
    /// "Not in a module".
    pub outside: Rates,
}

impl Figures {
    /// Places what changed over a span and works out the rates. Every module of `app` has a row,
    /// also with no change.
    pub(crate) fn of(app: &AppSnapshot, placing: &Placing<'_>, change: Option<Change<'_>>) -> Self {
        let mut modules: Vec<ModuleFigures> = app
            .modules
            .iter()
            .map(|module| ModuleFigures {
                module: module.name.clone(),
                rates: Rates::default(),
            })
            .collect();
        let mut outside = Rates::default();
        let covered = change.map(|change| {
            let row: HashMap<&str, usize> = app
                .modules
                .iter()
                .enumerate()
                .map(|(row, module)| (module.name.as_str(), row))
                .collect();
            let per_minute = LAST_MINUTE.as_secs_f64() / change.covered.as_secs_f64();
            for (subject, cell) in &change.cells {
                let rates = match placing.place(subject).and_then(|name| row.get(name)) {
                    Some(&row) => &mut modules[row].rates,
                    None => &mut outside,
                };
                match subject {
                    Subject::Request { .. } => rates.requests_per_minute += cell.count * per_minute,
                    Subject::Job { .. } => rates.jobs_per_minute += cell.count * per_minute,
                    Subject::Database { .. } => {}
                }
            }
            change.covered
        });
        modules.sort_by(|a, b| {
            b.rates
                .requests_per_minute
                .total_cmp(&a.rates.requests_per_minute)
                .then_with(|| a.module.cmp(&b.module))
        });
        Self {
            covered,
            modules,
            outside,
        }
    }
}
