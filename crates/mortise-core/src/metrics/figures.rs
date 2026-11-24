//! The figures of the Metrics tab: for each row, the rates over a range and the line of requests per
//! minute. A row is a module of the app, or a route of one module.

use std::collections::HashMap;
use std::time::Duration;

use crate::app::AppSnapshot;
use crate::metrics::place::Placing;
use crate::metrics::sample::{Cell, MetricsStore, Step, Subject};

/// The time the figures of the tab cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Range {
    FifteenMinutes,
    #[default]
    Hour,
    Day,
}

impl Range {
    pub const ALL: [Range; 3] = [Range::FifteenMinutes, Range::Hour, Range::Day];

    pub fn span(self) -> Duration {
        let minutes = match self {
            Range::FifteenMinutes => 15,
            Range::Hour => 60,
            Range::Day => 24 * 60,
        };
        Duration::from_secs(minutes * 60)
    }
}

/// What the rows of the table are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope<'a> {
    /// One row per module of the app.
    All,
    /// One row per route of the module with this name.
    Module(&'a str),
}

/// The 95% figure: a bucket bound, in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Under {
    /// The smallest finite bound with at least 95% of requests at or under it.
    Bound(f64),
    /// More than 5% of requests are over the largest finite bound, which this holds.
    Above(f64),
}

/// What one row of the table counts over the range.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rates {
    pub requests_per_minute: f64,
    pub jobs_per_minute: f64,
    /// `None` with no request in the range, or no bucket bound kept.
    pub under_95: Option<Under>,
    /// The share, 0 to 1, of requests answered with a 5xx status. `None` with no request.
    pub errors: Option<f64>,
    /// Database time against request time. `None` with no request time, for a route, and when
    /// `Figures::database_known` is false.
    pub database: Option<f64>,
}

/// What a row is about.
/// Rows of one kind sort by name: a route by its route, then its method.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RowOf {
    Module(String),
    /// A route of the module, by method and `http_route`. `route` is `None` for a request with a
    /// `module` label and no route.
    Route {
        route: Option<String>,
        method: String,
    },
}

/// The row of the app as a whole: a module with no name.
impl Default for RowOf {
    fn default() -> Self {
        RowOf::Module(String::new())
    }
}

/// A rate between two neighbouring kept reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// How long before the newest read this point is: zero for the newest.
    pub ago: Duration,
    pub per_minute: f64,
}

/// Requests per minute over a range.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Line {
    /// Oldest first. A new run starts after a gap: the chart draws no segment between two runs.
    pub runs: Vec<Vec<Point>>,
}

impl Line {
    /// The newest point when it is the newest read: "842 now".
    pub fn now(&self) -> Option<f64> {
        self.runs
            .last()
            .and_then(|run| run.last())
            .filter(|point| point.ago.is_zero())
            .map(|point| point.per_minute)
    }
}

/// One row of the table.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Row {
    pub of: RowOf,
    pub rates: Rates,
    /// Requests per minute over the range, for the chart.
    pub line: Line,
    /// Requests per minute over the last hour, for the column "Last hour".
    pub last_hour: Line,
}

/// The figures of a scope over a range.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Figures {
    /// The time the rates cover. `None` before two reads of one run lie in the range.
    pub covered: Option<Duration>,
    /// All: one row per module of the app. Module: one row per route of the module.
    /// Most requests first, then by name (route, then method).
    pub rows: Vec<Row>,
    /// All: "Not in a module". Module: `None`.
    pub outside: Option<Row>,
    /// Everything in the scope: the app, or the module.
    pub total: Row,
    /// Some kept series has a `module` label.
    pub database_known: bool,
}

/// Where a kept subject counts.
#[derive(Debug, Clone, Copy, Default)]
struct Target {
    /// The row in `rows`, or the last slot for "Not in a module".
    slot: Option<usize>,
    /// It counts in the total of the scope.
    total: bool,
}

/// What the subjects of one row grew by, added.
#[derive(Debug, Clone, Default)]
struct Sum {
    requests: Cell,
    jobs: f64,
    database: f64,
}

