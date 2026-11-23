# Design

## Context

See `proposal.md` for the reasons. The facts below come from the recorded answers in
`fixtures/actuator/` (see the design of `phase-3-groundwork`) and from the code at `4b98d8a`.

- The actuator client is `runtime/actuator.rs`. `fetch()` reads JSON. One worker thread per
  environment (`runtime/live.rs`, `LiveRuntime`) asks `health` every 15 s while the app calls `seen()`.
- The worker has no `AppSnapshot`. The app holds it on the interface thread.
- An answer for 60 modules of 10 routes is about 3.3 MB, about 19,000 lines. Each further status of a
  route adds about 4 KB, so 600 routes with three statuses each are about 8 MB.
- `ureq` 3.5.1, read in `src/body/mod.rs` and `limit.rs`: the limit of `read_to_string()` (10 MB) wraps
  the socket, below the gzip decoder. It counts compressed bytes. The workspace turns `gzip` on, and
  the busy answer shrinks 25 times under gzip. So that limit does not bound memory. `LimitReader` also
  fails a body of exactly the limit.
- No series has a `module` label today. `http_route` is `/` followed by the `uri` of `mappings` without
  leading slashes. The domain is dropped. A 404 or 405 has no `http_route`.

## Goals / Non-Goals

**Goals:**

- One store that milestone 21 reads for every figure of S7.3. Milestone 21 adds functions, not data.
- Memory flat after 24 hours, with a bound a test can state.
- No work for a 3.3 MB answer on the interface thread.
- Tasks 20.2, 20.3 and 20.5 build in parallel without touching the same lines.

**Non-Goals:**

- The chart, ranges, the 95% bound, the error share, database time, figures per route, J and K.
- Reading `modules`. Storing anything. Limits or colours on a figure.

## Decisions

### D1. A kept read holds running totals; older reads are thinned

What is kept is keyed by a **subject**, which is what the answer says and not a module:

```rust
// metrics/sample.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Subject {
    /// A request series. Statuses are added into one subject.
    Request { module: Option<String>, route: Option<String>, method: String },
    /// A job series. `class` is empty when the series has no `laravel_job_name`.
    Job { module: Option<String>, class: String },
    /// Database time. Kept only for a series with a `module` label (decision 3).
    Database { module: String },
}

/// The counters of one series, or what a subject grew by. All are counts or seconds, so all are `f64`.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Cell {
    pub count: f64,        // `_count`
    pub errors: f64,       // `_count` of a series with a 5xx status
    pub sum: f64,          // `_sum`, seconds
    pub buckets: Vec<f64>, // `_bucket` per finite bound, as the answer gives them. Requests only
}

/// One answer, folded. This is the interface between the parser and the store.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Reading {
    /// The finite bucket bounds of the request histogram, lowest first.
    pub bounds: Vec<f64>,
    /// The counters of each series, by subject and its labels except `le`, sorted by name.
    pub series: HashMap<(Subject, String), Cell>,
}
```

The store turns the counters of each reading into **running totals**: a total starts at zero and grows
by what each read adds (D2). A kept read is `at`, `covered` (the time covered so far, D2) and one row of
totals: `3 + bounds.len()` numbers per subject, in one flat `Vec<f64>` indexed by subject. A subject
unknown at an older read has no column there, which reads as zero.

A figure over a span is then `newest − oldest read in the span`, divided by the difference of
`covered`. It needs two reads, not all of them. So thinning loses nothing for a figure:

- Of the last hour, one read per pace (15 s) is kept: a read in the step of the read before takes its
  place, so the newest read is always kept. At most 241, however fast "Try again" reads.
- Of the 23 hours before, the first read of each 5 minute step is kept: at most 277.
- A read older than 24 hours is dropped. The store holds at most 520 reads.
- A subject that did not change in the whole kept window and is not in the newest read is dropped,
  and the rows are packed. So a renamed route does not stay for ever.

Numbers, with the 14 finite bounds of the recorded app (17 numbers, 136 bytes per subject and read):

| Case | Subjects | One read | 520 reads | 5,760 reads, not thinned |
|---|---|---|---|---|
| The recorded app | 19 | 2.6 KB | 1.3 MB | 15 MB |
| 60 modules of 10 routes, 60 jobs | 660 | 90 KB | 47 MB | 517 MB |

