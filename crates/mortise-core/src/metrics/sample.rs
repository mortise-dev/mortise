//! The reads of the metrics, kept in memory.
//!
//! A kept read holds running totals per subject. A figure over a span is the newest totals minus the
//! totals of the oldest kept read in the span, over the time covered between the two. So older reads
//! can be thinned without changing a figure. The time is a parameter: the store never asks the clock.
//! [`LiveMetrics`] holds the store of one environment and the clock of a real run.

use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime};

use crate::app::AppSnapshot;
use crate::metrics::figures::Figures;
use crate::metrics::place::Placing;
use crate::metrics::prometheus::{Part, samples};
use crate::runtime::actuator::{NoMetrics, Route};
use crate::runtime::live::Pace;

/// A read older than this is dropped.
const WINDOW: Duration = Duration::from_secs(24 * 60 * 60);
/// In this last part of the window, one read per pace is kept: the last of each step.
const FINE: Duration = Duration::from_secs(60 * 60);
/// Before the fine part, the first read of each step is kept.
const STEP: Duration = Duration::from_secs(5 * 60);
/// A read more than this many paces after the one before starts a new run.
const GAP_PACES: u32 = 3;
/// The families that [`Reading::of`] folds. Every other family is skipped.
const REQUESTS: &str = "http_server_request_duration_seconds";
const JOBS: &str = "messaging_process_duration_seconds";
const DATABASE: &str = "db_client_operation_duration_seconds";
/// `count`, `errors` and `sum` come before the buckets in a row.
const FIXED: usize = 3;
/// The most numbers one read keeps: `FIXED` and one per bound, for each series. A reading over it is
/// refused. The 60 module app of design D1 keeps 660 × 17 = 11,220.
const MAX_ROW: usize = 32_768;

/// What a series is about, as the answer says it. Not a module.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Subject {
    /// A request series. Statuses are added into one subject.
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

/// The counters of one series, or what a subject grew by. All are counts or seconds, so all are
/// `f64`.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Cell {
    /// `_count`.
    pub count: f64,
    /// `_count` of a series with a 5xx status.
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
    /// The counters of each series, by subject and the labels of the series (see [`series_key`]).
    pub series: HashMap<(Subject, String), Cell>,
}

/// A reading that would make a read keep more than `MAX_ROW` numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TooMany;