impl Sum {
    fn add(&mut self, subject: &Subject, cell: &Cell) {
        match subject {
            Subject::Request { .. } => {
                let requests = &mut self.requests;
                requests.count += cell.count;
                requests.errors += cell.errors;
                requests.sum += cell.sum;
                if requests.buckets.len() < cell.buckets.len() {
                    requests.buckets.resize(cell.buckets.len(), 0.0);
                }
                // Buckets are cumulative, so a bucket holds at least every bucket below it. A
                // series that lacks a bound the store holds has 0 there: the running maximum
                // gives it the bucket of the bound below.
                let mut below = 0.0_f64;
                for (bucket, &grew) in requests.buckets.iter_mut().zip(&cell.buckets) {
                    below = below.max(grew);
                    *bucket += below;
                }
            }
            Subject::Job { .. } => self.jobs += cell.count,
            Subject::Database { .. } => self.database += cell.sum,
        }
    }

    /// The rates over `covered`. `database` is false for a route, and when no series has a
    /// `module` label.
    fn rates(&self, covered: Duration, bounds: &[f64], database: bool) -> Rates {
        let per_minute = 60.0 / covered.as_secs_f64();
        let Cell {
            count,
            errors,
            sum,
            buckets,
        } = &self.requests;
        let requested = *count > 0.0;
        // Buckets are cumulative. 20 × bucket ≥ 19 × count is 95% with no rounding of 0.95.
        let under_95 = bounds.last().filter(|_| requested).map(|&largest| {
            bounds
                .iter()
                .zip(buckets)
                .find(|(_, bucket)| **bucket * 20.0 >= count * 19.0)
                .map_or(Under::Above(largest), |(&bound, _)| Under::Bound(bound))
        });
        Rates {
            requests_per_minute: count * per_minute,
            jobs_per_minute: self.jobs * per_minute,
            under_95,
            errors: requested.then(|| errors / count),
            database: (database && *sum > 0.0).then(|| self.database / sum),
        }
    }
}

/// The line of each slot: what the request subjects of the slot grew by at each step. The last
/// slot is the total. `targets` holds a target per kept subject; one that is no request counts
/// nowhere.
fn lines(steps: &[Step], newest: Duration, targets: &[Target], width: usize) -> Vec<Line> {
    let mut lines = vec![Line::default(); width];
    let mut counts = vec![0.0; width];
    for step in steps {
        counts.fill(0.0);
        for (target, grew) in targets.iter().zip(&step.counts) {
            if let Some(slot) = target.slot {
                counts[slot] += grew;
            }
            if target.total {
                counts[width - 1] += grew;
            }
        }
        let ago = newest.saturating_sub(step.at);
        let per_minute = 60.0 / step.covered.as_secs_f64();
        for (line, &count) in lines.iter_mut().zip(&counts) {
            let point = Point {
                ago,
                per_minute: count * per_minute,
            };
            match line.runs.last_mut() {
                Some(run) if !step.starts => run.push(point),
                _ => line.runs.push(vec![point]),
            }
        }
    }
    lines
}