Several statuses of a route add no row, because statuses add into one subject. Each status keeps its
own base (D2), so a status costs one more base, not one more row per read. The bound is
`520 × subjects × (3 + bounds) × 8` bytes.

**The cap.** Bucket bounds come from the answer, so many distinct `le` values times many subjects
could need far more than the body limit (D7) suggests. A read keeps at most `MAX_ROW = 32,768`
numbers: `3 + bounds` for each series. `Reading::of` checks it before a new series or a new bound
grows the reading. `push` checks it for the store: every series of the kept reads, plus the new ones
of the reading. A reading over the cap is refused as a whole: nothing is pushed, and the Metrics tab
says so (D7). The 60 module app keeps 660 × 17 = 11,220; 600 routes with three statuses and 60 jobs
keep 1,860 × 17 = 31,620. So the store holds at most 520 × 32,768 × 8 B ≈ 136 MB of totals, plus one
base per series (256 KB).

What milestone 21 reads from the same totals: the request rate (`count`), the 5xx share
(`errors / count`), the 95% bound (`buckets` and `bounds`), the job rate, the database share
(`Database.sum` against the `sum` of the module's requests), each per subject, so per route. A chart
series is the difference between neighbouring kept reads: a point every 15 s in the last hour and every
5 minutes before.

Rejected:

- Keep every read for 24 hours, as section 4 of the plan words it. 517 MB for the 60 module case.
  **This design differs from that sentence of the plan. The lead confirmed it on 23 November 2026.**
- Keep totals per module only. Milestone 21 needs figures per route and would need a second store.
- Keep differences per read. A figure over 24 hours would sum 520 rows, and thinning would have to
  add rows together.
- Place a series in a module when it is read. The worker has no `AppSnapshot`, and a change of the
  files or of `mappings` would not re-place what is kept (D6).
- Leave the buckets to milestone 21. It would change the row layout and the bound after the fact.

### D2. Resets, gaps, new series

The store keeps, per series, the counters of the last read that held it (the base). A series is one
label set of the answer, `le` aside, sorted by name, so a status, a `url_scheme` or a queue each make
their own series. Totals stay per subject.
`MetricsStore::push(at, reading)` applies these rules, in this order:

0. A reading over the cap of D1 is refused as a whole and changes nothing.
1. A reading whose `bounds` differ from the kept ones empties the store first. A reading with no
   request histogram has no `bounds`; it keeps the kept ones and empties nothing. When the store
   holds no bounds yet, it takes those of the reading and gives every kept row a zero per bucket.
2. `at` not later than the newest kept read: the reading is dropped.
3. **First read of a run.** A run starts with the first read, and with a read more than 3 paces (45 s)
   after the one before it. Every older base becomes stale. Every base of the reading is set. Nothing
   is added. `covered` does not grow. One missed read (30 s) is not a gap.
4. **Otherwise** `covered` grows by the time since the read before, and for each series of the reading:
   - Every counter is at or above its base: its subject's totals grow by the differences.
   - Any counter is below its base: a **reset**. Nothing is added for this series. The base is set.
     The next read gives its next rate. Another status of the subject still adds what it grew by.
   - The base is stale: the series was missing when the run started, so what it did in the gap is not
     known. Nothing is added. The base is set.
   - The series has no base (it is new): it counts from zero, so the totals grow by its counters.
5. A series that is missing from the reading keeps its base, and its subject keeps its totals.
6. A read in the same pace step as the read before takes its place (D1).

A rate is `Δtotal / Δcovered`. Both only grow, so no rate is negative. Before two reads of one run lie
in the span, `Δcovered` is zero and there is no rate. When the newest read is the first of its run,
there is no rate either, though the span reaches into the run before: that rate would be the old one.

Why a rate across a gap is not allowed: the newest two reads would give the average of an hour as
"now", and a reset inside the gap cannot be seen. Mortise shows what it covered.

Why a new series counts from zero: the package creates a series at its first observation. The first
5xx of a route is a new series, and dropping its first read would lose most errors.

Why the base of a missing series is kept: if it comes back with a higher value, only the difference
counts. This matters when a load balancer shows one server per read (risk below). After a gap the base
is stale, because the difference would hold the gap.

Why a base per series: series of several labels add into one subject. A reset of the 404s (100 to 1) would hide the
growth of the 200s (100 to 110) if the reset rule ran on the sum.

Rejected: adding the whole new value at a reset, as Prometheus `increase` does. With two servers behind
one address the counters go down at every other read, and each would add a whole counter.

### D3. The clock is a parameter

`MetricsStore::push(at: Duration, reading)` takes the time as a `Duration` since any fixed start. The
store never asks the time. A test of 24 hours calls `push(15 s × n, …)` 5,760 times and takes
milliseconds. Milestone 23 writes that test against `MetricsStore::len()` and the bound of D1.

`LiveMetrics` (D4) owns the clock of a real run: `pub type MetricsClock = Arc<dyn Fn() -> Duration + Send + Sync>`.
`LiveMetrics::new()` uses the real one. `LiveMetrics::with_clock(clock)` takes another, for the image
tests of the app.

The real clock is `Instant` since the start, plus the time the computer slept. The standard library
does not promise that `Instant` moves during sleep, and on macOS and Linux it does not. Without the
correction a read after a night of sleep is "15 s" after the one before, and gives a false peak.
The clock compares `Instant` with `SystemTime`: when the wall clock ran further than `Instant` since
the last call, the difference is added and stays added. The result never goes back. A wall clock that
is set forward makes one false gap, which is harmless. This is a pure function of two durations and an
offset, tested once. Not run on Windows or Linux.

Rejected: a `Clock` trait that the store holds. One more type, and the store has no reason to ask.
Rejected: `Instant` as the parameter. A test cannot name an `Instant` 24 hours away without arithmetic
on `now()`.

### D4. Threading: one worker, a sink, a shared store

```
interface thread                         worker thread (runtime/live.rs)
  live.seen()            ───────────────▶ visible
  live.metrics_seen()    ───────────────▶ the tab is open
                                          ask_metrics()            (runtime/actuator.rs)
                                          on_metrics(Result<String, NoMetrics>)
                                            = LiveMetrics::sink(): fold the text, then lock and push
  metrics.state()        ◀── Arc<Mutex> ─ on_change() → repaint
  metrics.figures(...)
```

- **The request.** The worker loop gets one new branch. `prometheus` is due when all hold: the window
  is visible; `metrics_seen()` was called within `pace.seen`; the last answer is `Answered` and lists
  `prometheus`; the last metrics read ended `pace.metrics` (15 s) ago or more, or there was none.
  The app calls `metrics_seen()` in each frame that draws the Metrics tab, as it calls `seen()`.
  Nothing has to be undone when the tab closes.
- **Order.** `health` or "everything" goes first when both are due. The loop then turns again and
  reads metrics. One request at a time per environment. A metrics read can delay `health` by its own
  length: at most 10 s (`timeout_global` covers the body) plus the fold (D8). It never skips one.
  A metrics read does not set `RuntimeState::asking` and does not change `RuntimeState::answer`.
- **`refresh()`** also makes metrics due at once, so "Try again" reads them again.
- **Parsing** runs inside the sink, on the worker thread, with no lock held. The lock is taken only to
  push the folded reading (microseconds).
- **Reading figures.** `LiveMetrics::state()` clones a small struct under the lock. The app works out
  `figures()` again only when `state().serial` changed or the snapshot or the runtime answer is another
  `Arc`. Else it draws the figures it has.
- **Environment switch.** `Connected` holds the `LiveMetrics` beside the `LiveRuntime`.
  `sync_runtime` already replaces `Connected` when the environment or the secret changes, and drops it
  when another app is on screen. The reads go with it. A read still on its way writes into the old
  store, which nobody holds any more.

Rejected: a second worker for metrics. Two requests at once to a production app, and the owner fixed
one worker. Rejected: the text in `RuntimeState` for the app to parse. That is 3.3 MB on the
interface thread.

### D5. The module order: the worker takes a closure

The order of core modules becomes: `php` and `program`; `modules`; `boundary`; `package`; `findings`;
`run`; `actions`; `generate`; `app`; `surface`, `runtime` and `palette`; `metrics`; `editor`, then
`settings`.

`metrics` names `runtime` (`Route`, `module_of`, `NoMetrics`), `app` (`AppSnapshot`) and `modules`.
`runtime` names nothing of `metrics`:

- The type of a metrics answer, `Result<String, NoMetrics>`, is defined in `runtime/actuator.rs`.
- The worker gets `on_metrics: impl FnMut(Result<String, NoMetrics>) + Send + 'static` when it opens.
  It does not know what the closure does.
- `metrics::LiveMetrics::sink()` returns that closure. `mortise-app` joins the two in `sync_runtime`.

Rejected: the store in `runtime/`. The plan puts it in `metrics/`, and `runtime/live.rs` is shared
with milestone 19. Rejected: `metrics` before `runtime`. Placing needs `Route` and `module_of`.

### D6. Placing runs when figures are worked out

`metrics/place.rs` builds a `Placing` from the `AppSnapshot` and the routes of the last full answer,
on the interface thread, each time figures are worked out (at most once per read: 600 routes against
60 namespaces is well under 1 ms). So what is kept is always placed by the files and the `mappings` of
now, and the worker needs no snapshot.

```rust
// metrics/place.rs
pub(crate) struct Placing<'a> { /* "/" + uri → [(methods, module)] ; the app for module_of */ }
impl<'a> Placing<'a> {
    pub(crate) fn new(app: &'a AppSnapshot, routes: Option<&[Route]>) -> Self;
    /// The name of the module, or `None` for "Not in a module".
    pub(crate) fn place(&self, subject: &Subject) -> Option<&'a str>;
}
```

Rules of `place`, first match wins:

1. A `module` label. Empty counts as no label. `none`, in any case, is "Not in a module". A value that
   equals a module name without regard to ASCII case is that module (milestone 19 matches indicators
   the same way). Any other value is "Not in a module": decision 2 says a series that matches nothing
   counts there.
2. `Request` with a route: the key is the `http_route` as written (case matters, as in Laravel).
   A route of `mappings` has the key `/` + `uri` without leading slashes. The candidates are the routes
   with that key whose `methods` hold the series' method. With none, the candidates are all routes with
   that key. The module of a route is `module_of(app, &route.action)`. When every candidate is in the
   same module, that is the place. Else "Not in a module".
   This one rule covers the domain: `Route` has no domain field and needs none. Two routes with one
   path and one method can only differ by domain; in two modules they are "Not in a module".
3. `Request` with no route (404, 405): "Not in a module".
4. `Job`: `module_of(app, &class)`. The parser has already turned `\\` into `\`.
5. `Database` has a label by construction, so rule 1 places it.

Without `mappings` (`routes` is `None`), rule 2 has no candidate. The table then adds one line that
says so.

Rejected: placing in the worker at each read. It needs the snapshot on the worker and freezes old
reads on old routes. Rejected: an enum `Place` that says how a series was placed. Nothing reads it yet.

### D7. The body limit and the type of an answer

```rust
// runtime/actuator.rs
/// The most text Mortise reads from one `prometheus` answer, after decompression.
pub const METRICS_BODY_LIMIT: u64 = 16 * 1024 * 1024;

/// Why a read of `prometheus` gave no metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoMetrics {
    /// 200 with an empty body: metrics are off in the app.
    Off,
    /// 503: the first line of the body as the app wrote it, at most 200 characters.
    Unavailable { url: String, line: String },
    /// The body is longer than the limit.
    TooLarge { url: String, limit: u64 },
    NoAnswer { url: String, message: String },
    NotAuthorized { url: String, status: u16 },
    Unexpected { url: String, status: u16, location: Option<String> },
}