/// The labels of a series, `le` aside, sorted by name: `name="value",` each, with the value escaped.
/// Two lines of one series give one key, whatever order they write the labels in.
fn series_key(labels: &mut [(&str, Cow<'_, str>)]) -> String {
    use std::fmt::Write;
    labels.sort_unstable_by_key(|(name, _)| *name);
    let mut key = String::new();
    for (name, value) in labels.iter() {
        // Writing to a `String` does not fail.
        let _ = write!(key, "{name}={value:?},");
    }
    key
}

/// `true` when `series` with `bounds` need more than `MAX_ROW` numbers.
fn over(series: usize, bounds: usize) -> bool {
    series.saturating_mul(FIXED + bounds) > MAX_ROW
}

impl Reading {
    /// Folds an answer. Runs on the worker thread.
    ///
    /// A `+Inf` bucket is dropped: it equals the count. An empty `module` label counts as no label.
    /// An answer over `MAX_ROW` numbers is refused before the reading grows past it.
    pub(crate) fn of(text: &str) -> Result<Reading, TooMany> {
        let mut reading = Reading::default();
        // The index of each bound in `reading.bounds`, by its bits. `0.0` and `-0.0` are one bound.
        let mut bound_at: HashMap<u64, usize> = HashMap::new();
        for mut sample in samples(text) {
            let module = || {
                sample
                    .label("module")
                    .filter(|module| !module.is_empty())
                    .map(str::to_owned)
            };
            let folded = match (sample.family, sample.part) {
                (REQUESTS, Part::Count | Part::Sum) => true,
                (REQUESTS, Part::Bucket(bound)) => bound.is_finite(),
                (JOBS, Part::Count) | (DATABASE, Part::Sum) => true,
                _ => false,
            };
            if !folded {
                continue;
            }
            let subject = match sample.family {
                REQUESTS => Subject::Request {
                    module: module(),
                    route: sample.label("http_route").map(str::to_owned),
                    method: sample
                        .label("http_request_method")
                        .unwrap_or_default()
                        .to_owned(),
                },
                JOBS => Subject::Job {
                    module: module(),
                    class: sample
                        .label("laravel_job_name")
                        .unwrap_or_default()
                        .to_owned(),
                },
                _ => match module() {
                    Some(module) => Subject::Database { module },
                    None => continue,
                },
            };
            let error = sample
                .label("http_response_status_code")
                .and_then(|status| status.parse::<u16>().ok())
                .is_some_and(|status| (500..600).contains(&status));
            let key = series_key(&mut sample.labels);
            let mut series = reading.series.len();
            let cell = match reading.series.entry((subject, key)) {
                Entry::Occupied(cell) => cell.into_mut(),
                Entry::Vacant(cell) => {
                    series += 1;
                    if over(series, reading.bounds.len()) {
                        return Err(TooMany);
                    }
                    cell.insert(Cell::default())
                }
            };
            match sample.part {
                Part::Count => {
                    cell.count += sample.value;
                    if error {
                        cell.errors += sample.value;
                    }
                }
                Part::Sum => cell.sum += sample.value,
                Part::Bucket(bound) => {
                    let bits = if bound == 0.0 { 0.0f64 } else { bound }.to_bits();
                    let at = match bound_at.get(&bits) {
                        Some(&at) => at,
                        None => {
                            if over(series, reading.bounds.len() + 1) {
                                return Err(TooMany);
                            }
                            reading.bounds.push(bound);
                            bound_at.insert(bits, reading.bounds.len() - 1);
                            reading.bounds.len() - 1
                        }
                    };
                    if cell.buckets.len() <= at {
                        cell.buckets.resize(at + 1, 0.0);
                    }
                    cell.buckets[at] += sample.value;
                }
                Part::Value => {}
            }
        }
        Ok(reading)
    }
}

/// What changed over a span.
#[derive(Debug)]
pub(crate) struct Change<'a> {
    /// The time between reads that the span covers. Never zero.
    pub covered: Duration,
    /// Every kept subject with what it grew by. Zero is included. Each cell has one bucket per bound
    /// of the store.
    pub cells: Vec<(&'a Subject, Cell)>,
}

/// One kept read.
#[derive(Debug)]
struct Read {
    at: Duration,
    /// The time covered by runs so far. It does not grow over a gap.
    covered: Duration,
    /// `true` for the first read of a run.
    starts_run: bool,
    /// The running totals, one row of `FIXED + bounds` numbers per subject, in the order of the
    /// columns. A subject that was not known yet has no row here, which reads as zero.
    totals: Vec<f64>,
}

impl Read {
    fn total(&self, at: usize) -> f64 {
        self.totals.get(at).copied().unwrap_or(0.0)
    }
}

/// The counters of one series at the last read that held it.
#[derive(Debug)]
struct Base {
    counters: Vec<f64>,
    /// The run of that read. A base of an older run is stale: the series was missing when the run
    /// started, so what it grew by in the gap is not known.
    run: u64,
    /// The time of that read.
    seen: Duration,
}

/// One kept subject.
#[derive(Debug)]
struct Column {
    subject: Subject,
    /// The base of each series of the subject, by its labels.
    bases: HashMap<String, Base>,
}

/// The reads of one environment over the last 24 hours, thinned.
#[derive(Debug)]
pub(crate) struct MetricsStore {
    every: Duration,
    bounds: Vec<f64>,
    columns: Vec<Column>,
    index: HashMap<Subject, usize>,
    /// The number of the current run. It goes up at the first read of each run.
    run: u64,
    /// Reads older than the fine part, the first of each step, oldest first.
    coarse: VecDeque<Read>,
    /// Reads of the fine part, the last of each pace, oldest first. The newest read is the last.
    fine: VecDeque<Read>,
    /// The counters of the series being pushed, kept to save an allocation per series.
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
            run: 0,
            coarse: VecDeque::new(),
            fine: VecDeque::new(),
            scratch: Vec::new(),
        }
    }

    /// Adds a read taken at `at`, a time since any fixed start.
    ///
    /// A reading that would make the store keep more than `MAX_ROW` numbers per read, counting
    /// every series of the kept reads, is refused as a whole and changes nothing.
    pub(crate) fn push(&mut self, at: Duration, reading: Reading) -> Result<(), TooMany> {
        let Reading { bounds, series } = reading;
        let renewed = !bounds.is_empty() && !self.bounds.is_empty() && bounds != self.bounds;
        let width = if bounds.is_empty() {
            self.bounds.len()
        } else {
            bounds.len()
        };
        let kept = if renewed {
            series.len()
        } else {
            let known = |(subject, labels): &(Subject, String)| {
                self.index
                    .get(subject)
                    .is_some_and(|&column| self.columns[column].bases.contains_key(labels))
            };
            let bases: usize = self.columns.iter().map(|kept| kept.bases.len()).sum();
            bases + series.keys().filter(|key| !known(key)).count()
        };
        if over(kept, width) {
            return Err(TooMany);
        }
        if !bounds.is_empty() && bounds != self.bounds {
            if self.bounds.is_empty() {
                self.widen(bounds);
            } else {
                *self = Self {
                    bounds,
                    ..Self::new(self.every)
                };
            }
        }
        let previous = match self.fine.back() {
            Some(previous) if at <= previous.at => return Ok(()),
            previous => previous.map(|previous| (previous.at, previous.covered)),
        };
        let first = previous.is_none_or(|(before, _)| at - before > self.every * GAP_PACES);
        let covered = match previous {
            Some((before, covered)) if !first => covered + (at - before),
            Some((_, covered)) => covered,
            None => Duration::ZERO,
        };
        // One read per pace in the fine part: a read in the step of the one before takes its place.
        // Totals only grow, so the dropped read is not needed for a figure.
        let replaces = previous.is_some_and(|(before, _)| self.step(before) == self.step(at));
        let mut totals = if replaces {
            self.fine.pop_back().map(|read| read.totals)
        } else {
            self.fine.back().map(|read| read.totals.clone())
        }
        .unwrap_or_default();
        if first {
            self.run += 1;
        }
        let run = self.run;
        let width = self.width();
        let mut counters = std::mem::take(&mut self.scratch);
        // Each series is compared with its own base, then what it grew by adds into its subject.
        for ((subject, labels), cell) in series {
            if !fill(&cell, self.bounds.len(), &mut counters) {
                continue;
            }
            let column = self.column(subject);
            let row = column * width..(column + 1) * width;
            if totals.len() < row.end {
                totals.resize(row.end, 0.0);
            }
            let bases = &mut self.columns[column].bases;
            match bases.get_mut(&labels) {
                Some(base) => {
                    // A stale base and a reset add nothing. On the first read of a run, every base
                    // is stale.
                    if base.run == run
                        && counters
                            .iter()
                            .zip(&base.counters)
                            .all(|(now, base)| now >= base)
                    {
                        add(
                            &mut totals[row],
                            counters
                                .iter()
                                .zip(&base.counters)
                                .map(|(now, base)| now - base),
                        );
                    }
                    base.counters.copy_from_slice(&counters);
                    base.run = run;
                    base.seen = at;
                }
                None => {
                    // A new series counts from zero, except on the first read of a run.
                    if !first {
                        add(&mut totals[row], counters.iter().copied());
                    }
                    bases.insert(
                        labels,
                        Base {
                            counters: counters.clone(),
                            run,
                            seen: at,
                        },
                    );
                }
            }
        }
        self.scratch = counters;
        self.fine.push_back(Read {
            at,
            covered,
            starts_run: first,
            totals,
        });
        self.thin(at);
        self.drop_gone();
        Ok(())
    }

    /// How many reads are kept.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.coarse.len() + self.fine.len()
    }

    /// The newest read against the oldest kept read at most `span` before it.
    /// `None` when no time is covered between them, or when the newest read starts a run.
    pub(crate) fn change(&self, span: Duration) -> Option<Change<'_>> {
        let newest = self.fine.back()?;
        if newest.starts_run {
            return None;
        }
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
        Some(Change { covered, cells })
    }

    /// The numbers per subject in a row.
    fn width(&self) -> usize {
        FIXED + self.bounds.len()
    }

    /// The step of the fine part that `at` falls in: one per pace.
    fn step(&self, at: Duration) -> u128 {
        at.as_nanos()
            .checked_div(self.every.as_nanos())
            .unwrap_or(at.as_nanos())
    }

    /// Takes the first bounds while none are kept. Every row and base gets a zero bucket per bound,
    /// so a subject read before them keeps its totals. Jobs and database time come before the first
    /// request in a quiet app.
    fn widen(&mut self, bounds: Vec<f64>) {
        let added = bounds.len();
        self.bounds = bounds;
        for read in self.coarse.iter_mut().chain(self.fine.iter_mut()) {
            read.totals = read
                .totals
                .chunks(FIXED)
                .flat_map(|row| row.iter().copied().chain(std::iter::repeat_n(0.0, added)))
                .collect();
        }
        for base in self
            .columns
            .iter_mut()
            .flat_map(|column| column.bases.values_mut())
        {
            base.counters.resize(FIXED + added, 0.0);
        }
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
            bases: HashMap::new(),
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

    /// Drops the series and the subjects that no kept read holds, and packs the rows.
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
        for column in &mut self.columns {
            column.bases.retain(|_, base| base.seen >= oldest);
        }
        let keep: Vec<bool> = self
            .columns
            .iter()
            .map(|kept| !kept.bases.is_empty())
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
/// `false` when a counter is negative or not finite: the store then ignores the series.
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

/// The clock of the reads: the time since any fixed start.
pub type MetricsClock = Arc<dyn Fn() -> Duration + Send + Sync>;

/// Why a read added nothing to the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotKept {
    /// The app gave no metrics.
    NoMetrics(NoMetrics),
    /// The answer has more series than a read keeps. `limit` is the most numbers a read keeps.
    TooManySeries { limit: usize },
}