impl Figures {
    /// Places what the store kept over `range` and works out the rows of `scope`. In the scope All,
    /// every module of `app` has a row, also with no change.
    pub(crate) fn of(
        app: &AppSnapshot,
        placing: &Placing<'_>,
        scope: Scope<'_>,
        store: &MetricsStore,
        range: Range,
    ) -> Self {
        let subjects: Vec<&Subject> = store.subjects().collect();
        let database_known = subjects.iter().any(|subject| match subject {
            Subject::Request { module, .. } | Subject::Job { module, .. } => module.is_some(),
            Subject::Database { .. } => true,
        });

        // The rows, then where each subject counts.
        let (mut ofs, targets): (Vec<RowOf>, Vec<Target>) = match scope {
            Scope::All => {
                let ofs: Vec<RowOf> = app
                    .modules
                    .iter()
                    .map(|module| RowOf::Module(module.name.clone()))
                    .collect();
                let slot: HashMap<&str, usize> = app
                    .modules
                    .iter()
                    .enumerate()
                    .map(|(slot, module)| (module.name.as_str(), slot))
                    .collect();
                let targets = subjects
                    .iter()
                    .map(|subject| Target {
                        slot: Some(
                            placing
                                .place(subject)
                                .and_then(|name| slot.get(name).copied())
                                .unwrap_or(ofs.len()),
                        ),
                        total: true,
                    })
                    .collect();
                (ofs, targets)
            }
            Scope::Module(name) => {
                let mut ofs = Vec::new();
                let mut slot: HashMap<RowOf, usize> = HashMap::new();
                let targets = subjects
                    .iter()
                    .map(|subject| {
                        // `place` gives only names of modules of the app.
                        if placing.place(subject) != Some(name) {
                            return Target::default();
                        }
                        let Subject::Request { route, method, .. } = subject else {
                            return Target {
                                slot: None,
                                total: true,
                            };
                        };
                        let of = RowOf::Route {
                            route: route.clone(),
                            method: method.clone(),
                        };
                        let at = *slot.entry(of).or_insert_with_key(|of| {
                            ofs.push(of.clone());
                            ofs.len() - 1
                        });
                        Target {
                            slot: Some(at),
                            total: true,
                        }
                    })
                    .collect();
                (ofs, targets)
            }
        };
        // The slots: the rows, "Not in a module", then the total.
        let outside = ofs.len();
        let width = outside + 2;
        ofs.push(RowOf::default());
        ofs.push(match scope {
            Scope::All => RowOf::default(),
            Scope::Module(name) => RowOf::Module(name.to_owned()),
        });

        let change = store.change(range.span());
        let mut rates = vec![Rates::default(); width];
        if let Some(change) = &change {
            let mut sums = vec![Sum::default(); width];
            for ((subject, cell), target) in change.cells.iter().zip(&targets) {
                if let Some(slot) = target.slot {
                    sums[slot].add(subject, cell);
                }
                if target.total {
                    sums[width - 1].add(subject, cell);
                }
            }
            let routes = matches!(scope, Scope::Module(_));
            for (slot, (sum, rates)) in sums.iter().zip(&mut rates).enumerate() {
                let database = database_known && !(routes && slot < outside);
                *rates = sum.rates(change.covered, change.bounds, database);
            }
        }

        // Lines count requests only.
        let requests: Vec<Target> = targets
            .iter()
            .zip(&subjects)
            .map(|(target, subject)| match subject {
                Subject::Request { .. } => *target,
                _ => Target::default(),
            })
            .collect();
        let newest = store.newest().unwrap_or_default();
        let line = lines(&store.steps(range.span()), newest, &requests, width);
        let last_hour = lines(&store.steps(Range::Hour.span()), newest, &requests, width);

        let mut rows: Vec<Row> = ofs
            .into_iter()
            .zip(rates)
            .zip(line.into_iter().zip(last_hour))
            .map(|((of, rates), (line, last_hour))| Row {
                of,
                rates,
                line,
                last_hour,
            })
            .collect();
        let total = rows.pop().unwrap_or_default();
        let outside = rows.pop().filter(|_| scope == Scope::All);
        rows.sort_by(|a, b| {
            b.rates
                .requests_per_minute
                .total_cmp(&a.rates.requests_per_minute)
                .then_with(|| a.of.cmp(&b.of))
        });
        Self {
            covered: change.map(|change| change.covered),
            rows,
            outside,
            total,
            database_known,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::metrics::sample::Reading;

    const PACE: Duration = Duration::from_secs(15);

    fn secs(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }

    fn shop() -> AppSnapshot {
        let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/shop-api");
        crate::read_app(Path::new(fixtures)).unwrap()
    }

    fn request(module: &str, route: &str) -> Subject {
        Subject::Request {
            module: Some(module.to_owned()),
            route: Some(route.to_owned()),
            method: "GET".to_owned(),
        }
    }

    fn reading(bounds: &[f64], cells: Vec<(Subject, Cell)>) -> Reading {
        Reading {
            bounds: bounds.to_vec(),
            series: cells
                .into_iter()
                .map(|(subject, cell)| ((subject, String::new()), cell))
                .collect(),
        }
    }

    #[test]
    fn a_module_label_gives_database_time_and_slow_requests_read_above() {
        let app = shop();
        let placing = Placing::new(&app, None);
        let mut store = MetricsStore::new(PACE);
        store.push(secs(0), reading(&[], Vec::new())).unwrap();
        let cell = |count: f64, sum: f64, buckets: Vec<f64>| Cell {
            count,
            errors: 0.0,
            sum,
            buckets,
        };
        let cells = vec![
            (
                request("billing", "/billing/invoices"),
                cell(10.0, 8.0, vec![10.0, 10.0]),
            ),
            (
                Subject::Database {
                    module: "billing".to_owned(),
                },
                cell(0.0, 2.0, Vec::new()),
            ),
            // Every request slower than 10 s, the largest finite bound.
            (
                request("catalog", "/catalog"),
                cell(10.0, 150.0, vec![0.0, 0.0]),
            ),
        ];
        store.push(secs(15), reading(&[0.1, 10.0], cells)).unwrap();

        let figures = Figures::of(&app, &placing, Scope::All, &store, Range::Hour);
        assert!(figures.database_known);
        let of = |name: &str| {
            let row = figures
                .rows
                .iter()
                .find(|row| row.of == RowOf::Module(name.to_owned()));
            row.map(|row| (row.rates.under_95, row.rates.database))
        };
        assert_eq!(of("Billing"), Some((Some(Under::Bound(0.1)), Some(0.25))));
        // A module with no database series reads 0% once any series has the label.
        assert_eq!(of("Catalog"), Some((Some(Under::Above(10.0)), Some(0.0))));
        // A route has no database time.
        let billing = Figures::of(
            &app,
            &placing,
            Scope::Module("Billing"),
            &store,
            Range::Hour,
        );
        assert_eq!(billing.rows[0].rates.database, None);
        assert_eq!(billing.total.rates.database, Some(0.25));

        // Two series of one row give different bounds: each lacks the bound of the other. All
        // 200 requests are at or under 2 s.
        let mut store = MetricsStore::new(PACE);
        store.push(secs(0), reading(&[], Vec::new())).unwrap();
        let cells = vec![
            (
                request("billing", "/billing/a"),
                cell(100.0, 10.0, vec![100.0, f64::NAN]),
            ),
            (
                request("billing", "/billing/b"),
                cell(100.0, 150.0, vec![f64::NAN, 100.0]),
            ),
        ];
        store.push(secs(15), reading(&[1.0, 2.0], cells)).unwrap();
        let figures = Figures::of(&app, &placing, Scope::All, &store, Range::Hour);
        let billing = figures
            .rows
            .iter()
            .find(|row| row.of == RowOf::Module("Billing".to_owned()));
        assert_eq!(
            billing.and_then(|row| row.rates.under_95),
            Some(Under::Bound(2.0))
        );
    }

    #[test]
    fn the_line_of_a_day_has_a_point_per_kept_step_and_breaks_at_a_gap() {
        let app = shop();
        let placing = Placing::new(&app, None);
        let route = request("billing", "/billing/invoices");
        let mut store = MetricsStore::new(PACE);
        // A read every 15 s for 25 hours, one request each, with no read for 75 s after 5 hours
        // and for 10 minutes after 10 hours. The newest read counts 3.
        let newest = 90_000;
        let mut count = 0.0;
        for at in (0..=newest).step_by(15) {
            if (18_015..18_090).contains(&at) || (36_015..36_600).contains(&at) {
                continue;
            }
            count += if at == newest { 3.0 } else { 1.0 };
            let cell = Cell {
                count,
                ..Cell::default()
            };
            store
                .push(secs(at), reading(&[], vec![(route.clone(), cell)]))
                .unwrap();
        }

        let figures = Figures::of(&app, &placing, Scope::All, &store, Range::Day);
        let line = &figures.total.line;
        let agos = |ats: Vec<u64>| -> Vec<Duration> {
            ats.into_iter().map(|at| secs(newest - at)).collect()
        };
        let runs: Vec<Vec<Duration>> = line
            .runs
            .iter()
            .map(|run| run.iter().map(|point| point.ago).collect())
            .collect();
        // A point every 5 minutes before the last hour and every 15 s in it. The read after a
        // gap starts a run, and gives no point. Thinning drops the read at 18,090 s, as 18,000 s
        // holds its 5 minute step, and the line still breaks before 18,300 s.
        assert_eq!(
            runs,
            [
                agos((3_900..=18_000).step_by(300).collect()),
                agos((18_600..=36_000).step_by(300).collect()),
                agos(
                    (36_900..=86_400)
                        .step_by(300)
                        .chain((86_415..=newest).step_by(15))
                        .collect()
                ),
            ]
        );
        let rates: Vec<f64> = line
            .runs
            .iter()
            .flatten()
            .map(|point| point.per_minute)
            .collect();
        let (last, before) = rates.split_last().unwrap();
        assert!(before.iter().all(|&rate| rate == 4.0));
        assert_eq!(*last, 12.0);
        assert_eq!(line.now(), Some(12.0));
        // The column "Last hour" is the same line over 1 hour, whatever the range.
        let hour: Vec<Point> = line.runs[2]
            .iter()
            .copied()
            .filter(|point| point.ago < Range::Hour.span())
            .collect();
        assert_eq!(figures.total.last_hour.runs, [hour]);
        assert_eq!(figures.rows[0].line, *line);
    }
}