/// Asks an environment for `prometheus`. `Ok` is the text of a 200 that is not empty.
/// This waits for the app: call it from a worker thread.
pub fn ask_metrics(environment: &Environment, secret: &Secret) -> Result<String, NoMetrics>;
```

- 16 MiB is five times the 3.3 MB of 60 modules, and twice the 8 MB of 600 routes with three statuses.
- The limit counts decoded bytes. `Client::get` gains a limit parameter: `fetch()` keeps today's
  `read_to_string()`; the text read uses `body_mut().with_config().reader()` (no `ureq` limit), then
  `take(limit + 1)` and `read_to_end`. More than `limit` bytes is `TooLarge`. Bytes that are not UTF-8
  are replaced (`String::from_utf8_lossy`), so one bad byte does not lose the read.
- The request keeps `Accept: application/json`. The recorded endpoint answers text to it, and Laravel
  then answers 401 as JSON, not as a redirect to a login page.
- Only the first line of a 503 is kept. A 503 of maintenance mode can be a whole page.
- "Not exposed" is not a variant: the worker does not ask (D4). A 404 is `Unexpected`.

How it maps to the tab, first match wins (`app/metrics.rs`):

| What is known | The tab shows |
|---|---|
| No `Connected` for the app (no chosen environment, or no actuator in the app) | S7.1 |
| No runtime answer yet | "Reading metrics from {environment}" (not in the spec) |
| The runtime answer is `NoAnswer`, `NotAuthorized` or `Unexpected` | The sentence of the runtime box, "Try again", "Open settings" |
| `Answered`, and `prometheus` is not listed | S7.2 |
| No metrics read ended yet | "Reading metrics from {environment}" |
| `Err(Off)` | The empty sentence, "Try again" |
| `Err(Unavailable)` | The 503 sentence with the line, "Try again" |
| `Err(TooLarge)` | The too large sentence, "Try again" |
| Too many series (D1) | The too many sentence, "Try again" |
| `Err(NoAnswer / NotAuthorized / Unexpected)` | The sentence of the runtime box for that address, "Try again", "Open settings" |
| `Ok` | The table |

A failed read after good ones shows its sentence, not an old table. The kept reads stay, and age out.

### D8. The budget

For a generated answer of 600 routes, one status each, and 60 jobs (about 3.3 MB):

- **Time.** `Reading::of(text)` ends within 50 ms in a release build on the owner's Mac. Task 20.7
  measures it and writes the number under "Measured" below. The standing test allows 1 s in a debug
  build, so it fails on a quadratic mistake and not on a slow CI machine.
- **Memory.** Beside the text, a read allocates less than 2 MB. The parser gives samples that borrow
  from the text, one at a time; nothing holds all samples. The result is one `Reading` of 660 cells
  (about 170 KB). The workspace forbids `unsafe`, so a test cannot count allocations with its own
  allocator. Task 20.7 measures the peak size of the test process with `/usr/bin/time -l` on macOS,
  with and without the read, and writes both numbers under "Measured".
- **The store.** The bound of D1: 47 MB for this case with a full window, 136 MB for any answer.

### D9. The figures of the plain table

```rust
// metrics/figures.rs
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rates { pub requests_per_minute: f64, pub jobs_per_minute: f64 }

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleFigures { pub module: String, pub rates: Rates }

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Figures {
    /// The time the rates cover. `None` before two reads of one run lie in the span.
    pub covered: Option<Duration>,
    /// One row per module of the app: most requests first, then by name.
    pub modules: Vec<ModuleFigures>,
    /// "Not in a module".
    pub outside: Rates,
}

