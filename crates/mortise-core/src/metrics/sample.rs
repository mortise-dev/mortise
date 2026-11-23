//! The reads of the metrics, kept in memory.
//!
//! A kept read holds running totals per subject. A figure over a span is the newest totals minus the
//! totals of the oldest kept read in the span, over the time covered between the two. So older reads
//! can be thinned without changing a figure. The time is a parameter: the store never asks the clock.

use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// A read older than this is dropped.
const WINDOW: Duration = Duration::from_secs(24 * 60 * 60);
/// Every read of this last part of the window is kept.
const FINE: Duration = Duration::from_secs(60 * 60);
/// Before the fine part, the first read of each step is kept.
const STEP: Duration = Duration::from_secs(5 * 60);
/// A read more than this many paces after the one before starts a new run.
const GAP_PACES: u32 = 3;
/// `count`, `errors` and `sum` come before the buckets in a row.
const FIXED: usize = 3;

/// What a series is about, as the answer says it. Not a module.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Subject {
    /// A request series. Statuses are merged into one subject.
    Request {
        module: Option<String>,
        route: Option<String>,
        method: String,
    },
    /// A job series. `class` is empty when the series has no `laravel_job_name`.
    Job {
        module: Option<String>,
        class: String,
    },
    /// Database time. Kept only for a series with a `module` label.
    Database { module: String },
}

/// The counters of one subject. All are counts or seconds, so all are `f64`.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Cell {
    /// `_count`, summed over statuses.
    pub count: f64,
    /// `_count` of the series with a 5xx status.
    pub errors: f64,
    /// `_sum`, in seconds.
    pub sum: f64,
    /// `_bucket` per finite bound, as the answer gives them. Requests only.
    pub buckets: Vec<f64>,
}

/// One answer, folded. This is the interface between the parser and the store.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Reading {
    /// The finite bucket bounds of the request histogram, lowest first.
    pub bounds: Vec<f64>,
    pub cells: HashMap<Subject, Cell>,
}

/// What changed over a span.
#[derive(Debug)]
pub(crate) struct Change<'a> {
    /// The time between reads that the span covers. Never zero.
    pub covered: Duration,
    pub bounds: &'a [f64],
    /// Every kept subject with what it grew by. Zero is included. Each cell has one bucket per bound.
    pub cells: Vec<(&'a Subject, Cell)>,
}

/// One kept read.
#[derive(Debug)]
struct Read {
    at: Duration,
    /// The time covered by runs so far. It does not grow over a gap.
    covered: Duration,
    /// The running totals, one row of `FIXED + bounds` numbers per subject, in the order of the
    /// columns. A subject that was not known yet has no row here, which reads as zero.
    totals: Vec<f64>,
}

impl Read {
    fn total(&self, at: usize) -> f64 {
        self.totals.get(at).copied().unwrap_or(0.0)
    }
}

/// One kept subject.
#[derive(Debug)]
struct Column {
    subject: Subject,
    /// The counters of the last read that held the subject.
    base: Option<Vec<f64>>,
    /// The time of the last read that held the subject.
    last_seen: Duration,
}

/// The reads of one environment over the last 24 hours, thinned.
#[derive(Debug)]
pub(crate) struct MetricsStore {
    every: Duration,
    bounds: Vec<f64>,
    columns: Vec<Column>,
    index: HashMap<Subject, usize>,
    /// Reads older than the fine part, the first of each step, oldest first.
    coarse: VecDeque<Read>,
    /// Reads of the fine part, oldest first. The newest read is the last.
    fine: VecDeque<Read>,
    /// The counters of the cell being pushed, kept to save an allocation per cell.
    scratch: Vec<f64>,
}

impl MetricsStore {
    /// `every` is the pace of the reads. A gap is longer than 3 times it.
    pub(crate) fn new(every: Duration) -> Self {
        Self {
            every,
            bounds: Vec::new(),
            columns: Vec::new(),
            index: HashMap::new(),
            coarse: VecDeque::new(),
            fine: VecDeque::new(),
            scratch: Vec::new(),
        }
    }