impl From<TooMany> for NotKept {
    fn from(TooMany: TooMany) -> Self {
        NotKept::TooManySeries { limit: MAX_ROW }
    }
}

/// What the reads of an environment gave so far.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetricsState {
    /// What the last read gave. `None` before the first read ends.
    pub last: Option<Result<(), NotKept>>,
    /// Goes up by one when a read ends.
    pub serial: u64,
}

/// The reads of one environment, shared between the worker and the interface.
#[derive(Clone)]
pub struct LiveMetrics {
    shared: Arc<Mutex<(MetricsStore, MetricsState)>>,
    clock: MetricsClock,
}

impl Default for LiveMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveMetrics {
    /// Reads timed by the clock of a real run.
    pub fn new() -> Self {
        Self::with_clock(real_clock())
    }

    /// Reads timed by `clock`. Tests pass a clock that steps.
    pub fn with_clock(clock: MetricsClock) -> Self {
        let store = MetricsStore::new(Pace::default().metrics);
        Self {
            shared: Arc::new(Mutex::new((store, MetricsState::default()))),
            clock,
        }
    }

    /// For `LiveRuntime::open_with_metrics`. The text is folded before the lock is taken.
    /// A read that gave no metrics, or too many series, adds nothing to the store.
    pub fn sink(&self) -> impl FnMut(Result<String, NoMetrics>) + Send + 'static {
        let metrics = self.clone();
        move |answer| {
            let at = (metrics.clock)();
            let reading = answer
                .map_err(NotKept::NoMetrics)
                .and_then(|text| Reading::of(&text).map_err(NotKept::from));
            let mut shared = metrics.lock();
            let last =
                reading.and_then(|reading| shared.0.push(at, reading).map_err(NotKept::from));
            shared.1.last = Some(last);
            shared.1.serial += 1;
        }
    }

    pub fn state(&self) -> MetricsState {
        self.lock().1.clone()
    }

    /// The figures per module over the last `span`, placed by `app` and `routes`.
    /// `routes` is `None` when `mappings` is not exposed.
    pub fn figures(&self, app: &AppSnapshot, routes: Option<&[Route]>, span: Duration) -> Figures {
        let placing = Placing::new(app, routes);
        let shared = self.lock();
        Figures::of(app, &placing, shared.0.change(span))
    }

    fn lock(&self) -> MutexGuard<'_, (MetricsStore, MetricsState)> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The clock of a real run: `Instant` since the start, plus the time the computer slept.