/// The span of the table of milestone 20.
pub const LAST_MINUTE: Duration = Duration::from_secs(60);
```

- The span is **the last minute**: from the newest read back to the oldest kept read at most 60 s
  before it, over the covered time. With two reads it is their 15 s. With five it is a minute.
  Rejected: since the last read only. One request in 15 s reads as 4 per minute, then 0.
  Rejected: the whole kept window. That is the range switch of milestone 21, which calls the same
  function with 15 minutes, 1 hour or 24 hours.
- Every module of the snapshot has a row, also with zero. "Not in a module" is always the last row.
- Before two reads of one run exist, and while the newest read is the first of its run, `covered` is
  `None`. Each figure cell shows an en dash, and one
  line under the table reads "A rate needs two reads. The second read comes 15 s after the first."
- A figure under 10 shows one decimal ("0.5"), else a whole number ("366"). Zero shows "0".
- The table is the same with a module selected. Routes of one module come in milestone 21.
- Milestone 21 adds fields to `Rates` and a list of routes. It adds no type beside them.

## Interfaces between the tasks

Tasks 20.2, 20.3 and 20.5 run in parallel. Each owns its files. The types above are the contract.

| Task | Owns | Must not touch |
|---|---|---|
| 20.2 | `metrics/prometheus.rs` | `metrics/sample.rs`, `runtime/` |
| 20.3 | `metrics/sample.rs`: `Subject`, `Cell`, `Reading`, `MetricsStore`, `Change` | `metrics/prometheus.rs`, `runtime/` |
| 20.5 | `runtime/actuator.rs`, `runtime/live.rs`, `testing.rs`, `tests/actuator.rs`, the `runtime` lines of `lib.rs` | `metrics/` |
| 20.4 | `metrics/place.rs`, `metrics/figures.rs`; adds `Reading::of`, `LiveMetrics`, `MetricsState` and the real clock to `sample.rs` | `runtime/` |
| 20.6 | `mortise-app` | `mortise-core`, except a missing re-export |

**`metrics/mod.rs` and `lib.rs`.** 20.2 and 20.3 each create `metrics/mod.rs` with their own line and
add `#[cfg(feature = "actuator")] mod metrics;` to `lib.rs` after `mod generate;`. The lead merges the
two files to this, which 20.4 then finishes:

```rust
//! The metrics of a running app: the text format, the reads kept in memory, the place of a series
//! in a module and the figures per module. Nothing here is written to disk.

// Task 20.4 uses both and removes these two attributes.
#[allow(dead_code)]
pub(crate) mod prometheus;
#[allow(dead_code)]
pub(crate) mod sample;
```

The `allow` is needed because `core-features` and clippy fail on code that only tests use.
After 20.4 no `allow(dead_code)` is left in `metrics/`.

**The parser (20.2).**

```rust
// metrics/prometheus.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind { Counter, Gauge, Histogram }

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Part {
    /// The value of a counter or a gauge.
    Value,
    /// `_bucket`, with its `le` bound. `+Inf` is `f64::INFINITY`.
    Bucket(f64),
    Sum,
    Count,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sample<'a> {
    /// The name on the `# TYPE` line, such as `http_server_request_duration_seconds`.
    pub family: &'a str,
    pub kind: Kind,
    pub part: Part,
    /// The labels in the order written, without `le`. A value is copied only when it has an escape.
    pub labels: Vec<(&'a str, Cow<'a, str>)>,
    pub value: f64,
}

impl Sample<'_> {
    pub(crate) fn label(&self, name: &str) -> Option<&str>;
}

/// The samples of an answer, in its order. A line that cannot be read is skipped.
pub(crate) fn samples(text: &str) -> impl Iterator<Item = Sample<'_>>;
```

Rules: a `# TYPE name kind` line sets the current family. A sample belongs to it when its name is the
family name (counter, gauge) or the family name with `_bucket`, `_sum` or `_count` (histogram). Any
other sample is skipped, as is a family of another kind. `# HELP` and other comments are skipped. A
timestamp after the value is ignored. Label escapes are `\\`, `\"` and `\n`; any other backslash stays
as written. A value is what `f64::from_str` reads (`+Inf`, `NaN`); else the line is skipped.
Lines end with `\n` or `\r\n`.