    /// Adds a read taken at `at`, a time since any fixed start.
    pub(crate) fn push(&mut self, at: Duration, reading: Reading) {
        let Reading { bounds, cells } = reading;
        if !bounds.is_empty() && bounds != self.bounds {
            *self = Self {
                bounds,
                ..Self::new(self.every)
            };
        }
        let (first, covered, mut totals) = match self.fine.back() {
            Some(previous) if at <= previous.at => return,
            Some(previous) if at - previous.at > self.every * GAP_PACES => {
                (true, previous.covered, previous.totals.clone())
            }
            Some(previous) => (
                false,
                previous.covered + (at - previous.at),
                previous.totals.clone(),
            ),
            None => (true, Duration::ZERO, Vec::new()),
        };
        let width = self.width();
        let mut counters = std::mem::take(&mut self.scratch);
        for (subject, cell) in cells {
            if !fill(&cell, self.bounds.len(), &mut counters) {
                continue;
            }
            let column = self.column(subject);
            let row = column * width..(column + 1) * width;
            if totals.len() < row.end {
                totals.resize(row.end, 0.0);
            }
            let kept = &mut self.columns[column];
            kept.last_seen = at;
            if !first {
                match kept.base.as_deref() {
                    None => add(&mut totals[row], counters.iter().copied()),
                    Some(base) if counters.iter().zip(base).all(|(now, base)| now >= base) => add(
                        &mut totals[row],
                        counters.iter().zip(base).map(|(now, base)| now - base),
                    ),
                    Some(_) => {}
                }
            }
            match &mut kept.base {
                Some(base) => base.copy_from_slice(&counters),
                None => kept.base = Some(counters.clone()),
            }
        }
        self.scratch = counters;
        self.fine.push_back(Read {
            at,
            covered,
            totals,
        });
        self.thin(at);
        self.drop_gone();
    }

    /// How many reads are kept.
    pub(crate) fn len(&self) -> usize {
        self.coarse.len() + self.fine.len()
    }

    /// The newest read against the oldest kept read at most `span` before it.
    /// `None` when no time is covered between them.
    pub(crate) fn change(&self, span: Duration) -> Option<Change<'_>> {
        let newest = self.fine.back()?;
        let from = newest.at.saturating_sub(span);
        let coarse = self.coarse.partition_point(|read| read.at < from);
        let oldest = match self.coarse.get(coarse) {
            Some(read) => read,
            None => &self.fine[self.fine.partition_point(|read| read.at < from)],
        };
        let covered = newest.covered - oldest.covered;
        if covered.is_zero() {
            return None;
        }
        let width = self.width();
        let grew = |at: usize| newest.total(at) - oldest.total(at);
        let cells = self
            .columns
            .iter()
            .enumerate()
            .map(|(column, kept)| {
                let start = column * width;
                let cell = Cell {
                    count: grew(start),
                    errors: grew(start + 1),
                    sum: grew(start + 2),
                    buckets: (start + FIXED..start + width).map(grew).collect(),
                };
                (&kept.subject, cell)
            })
            .collect();
        Some(Change {
            covered,
            bounds: &self.bounds,
            cells,
        })
    }

    /// The numbers per subject in a row.
    fn width(&self) -> usize {
        FIXED + self.bounds.len()
    }

    /// The column of a subject. A new subject gets a new column, with no base.
    fn column(&mut self, subject: Subject) -> usize {
        if let Some(&column) = self.index.get(&subject) {
            return column;
        }
        let column = self.columns.len();
        self.index.insert(subject.clone(), column);
        self.columns.push(Column {
            subject,
            base: None,
            last_seen: Duration::ZERO,
        });
        column
    }

    /// Moves reads out of the fine part, keeping the first of each step, and drops what is too old.
    fn thin(&mut self, newest: Duration) {
        let step = |at: Duration| at.as_secs() / STEP.as_secs();
        while let Some(read) = self.fine.pop_front_if(|read| newest - read.at > FINE) {
            if self
                .coarse
                .back()
                .is_none_or(|last| step(last.at) != step(read.at))
            {
                self.coarse.push_back(read);
            }
        }
        while self
            .coarse
            .pop_front_if(|read| newest - read.at > WINDOW)
            .is_some()
        {}
    }

    /// Drops the subjects that no kept read holds, and packs the rows.
    ///
    /// Such a subject did not change in the kept window and is not in the newest read.
    /// One that is in an older kept read stays, so it keeps its base (a load balancer can show
    /// one server per read).
    fn drop_gone(&mut self) {
        let Some(oldest) = self
            .coarse
            .front()
            .or(self.fine.front())
            .map(|read| read.at)
        else {
            return;
        };
        let keep: Vec<bool> = self
            .columns
            .iter()
            .map(|kept| kept.last_seen >= oldest)
            .collect();
        if keep.iter().all(|&keep| keep) {
            return;
        }
        let width = self.width();
        for read in self.coarse.iter_mut().chain(self.fine.iter_mut()) {
            let rows = read.totals.len() / width;
            let mut packed = 0;
            for (column, _) in keep
                .iter()
                .enumerate()
                .take(rows)
                .filter(|(_, keep)| **keep)
            {
                let start = column * width;
                read.totals
                    .copy_within(start..start + width, packed * width);
                packed += 1;
            }
            read.totals.truncate(packed * width);
        }
        let mut keep = keep.into_iter();
        self.columns.retain(|_| keep.next() == Some(true));
        self.index = self
            .columns
            .iter()
            .enumerate()
            .map(|(column, kept)| (kept.subject.clone(), column))
            .collect();
    }
}