///
/// `Instant` does not move while macOS or Linux sleep. So each call compares it with the wall clock
/// and keeps the time the wall clock ran ahead. The result never goes back.
fn real_clock() -> MetricsClock {
    let start = Instant::now();
    let last = Mutex::new((Duration::ZERO, SystemTime::now(), Duration::ZERO));
    Arc::new(move || {
        let mut last = last.lock().unwrap_or_else(PoisonError::into_inner);
        let (instant, wall, offset) = *last;
        let (instant_now, wall_now) = (start.elapsed(), SystemTime::now());
        let offset = ahead(
            offset,
            instant_now.saturating_sub(instant),
            wall_now.duration_since(wall).unwrap_or_default(),
        );
        *last = (instant_now, wall_now, offset);
        instant_now + offset
    })
}

/// The offset of the real clock after a call. It grows by the time the wall clock ran further than
/// `Instant` since the call before. A wall clock that was set back ran zero, so it never shrinks.
fn ahead(offset: Duration, instant_ran: Duration, wall_ran: Duration) -> Duration {
    offset + wall_ran.saturating_sub(instant_ran)
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

    /// A reading with one series per subject, with no label.
    fn reading(cells: &[(Subject, Cell)]) -> Reading {
        Reading {
            bounds: Vec::new(),
            series: cells
                .iter()
                .map(|(subject, cell)| ((subject.clone(), String::new()), cell.clone()))
                .collect(),
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
        store
            .push(secs(0), reading(&[(route("/"), count(7.0))]))
            .unwrap();
        store
            .push(secs(0), reading(&[(route("/"), count(9.0))]))
            .unwrap();

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
        store
            .push(secs(0), reading(&[(route("/"), cell(100.0, 50.0))]))
            .unwrap();
        store
            .push(secs(15), reading(&[(route("/"), cell(3.0, 1.0))]))
            .unwrap();
        assert_eq!(grew(&store, PACE, &route("/")), Some(cell(0.0, 0.0)));

        store
            .push(secs(30), reading(&[(route("/"), cell(5.0, 2.0))]))
            .unwrap();
        assert_eq!(grew(&store, PACE, &route("/")), Some(cell(2.0, 1.0)));
        assert_eq!(grew(&store, WINDOW, &route("/")), Some(cell(2.0, 1.0)));

        // Each label set is a series of its own: a reset on http does not hide the growth on https,
        // in whatever order a read writes the labels.
        let schemes = |http, https, labels: [&str; 2]| {
            let [first, second] = labels;
            Reading::of(&format!(
                "# TYPE {REQUESTS} histogram\n\
                 {REQUESTS}_count{{http_request_method=\"GET\",{first},{second}}} {https}\n\
                 {REQUESTS}_count{{http_request_method=\"GET\",http_route=\"/\",url_scheme=\"http\"}} {http}\n"
            ))
            .unwrap()
        };
        let https = ["http_route=\"/\"", "url_scheme=\"https\""];
        let [route_label, scheme_label] = https;
        let mut store = MetricsStore::new(PACE);
        store.push(secs(0), schemes(100, 100, https)).unwrap();
        store
            .push(secs(15), schemes(1, 110, [scheme_label, route_label]))
            .unwrap();
        assert_eq!(grew(&store, PACE, &route("/")), Some(count(10.0)));
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
        store
            .push(secs(0), reading(&[(route("/"), count(10.0))]))
            .unwrap();
        store
            .push(
                secs(15),
                reading(&[(route("/"), count(10.0)), (route("/pay"), failing.clone())]),
            )
            .unwrap();

        assert_eq!(grew(&store, WINDOW, &route("/pay")), Some(failing));
    }

    #[test]
    fn a_missing_subject_keeps_its_totals_and_its_base() {
        let mut store = MetricsStore::new(PACE);
        store
            .push(secs(0), reading(&[(route("/"), count(10.0))]))
            .unwrap();
        store
            .push(secs(15), reading(&[(route("/"), count(15.0))]))
            .unwrap();
        store.push(secs(30), reading(&[])).unwrap();
        assert_eq!(grew(&store, WINDOW, &route("/")), Some(count(5.0)));

        // A negative counter is ignored, as if the subject were missing.
        store
            .push(secs(45), reading(&[(route("/"), count(-1.0))]))
            .unwrap();
        store
            .push(secs(60), reading(&[(route("/"), count(18.0))]))
            .unwrap();
        assert_eq!(grew(&store, WINDOW, &route("/")), Some(count(8.0)));
    }

    #[test]
    fn a_read_after_a_gap_starts_a_new_run() {
        let both = |home, blog| reading(&[(route("/"), count(home)), (route("/b"), count(blog))]);
        let mut store = MetricsStore::new(PACE);
        store.push(secs(0), both(10.0, 1.0)).unwrap();
        store.push(secs(15), both(12.0, 2.0)).unwrap();
        // The first read after a gap gives no rate, though the span reaches into the run before.
        store
            .push(secs(61), reading(&[(route("/"), count(20.0))]))
            .unwrap();
        assert!(store.change(WINDOW).is_none());

        // `/b` was missing when the run started, so its return adds nothing.
        store.push(secs(76), both(25.0, 50.0)).unwrap();
        let change = store.change(WINDOW).expect("a change");
        assert_eq!(change.covered, secs(30));
        assert_eq!(grew(&store, WINDOW, &route("/")), Some(count(7.0)));
        assert_eq!(grew(&store, WINDOW, &route("/b")), Some(count(1.0)));

        store.push(secs(91), both(25.0, 53.0)).unwrap();
        assert_eq!(grew(&store, PACE, &route("/b")), Some(count(3.0)));
    }

    #[test]
    fn a_rate_divides_by_the_covered_time() {
        let mut store = MetricsStore::new(PACE);
        for (at, total) in [(0, 0.0), (15, 6.0), (100, 50.0), (115, 56.0)] {
            store
                .push(secs(at), reading(&[(route("/"), count(total))]))
                .unwrap();
        }

        let change = store.change(WINDOW).expect("a change");
        assert_eq!(change.covered, secs(30));
        assert_eq!(change.cells[0].1.count / change.covered.as_secs_f64(), 0.4);
    }

    #[test]
    fn the_first_bounds_widen_the_store_and_other_bounds_empty_it() {
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
            series: HashMap::from([((route("/"), String::new()), cell)]),
        };
        let mut store = MetricsStore::new(PACE);
        // A job comes before the first request, so before the first bounds.
        store
            .push(secs(0), reading(&[(job.clone(), count(1.0))]))
            .unwrap();
        store
            .push(
                secs(15),
                with_bounds(vec![0.1, 1.0], request(vec![1.0, 2.0])),
            )
            .unwrap();
        store
            .push(
                secs(30),
                with_bounds(vec![0.1, 1.0], request(vec![2.0, 4.0])),
            )
            .unwrap();

        store
            .push(secs(45), reading(&[(job.clone(), count(3.0))]))
            .unwrap();
        assert_eq!(store.len(), 4);
        assert_eq!(store.bounds, [0.1, 1.0]);
        assert_eq!(
            grew(&store, WINDOW, &route("/")),
            Some(request(vec![2.0, 4.0]))
        );
        assert_eq!(grew(&store, WINDOW, &job).map(|cell| cell.count), Some(2.0));

        store
            .push(secs(60), with_bounds(vec![0.5], request(vec![3.0, 3.0])))
            .unwrap();
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
            store.push(PACE * n as u32, reading(&cells)).unwrap();

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

        // "Try again" can read faster than the pace. Two hours of reads 1 s apart keep the bound,
        // and the figures.
        let burst = PACE * 7_000;
        for n in 0..7_200 {
            let reading = reading(&[(route("/"), count(total + n as f64))]);
            store.push(burst + secs(n), reading).unwrap();
            assert!(store.len() <= 520);
        }
        let change = store.change(secs(60)).expect("a change");
        assert_eq!(change.cells[0].1.count, change.covered.as_secs_f64());
        assert!(started.elapsed() < secs(1), "took {:?}", started.elapsed());
    }

    #[test]
    fn too_many_series_are_refused_as_a_whole() {
        let jobs = |range: std::ops::Range<usize>| {
            let mut text = format!("# TYPE {JOBS} histogram\n");
            for job in range {
                text.push_str(&format!("{JOBS}_count{{laravel_job_name=\"J{job}\"}} 1\n"));
            }
            text
        };
        // 3 numbers per job.
        let metrics = LiveMetrics::with_clock(Arc::new(|| Duration::ZERO));
        let mut sink = metrics.sink();
        sink(Ok(jobs(0..MAX_ROW / 3 + 1)));
        let too_many = Some(Err(NotKept::TooManySeries { limit: MAX_ROW }));
        assert_eq!(metrics.state().last, too_many);
        assert_eq!(metrics.lock().0.len(), 0);

        // The store counts every series of its kept reads.
        let mut store = MetricsStore::new(PACE);
        let half = MAX_ROW / 6 + 1;
        store
            .push(secs(0), Reading::of(&jobs(0..half)).unwrap())
            .unwrap();
        let other = Reading::of(&jobs(half..2 * half)).unwrap();
        assert_eq!(store.push(secs(15), other), Err(TooMany));
        assert_eq!(store.len(), 1);
    }

    /// An answer of 60 modules of 10 routes and 60 jobs, in the shape of the recorded lines.
    /// The last route of each module answers 500.
    fn large_answer() -> String {
        use std::fmt::Write;
        let histogram = |text: &mut String, family: &str, bounds: &[&str], labels: &str| {
            for (at, le) in bounds.iter().enumerate() {
                writeln!(text, "{family}_bucket{{{labels},le=\"{le}\"}} {at}").unwrap();
            }
            writeln!(text, "{family}_sum{{{labels}}} 0.25").unwrap();
            writeln!(text, "{family}_count{{{labels}}} {}", bounds.len() - 1).unwrap();
        };
        let seconds = [
            "0.005", "0.01", "0.025", "0.05", "0.075", "0.1", "0.25", "0.5", "0.75", "1", "2.5",
            "5", "7.5", "10", "+Inf",
        ];
        let queries = [
            "0", "1", "2", "5", "10", "25", "50", "100", "250", "500", "1000", "+Inf",
        ];
        let scope = r#"otel_scope_name="nwidart/laravel-metrics""#;
        let route = |module: usize, route: usize| format!("/module-{module}/route-{route}/{{id}}");
        let mut text = String::from("# TYPE http_server_request_duration_seconds histogram\n");
        for (module, at) in (0..60).flat_map(|module| (0..10).map(move |at| (module, at))) {
            let status = if at == 9 { 500 } else { 200 };
            let labels = format!(
                r#"http_request_method="GET",http_response_status_code="{status}",http_route="{}",network_protocol_version="1.1",{scope},url_scheme="http""#,
                route(module, at)
            );
            histogram(&mut text, REQUESTS, &seconds, &labels);
        }
        text.push_str("# TYPE laravel_db_queries histogram\n");
        for (module, at) in (0..60).flat_map(|module| (0..10).map(move |at| (module, at))) {
            let labels = format!(r#"http_route="{}",{scope}"#, route(module, at));
            histogram(&mut text, "laravel_db_queries", &queries, &labels);
        }
        text.push_str("# TYPE messaging_process_duration_seconds histogram\n");
        for module in 0..60 {
            let labels = format!(
                r#"laravel_job_name="Modules\\Module{module}\\Jobs\\Work",messaging_destination_name="default",messaging_operation_name="process",messaging_system="database",{scope}"#
            );
            histogram(&mut text, JOBS, &seconds, &labels);
        }
        text
    }

    #[test]
    fn a_large_answer_is_folded_within_the_budget() {
        let text = large_answer();
        let started = Instant::now();
        let reading = Reading::of(&text).unwrap();
        let took = started.elapsed();
        println!("folded {} bytes in {took:?}", text.len());

        assert_eq!(reading.series.len(), 660);
        assert_eq!(reading.bounds.len(), 14);
        let errors: f64 = reading.series.values().map(|cell| cell.errors).sum();
        assert_eq!(errors, 60.0 * 14.0);
        let job = Subject::Job {
            module: None,
            class: r"Modules\Module7\Jobs\Work".to_owned(),
        };
        let of = |subject: &Subject| {
            let mut cells = reading
                .series
                .iter()
                .filter(|((kept, _), _)| kept == subject);
            let (_, cell) = cells.next().unwrap();
            assert!(cells.next().is_none());
            cell
        };
        assert_eq!(of(&job), &count(14.0));
        let request = Subject::Request {
            module: None,
            route: Some("/module-7/route-0/{id}".to_owned()),
            method: "GET".to_owned(),
        };
        let folded = of(&request);
        assert_eq!((folded.count, folded.errors, folded.sum), (14.0, 0.0, 0.25));
        assert_eq!(folded.buckets, (0..14).map(f64::from).collect::<Vec<_>>());
        assert!(took < secs(1), "took {took:?}");
    }

    #[test]
    fn the_real_clock_adds_the_time_asleep_and_never_goes_back() {
        // What `Instant` and the wall clock ran between calls: awake, a night asleep, the wall clock
        // set back an hour, awake.
        let calls = [(15, 15), (15, 8 * 3600), (15, 0), (15, 15)];
        let (mut instant, mut offset) = (Duration::ZERO, Duration::ZERO);
        let times: Vec<Duration> = calls
            .into_iter()
            .map(|(instant_ran, wall_ran)| {
                offset = ahead(offset, secs(instant_ran), secs(wall_ran));
                instant += secs(instant_ran);
                instant + offset
            })
            .collect();

        let night = 8 * 3600 - 15;
        assert_eq!(
            times,
            [
                secs(15),
                secs(30 + night),
                secs(45 + night),
                secs(60 + night)
            ]
        );
    }
}