**The store (20.3).**

```rust
// metrics/sample.rs
pub(crate) struct MetricsStore { /* … */ }

/// What changed over a span.
pub(crate) struct Change<'a> {
    /// The time between reads that the span covers. Never zero.
    pub covered: Duration,
    /// Every kept subject with what it grew by. Zero is included.
    pub cells: Vec<(&'a Subject, Cell)>,
}

impl MetricsStore {
    /// `every` is the pace of the reads. A gap is longer than 3 times it.
    pub(crate) fn new(every: Duration) -> Self;
    /// `Err` when the reading is over the cap of D1. It then changes nothing.
    pub(crate) fn push(&mut self, at: Duration, reading: Reading) -> Result<(), TooMany>;
    /// How many reads are kept.
    pub(crate) fn len(&self) -> usize;
    /// The newest read against the oldest kept read at most `span` before it.
    /// `None` when no time is covered between them, or when the newest read starts a run.
    pub(crate) fn change(&self, span: Duration) -> Option<Change<'_>>;
}
```

Constants in `sample.rs`: the window 24 hours, the fine part 1 hour, the step 5 minutes, the gap
3 paces, the cap `MAX_ROW` 32,768 numbers per read. The store ignores a counter that is negative or not finite. Tests of 20.3 build `Reading`s by
hand; they need no text.

**The client (20.5).** `ask_metrics`, `NoMetrics` and `METRICS_BODY_LIMIT` as in D7, and:

```rust
// runtime/live.rs
pub struct Pace { pub status: Duration, pub everything: Duration, pub seen: Duration,
                  /// Between two requests for `prometheus`. 15 s.
                  pub metrics: Duration }

impl LiveRuntime {
    /// As `open_with`, and hands every answer of `prometheus` to `on_metrics`, on the worker thread.
    pub fn open_with_metrics(
        environment: Environment, secret: Secret, pace: Pace,
        on_change: impl Fn() + Send + 'static,
        on_metrics: impl FnMut(Result<String, NoMetrics>) + Send + 'static,
    ) -> Self;
    /// Tells the worker that the Metrics tab drew itself. Without this it does not ask for `prometheus`.
    pub fn metrics_seen(&self);
}
```