/// Writes the counters of `cell` into `into`, one bucket per bound. A missing bucket is zero.
/// `false` when a counter is negative or not finite: the store then ignores the cell.
fn fill(cell: &Cell, bounds: usize, into: &mut Vec<f64>) -> bool {
    into.clear();
    into.extend([cell.count, cell.errors, cell.sum]);
    into.extend(
        cell.buckets
            .iter()
            .copied()
            .chain(std::iter::repeat(0.0))
            .take(bounds),
    );
    into.iter().all(|value| value.is_finite() && *value >= 0.0)
}

fn add(totals: &mut [f64], by: impl Iterator<Item = f64>) {
    for (total, by) in totals.iter_mut().zip(by) {
        *total += by;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    const PACE: Duration = Duration::from_secs(15);

    fn secs(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }

    fn route(path: &str) -> Subject {
        Subject::Request {
            module: None,
            route: Some(path.to_owned()),
            method: "GET".to_owned(),
        }
    }

    fn count(count: f64) -> Cell {
        Cell {
            count,
            ..Cell::default()
        }
    }

    fn reading(cells: &[(Subject, Cell)]) -> Reading {
        Reading {
            bounds: Vec::new(),
            cells: cells.iter().cloned().collect(),
        }
    }

    /// What `subject` grew by over `span`.
    fn grew(store: &MetricsStore, span: Duration, subject: &Subject) -> Option<Cell> {
        let change = store.change(span)?;
        change
            .cells
            .into_iter()
            .find(|(kept, _)| *kept == subject)
            .map(|(_, cell)| cell)
    }

    #[test]
    fn the_first_read_gives_no_change() {
        let mut store = MetricsStore::new(PACE);
        store.push(secs(0), reading(&[(route("/"), count(7.0))]));
        store.push(secs(0), reading(&[(route("/"), count(9.0))]));

        assert_eq!(store.len(), 1);
        assert!(store.change(WINDOW).is_none());
    }

    #[test]
    fn a_reset_adds_nothing_and_the_next_read_counts() {
        let cell = |count, sum| Cell {
            count,
            sum,
            ..Cell::default()
        };
        let mut store = MetricsStore::new(PACE);
        store.push(secs(0), reading(&[(route("/"), cell(100.0, 50.0))]));
        store.push(secs(15), reading(&[(route("/"), cell(3.0, 1.0))]));
        assert_eq!(grew(&store, PACE, &route("/")), Some(cell(0.0, 0.0)));

        store.push(secs(30), reading(&[(route("/"), cell(5.0, 2.0))]));
        assert_eq!(grew(&store, PACE, &route("/")), Some(cell(2.0, 1.0)));
        assert_eq!(grew(&store, WINDOW, &route("/")), Some(cell(2.0, 1.0)));
    }

    #[test]
    fn a_subject_that_appears_in_a_run_counts_from_zero() {
        let failing = Cell {
            count: 4.0,
            errors: 1.0,
            sum: 0.5,
            buckets: Vec::new(),
        };
        let mut store = MetricsStore::new(PACE);
        store.push(secs(0), reading(&[(route("/"), count(10.0))]));
        store.push(
            secs(15),
            reading(&[(route("/"), count(10.0)), (route("/pay"), failing.clone())]),
        );

        assert_eq!(grew(&store, WINDOW, &route("/pay")), Some(failing));
    }

    #[test]
    fn a_missing_subject_keeps_its_totals_and_its_base() {
        let mut store = MetricsStore::new(PACE);
        store.push(secs(0), reading(&[(route("/"), count(10.0))]));
        store.push(secs(15), reading(&[(route("/"), count(15.0))]));
        store.push(secs(30), reading(&[]));
        assert_eq!(grew(&store, WINDOW, &route("/")), Some(count(5.0)));

        // A negative counter is ignored, as if the subject were missing.
        store.push(secs(45), reading(&[(route("/"), count(-1.0))]));
        store.push(secs(60), reading(&[(route("/"), count(18.0))]));
        assert_eq!(grew(&store, WINDOW, &route("/")), Some(count(8.0)));
    }

    #[test]
    fn a_read_after_a_gap_starts_a_new_run() {
        let mut store = MetricsStore::new(PACE);
        store.push(secs(0), reading(&[(route("/"), count(10.0))]));
        store.push(secs(46), reading(&[(route("/"), count(20.0))]));
        assert!(store.change(WINDOW).is_none());

        store.push(secs(76), reading(&[(route("/"), count(25.0))]));
        let change = store.change(WINDOW).expect("a change");
        assert_eq!(change.covered, secs(30));
        assert_eq!(change.cells, vec![(&route("/"), count(5.0))]);
    }

    #[test]
    fn a_rate_divides_by_the_covered_time() {
        let mut store = MetricsStore::new(PACE);
        for (at, total) in [(0, 0.0), (15, 6.0), (100, 50.0), (115, 56.0)] {
            store.push(secs(at), reading(&[(route("/"), count(total))]));
        }

        let change = store.change(WINDOW).expect("a change");
        assert_eq!(change.covered, secs(30));
        assert_eq!(change.cells[0].1.count / change.covered.as_secs_f64(), 0.4);
    }

    #[test]
    fn other_bounds_empty_the_store() {
        let request = |buckets: Vec<f64>| Cell {
            count: buckets[1],
            buckets,
            ..Cell::default()
        };
        let job = Subject::Job {
            module: None,
            class: String::new(),
        };
        let with_bounds = |bounds: Vec<f64>, cell: Cell| Reading {
            bounds,
            cells: HashMap::from([(route("/"), cell)]),
        };
        let mut store = MetricsStore::new(PACE);
        store.push(
            secs(0),
            with_bounds(vec![0.1, 1.0], request(vec![1.0, 2.0])),
        );
        store.push(
            secs(15),
            with_bounds(vec![0.1, 1.0], request(vec![2.0, 4.0])),
        );

        store.push(secs(30), reading(&[(job.clone(), count(1.0))]));
        assert_eq!(store.len(), 3);
        let change = store.change(WINDOW).expect("a change");
        assert_eq!(change.bounds, [0.1, 1.0]);
        assert_eq!(
            grew(&store, WINDOW, &route("/")),
            Some(request(vec![1.0, 2.0]))
        );

        store.push(secs(45), with_bounds(vec![0.5], request(vec![3.0, 3.0])));
        assert_eq!(store.len(), 1);
        assert!(store.change(WINDOW).is_none());
    }

    #[test]
    fn a_day_of_reads_is_thinned_and_keeps_its_figures() {
        let started = Instant::now();
        let gone = route("/old");
        let stays = |n: u64| (n % 5) as f64;
        let mut store = MetricsStore::new(PACE);
        let mut total = 0.0;
        let mut full = 0;
        for n in 0..7_000 {
            total += stays(n);
            // `/old` comes first, so dropping it moves the row of `/`.
            let mut cells = Vec::new();
            if n < 1_000 {
                cells.push((gone.clone(), count(n as f64)));
            }
            if n > 0 {
                cells.push((route("/"), count(total)));
            }
            store.push(PACE * n as u32, reading(&cells));

            assert!(store.len() <= 520);
            match n + 1 {
                5_760 => full = store.len(),
                6_000 => assert!(grew(&store, WINDOW, &gone).is_some()),
                pushes if pushes > 5_760 => assert!(store.len() <= full),
                _ => {}
            }
        }

        let change = store.change(WINDOW).expect("a change");
        assert!(change.covered > WINDOW - STEP && change.covered <= WINDOW);
        let oldest = 6_999 - change.covered.as_secs() / PACE.as_secs();
        let pushed: f64 = (oldest + 1..7_000).map(stays).sum();
        assert_eq!(change.cells, vec![(&route("/"), count(pushed))]);
        assert!(started.elapsed() < secs(1), "took {:?}", started.elapsed());
    }
}