`open` and `open_with` keep their signatures and pass a closure that does nothing. `Shared` gains
`metrics_seen: Option<Instant>`. `lib.rs` re-exports `ask_metrics`, `NoMetrics` and
`METRICS_BODY_LIMIT`. `testing.rs` gains `StubActuator::recorded()`, which serves the files of
`fixtures/actuator/` with `include_str!` and lists `env` among the links.

**The join (20.4).**

```rust
// metrics/sample.rs
impl Reading {
    /// Folds an answer. Runs on the worker thread. `Err` when it is over the cap of D1.
    pub(crate) fn of(text: &str) -> Result<Reading, TooMany>;
}

pub type MetricsClock = Arc<dyn Fn() -> Duration + Send + Sync>;

/// Why a read added nothing to the store. Owned by `metrics`: `runtime` names nothing of it (D5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotKept {
    NoMetrics(NoMetrics),
    /// More than `limit` numbers per read (D1).
    TooManySeries { limit: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetricsState {
    /// What the last read gave. `None` before the first read ends.
    pub last: Option<Result<(), NotKept>>,
    /// Goes up by one when a read ends.
    pub serial: u64,
}

/// The reads of one environment, shared between the worker and the interface.
#[derive(Clone)]
pub struct LiveMetrics { /* Arc<Mutex<(MetricsStore, MetricsState)>>, MetricsClock */ }

impl LiveMetrics {
    pub fn new() -> Self;
    pub fn with_clock(clock: MetricsClock) -> Self;
    /// For `LiveRuntime::open_with_metrics`.
    pub fn sink(&self) -> impl FnMut(Result<String, NoMetrics>) + Send + 'static;
    pub fn state(&self) -> MetricsState;
    /// `routes` is `None` when `mappings` is not exposed.
    pub fn figures(&self, app: &AppSnapshot, routes: Option<&[Route]>, span: Duration) -> Figures;
}
```

`Reading::of` folds three families and skips the rest:

| Family | Subject | Fields |
|---|---|---|
| `http_server_request_duration_seconds` | `Request` from `module`, `http_route`, `http_request_method` | `_count` adds to `count`, and to `errors` when `http_response_status_code` is 500 to 599. `_sum` adds to `sum`. A finite `_bucket` adds to its bucket. `+Inf` is dropped: it equals the count |
| `messaging_process_duration_seconds` | `Job` from `module`, `laravel_job_name` | `_count` adds to `count` |
| `db_client_operation_duration_seconds`, only with a `module` label | `Database` | `_sum` adds to `sum` |

The `LiveMetrics` store uses `Pace::default().metrics` as its pace. `lib.rs` re-exports `Figures`,
`ModuleFigures`, `Rates`, `LAST_MINUTE`, `LiveMetrics`, `MetricsState`, `NotKept` and `MetricsClock`.

**The app (20.6).**

- `Config` gains `pace: Pace` and `metrics_clock: Option<MetricsClock>`. `main.rs` passes the defaults.
  A test passes a short pace and a clock that steps 15 s per call, so two reads give the same figures
  on every run.
- `Connected` gains `metrics: LiveMetrics`. `sync_runtime` opens with `open_with_metrics` and
  `metrics.sink()`.
- `app/metrics.rs` calls `connected.live.metrics_seen()` in each frame that draws the Metrics tab of
  the app on screen, builds what the tab shows (the table of D7) and keeps the last `Figures` with the
  `serial` they were worked out for.
- `metrics/view.rs` draws it and returns `WorkAction::Runtime(RuntimeAction::TryAgain)` or
  `OpenSettings`. S7.1 and S7.2 use `empty_state` with `Icon::Metrics`. "Connect an environment"
  returns `RuntimeAction::OpenSettings`.
- `Tab::explanation`, `METRICS_UNAVAILABLE_TITLE` and `METRICS_UNAVAILABLE` go.
- Text that two features now show moves to `mortise-app/src/strings.rs`: `TRY_AGAIN`, `cannot_reach`,
  `refused_credentials`, `answered_unexpected` (from `runtime/strings.rs`) and `NOT_IN_A_MODULE`
  (from `summary/strings.rs`). The words do not change.

Text of `metrics/strings.rs`. S7.1 and S7.2 are the spec's words. The rest is marked "not in the spec".

| Where | Text |
|---|---|
| S7.1 title | Metrics are not connected |
| S7.1 paragraph | Metrics come from the actuator of the running app. Connect an environment for {app} to read requests, response time and errors per module. |
| S7.1 button | Connect an environment |
| S7.2 title | Metrics are not exposed on {environment} |
| S7.2 paragraph | Add prometheus to exposure.include in config/actuator.php, next to health. |
| Waiting | Reading metrics from {environment} |
| Empty, title | Metrics are off on {environment} |
| Empty, sentence | The app answers with no metrics. Set METRICS_ENABLED=true in its .env to turn them on. |
| 503, title | Metrics cannot be read on {environment} |
| 503, sentence | {environment} answered 503 for {path}: "{line}" Check the metrics store of the app, then try again. |
| Too large, title | The metrics of {environment} are too large |
| Too large, sentence | The answer is larger than 16 MB, so Mortise does not read it. Export fewer series from the app. |
| Too many, title | The metrics of {environment} have too many series |
| Too many, sentence | Mortise keeps at most 32,768 figures per read. Export fewer series from the app. |
| Columns | Module, Requests/min, Jobs/min |
| Under the table, one read | A rate needs two reads. The second read comes 15 s after the first. |
| Under the table, no `mappings` | Requests are not placed in a module, because mappings is not exposed on {environment}. |

## Measured

Measured on 2026-11-23 on an Apple M3 Max with 128 GB of memory (137438953472 bytes), macOS, release
build. Nothing ran on Windows or Linux.

**Time.** `a_large_answer_is_folded_within_the_budget` folds the generated answer of 4.6, 3678352 bytes,
in 11.5 ms, 5.6 ms and 5.8 ms in three runs. The first run is the slowest, as the caches are cold. The
budget of D8 is 50 ms. All three runs are within it.

**Memory.** `/usr/bin/time -l` on the release test binary, three runs each:

| Test | Maximum resident set size |
|---|---|
| `a_large_answer_is_folded_within_the_budget` (builds the text and reads it) | 12.6 MB (12615680 to 12648448 bytes) |
| `the_real_clock_adds_the_time_asleep_and_never_goes_back` (no answer) | 7.06 MB (7061504 to 7077888 bytes) |

The difference is 5.55 MB. The text is 3.68 MB, so 1.87 MB is left. That includes the growth of the
`String` that builds the text (it doubles, and the old block lives while the new one is made) and the
`Reading` of about 170 KB. The read alone is below 1.87 MB. The budget of D8 is under 2 MB beside the
text, so it holds. The number is an upper bound, because the test cannot tell the read from the builder
of the text.

## Risks / Trade-offs

- [Thinning differs from section 4 of the plan] → A figure over any span is unchanged (D1). The chart
  of milestone 21 gets a point per 5 minutes beyond the last hour. Milestone 23 updates the plan.
- [47 MB for a large app with a full window; 136 MB in the worst case the cap allows] →
  Stated as the bound. Milestone 23 measures a 24 hour run.
- [An app with more series than the cap shows no table] → The tab says so and names the cap. Series
  of 24 hours count, so an app whose routes change often can stay over the cap until old series age
  out.
- [A cache store behind a load balancer shows one server per read] → Counters go up and down between
  reads. D2 never adds a whole counter, so the figures are too low, not too high. Not solved here.
- [Values reach the store of the app after the response] → A rate can lag one read. The last minute
  smooths it.
- [A read every 15 s adds two database operations in the app, with a database cache store] → Reads
  stop when the tab closes or the window hides.
- [The limit is not tested against a compressed answer] → The stub does not compress. That the limit
  counts decoded bytes rests on reading `ureq` 3.5.1.
- [Milestone 19 edits `runtime/actuator.rs` and `runtime/live.rs`] → This change adds a function, a
  parameter of `Client::get`, one field of `Pace` and `Shared`, one constructor and one branch.
  If both are ready, 19 merges first and this change rebases.
- [`Config.pace` and `Config.metrics_clock` exist for tests] → As `quick_read` and `pick_folder` do.
  Without them the image of the table waits 15 s and differs on every run.
- [Another app on the rail drops the reads] → The runtime of an app that is not on screen is already
  closed. The reads start again when the app comes back.
