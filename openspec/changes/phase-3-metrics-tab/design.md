# Design

## Context

See `proposal.md` for the reasons. The facts below come from the code at `5df48b9` and from the
archived design of `phase-3-reading-metrics` (called "M20" here).

- `MetricsStore` (M20 D1) keeps running totals per subject: `count`, `errors`, `sum` and one bucket per
  finite bound, bounds lowest first. A kept read is `at`, `covered` and one row of totals. Of the last
  hour one read per 15 s is kept, of the 23 hours before one read per 5 minutes, at most 520 reads.
- `MetricsStore::change(span)` gives what each subject grew by between the newest read and the oldest
  kept read at most `span` before it, over the covered time between them.
- `Figures::of` places each subject with `Placing` (M20 D6) and sums requests and jobs per module over
  `LAST_MINUTE`.
- `Subject::Database` exists only for a series with a `module` label. No recorded series has the label.
- The bucket bounds of laravel-metrics 1.1.0 are 5 ms to 10 s, 14 finite bounds
  (`fixtures/actuator/prometheus-busy.txt`).
- The app has a segmented control (`draw::segmented`), a list that J and K move (`draw::ListNav`), a box
  with a heading (`draw::box_head`) and `RuntimeAction::Choose`, which the runtime box uses to choose an
  environment.

## Goals / Non-Goals

**Goals:**

- S7.3 in full, on the store of M20. Functions are added; the store keeps its data.
- Each rule (the 95% bound, the error share, the database share, a range on thinned reads) is worked out
  in the core and tested once there.
- `metrics/chart.rs` draws with the painter only, and holds no interface text.

**Non-Goals:**

- Limits, colours, alerts (Q16). Storing metrics. A wall clock on the time axis (D6).
- Routes of a module that no kept read holds (D3).

## Decisions

### D1. Every figure of the tab is over the chosen range

```rust
// metrics/figures.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Range { FifteenMinutes, #[default] Hour, Day }
impl Range {
    pub const ALL: [Range; 3] = [Range::FifteenMinutes, Range::Hour, Range::Day];
    pub fn span(self) -> Duration; // 15 min, 1 hour, 24 hours
}
```

The table, the chart and "now" all use one range. The default is 1 hour, as S7.3 draws. `LAST_MINUTE`
goes: M20 D9 already named the range switch as the caller of the same function with another span.

A figure over a range is `change(range.span())` (M20 D1): the newest read against the oldest kept read
at most the span before it, divided by the covered time between the two.

- **15 min and 1 hour** lie in the fine part. The oldest read used is at most 15 s inside the span.
- **24 hours** reaches the thinned part. The oldest read used is the first kept read of the 5 minute
  steps that lie in the span, so at most 5 minutes inside it. The figure covers at least 23 hours
  55 minutes when the reads ran all day, and divides by the time it covers. A kept read holds totals,
  not differences, so the figure equals the one worked out from every read over that time. The test of
  M20 "A range that reaches into the thinned part" already proves this; this change adds no rule there.
- **Gaps.** `covered` does not grow over a gap (M20 D2). A range that holds a gap is the average over
  the time Mortise read, not over the range.

The range switch changes a parameter. It keeps the reads. The environment switch replaces `Connected`
(M20 D4), so it empties them.

### D2. The figures of a row

```rust
// metrics/figures.rs
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rates {
    pub requests_per_minute: f64,
    pub jobs_per_minute: f64,
    /// Decision 5. `None` with no request in the range, or no bucket bound kept.
    pub under_95: Option<Under>,
    /// Decision 6: the share, 0 to 1, of requests answered with a 5xx status. `None` with no request.
    pub errors: Option<f64>,
    /// Decision 3: database time against request time. `None` with no request time, for a route,
    /// and when `Figures::database_known` is false.
    pub database: Option<f64>,
}

/// The 95% figure: a bucket bound, in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Under {
    /// The smallest finite bound with at least 95% of requests at or under it.
    Bound(f64),
    /// More than 5% of requests are over the largest finite bound, which this holds.
    Above(f64),
}
```

The cells of every subject placed in a row are added: `count`, `errors`, `sum` and each bucket. Buckets
add bound by bound, as every cell of a `Change` has one bucket per bound of the store. A series that
lacks a bound of the store has 0 there. Buckets are cumulative, so before a cell is added each bucket
takes the largest bucket at or below its bound.

- **95% under** (decision 5). Buckets are cumulative: the bucket of a bound counts the requests at or
  under it. The figure is the first bound, lowest first, whose bucket is at least `0.95 × count`. When
  no finite bucket reaches it, the rest is in `+Inf`, and the figure is `Above(largest bound)`. No value
  between two bounds is guessed. S7.3 draws "84 ms", which no bucket of laravel-metrics gives. Mortise
  shows "100 ms". This is a deviation (S68).
- **Errors** (decision 6). `errors / count`. A status of 500 to 599 counts (M20, `Reading::of`).
- **Database time** (decision 3). `Database.sum` placed in the row against the `sum` of the requests of
  the row. `Figures::database_known` is true when any kept subject has a `module` label, of any kind.
  When it is false, every row reads "unknown". When it is true, a module with no `Database` subject
  reads 0%. The share can pass 100%: the queries of jobs count, and job time is not in the request
  time. It is shown as it is, with no limit.
- **Per route.** A route has no jobs and no database time: a database series names a module, not a
  route. `jobs_per_minute` is 0 and `database` is `None`, and the table shows an en dash for both.

Rejected: interpolating inside a bucket, as Prometheus `histogram_quantile` does. It shows a precision
the answer does not hold. Rejected: database time against request plus job time. `Reading::of` keeps
no job time, and the plan names request time.

### D3. The scope: all modules, or the routes of one

```rust
// metrics/figures.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope<'a> { All, Module(&'a str) }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RowOf {
    Module(String),
    /// A route of the module, by method and `http_route`. `route` is `None` for a request with a
    /// `module` label and no route.
    Route { method: String, route: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub of: RowOf,
    pub rates: Rates,
    /// Requests per minute over the range, for the chart.
    pub line: Line,
    /// Requests per minute over the last hour, for the column "Last hour".
    pub last_hour: Line,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Figures {
    /// The time the rates cover. `None` before two reads of one run lie in the range.
    pub covered: Option<Duration>,
    /// All: one row per module of the app. Module: one row per route of the module.
    /// Most requests first, then by name (route, then method).
    pub rows: Vec<Row>,
    /// All: "Not in a module". Module: `None`.
    pub outside: Option<Row>,
    /// Everything in the scope: the app, or the module. The chart draws it when no row is selected.
    pub total: Row,
    /// Decision 3: some kept series has a `module` label.
    pub database_known: bool,
}
```

`RowOf` of `total` is `Module` with the module's name, or an empty name for the app. `ModuleFigures`
goes: `Row` takes its place.

- **All.** As in M20: every module of the snapshot has a row, also with zero.
- **One module.** A row per `Request` subject placed in the module, merged by method and route: a
  subject with a `module` label and one without, for the same route, make one row. A route that no
  kept read holds has no row. So a quiet route that was asked for in the last 24 hours shows zero, and
  one that was not is not listed.
  Rejected: a row for every route of `mappings` in the module. Two sources for one list, and placing
  with a label would still add routes that `mappings` does not have.
- A module of `Scope::Module` that the snapshot does not have gives no row and a total of zero.

### D4. A line: the rate between neighbouring kept reads

```rust
// metrics/figures.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// How long before the newest read this point is: zero for the newest.
    pub ago: Duration,
    pub per_minute: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Line {
    /// Oldest first. A new run starts after a gap: the chart draws no segment between two runs.
    pub runs: Vec<Vec<Point>>,
}
impl Line {
    /// The newest point when it is the newest read: "842 now".
    pub fn now(&self) -> Option<f64>;
}
```

```rust
// metrics/sample.rs
pub(crate) struct Step {
    /// The time of the later read.
    pub at: Duration,
    /// The covered time between the two reads. Never zero.
    pub covered: Duration,
    /// What the request count of each kept subject grew by, in the order of `Change::cells`.
    pub counts: Vec<f64>,
    /// `true` for the first step, and for a step after a pair that is no step: a new run of the line.
    pub starts: bool,
}

impl MetricsStore {
    /// The steps between neighbouring kept reads from the oldest read that `change(span)` uses to
    /// the newest. A pair whose two reads are of different runs, or that covers no time, is no step.
    pub(crate) fn steps(&self, span: Duration) -> Vec<Step>;
}
```

Each kept read holds the number of its run. Thinning can drop the first read of a run when a read
before the gap holds its 5 minute step, so `starts_run` alone would join the line across the gap.

`Change` also gains `pub bounds: &'a [f64]`, the bounds of the store, lowest first.

A point is `Σ counts of the row × 60 / covered`, at `ago = newest − at`. A point is a rate over the
time between two kept reads: 15 s in the last hour, 5 minutes before it. A pair across the boundary
of the two parts covers the time between them. So the line of 24 hours has a point every 5 minutes, then
one every 15 s for the last hour. Thinning loses no request: each step holds the difference of totals.

`last_hour` is the same with a span of 1 hour, whatever the range.

Cost: 24 hours keep at most 519 steps. For 660 subjects that is 342,540 numbers (2.7 MB) while the
figures are worked out, then dropped. The lines of 61 rows hold at most 61 × 760 points (1.1 MB). The
app works the figures out again only after a read, or for another snapshot, answer, scope or range.

### D5. The time since the last read

`MetricsState` gains `pub ago: Option<Duration>`: the time from the newest kept read to now, by the
clock of `LiveMetrics`. It is `Some` exactly when `last` is `Some(Ok(()))`. The app asks `state()`
in every frame and asks for a frame again after one second while the table is on screen, so
"last read 8 s ago" counts up between reads. "read every 15 s" is the metrics pace of `Config`.

### D6. The tab

```
[local | staging | production]  [15 min | 1 hour | 24 hours]      read every 15 s, last read 8 s ago
┌ Requests per minute, all modules ─────────────────────────────────────────────────── 842 now ┐
│ 1,500 ─────────────────────────────────────────────────────────────────────────────────────── │
│   ... the line ...                                                                           │
│     0 ─────────────────────────────────────────────────────────────────────────────────────── │
│ 1 hour ago                                                                              now   │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
┌──────┬──────────┬──────────────┬───────────┬────────┬──────────┬───────────────┬───────────┐
│      │ Module   │ Requests/min │ 95% under │ Errors │ Jobs/min │ Database time │ Last hour │
│ CAT  │ Catalog  │          366 │    100 ms │   0.1% │      4.0 │       unknown │  ╱╲╱‾     │
│      │ Not in a module ...                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
lines under the table, as in M20
```

- **Top line.** The environment switch lists the environments of the app; the chosen one is selected.
  A click returns `RuntimeAction::Choose`, as the runtime box does. The range switch returns the range.
  Both are `draw::segmented`. The line at the right is 11.5 px in ink-2.
- **The chart box.** `draw::box_head` with the title "Requests per minute, all modules", "Requests per
  minute, Billing" or "Requests per minute, GET /billing/invoices/{invoice}", and the note "842 now"
  from `Line::now()`, empty without it.
- **Selection.** `OpenApp` keeps `MetricsTab { range: Range, row: Option<RowOf> }`. The chart draws the
  selected row, else `Figures::total`. With a module selected in the index the scope is
  `Scope::Module`, so the total is the module and the rows are its routes. A row is selected by a
  click, or while the table has focus, where J, K, Down, Up, Home and End move it (`ListNav::over`).
  The row is kept by its `RowOf`, because rows sort again after each read. A row that is gone, or a
  change of the index selection, clears it. The range switch keeps it.
- **The table.** Columns of S7.3: the code stamp (34 px), Module (or "Route", not in the spec),
  Requests/min (90), 95% under (80), Errors (70), Jobs/min (70), Database time (100), Last hour (70).
  Figures are right aligned, 13 px ink, no colour. The selected row has the peg (`draw::peg`).
  "unknown" has the tooltip "Needs a newer nwidart/laravel-modules" (not in the spec).
  "Not in a module" is the last row in the scope All.
- **Text.** 95% under: under 1 s in whole ms ("100 ms"), else seconds with at most one decimal
  ("2.5 s"); `Above` reads "over 10 s". Errors: a percent with one decimal ("2.4%", "0.0%"). Database
  time: a whole percent ("31%"). Before two reads every figure is an en dash, as in M20.
- **Time labels.** Under the left end of the time axis "1 hour ago" (or "15 min ago", "24 hours ago"),
  under the right end "now". S7.3 draws wall clock times ("13:10", "14:10"). The clock of the reads is
  not the wall clock (it adds the time asleep, M20 D3), and a wall clock label would make each image
  differ by the machine's time zone. Not in the spec; deviation S69.

### D7. The chart is drawn with the painter

```rust
// mortise-app/src/metrics/chart.rs
/// What the chart draws. All text comes from the caller.
pub struct Plot<'a> {
    pub line: &'a Line,
    pub span: Duration,
    /// The accessible name, such as "Requests per minute, all modules, last hour".
    pub name: &'a str,
    /// The text summary, given as the accessible description.
    pub summary: &'a str,
    /// Under the two ends of the time axis.
    pub from: &'a str,
    pub to: &'a str,
}
/// The height of the chart in its box, as the viewBox of S7.3.
pub const CHART_H: f32 = 132.0;
pub fn chart(ui: &Ui, pal: &Palette, rect: Rect, plot: &Plot<'_>);
/// The sparkline of a row: 58 by 14 px, ink-2, 1.2 px, scaled to the row's own highest point.
pub fn spark(ui: &Ui, pal: &Palette, rect: Rect, line: &Line, span: Duration, name: &str);
```

- The plot runs from 40 px from the left to 10 px from the right, and from 10.5 px to 118.5 px from
  the top, as the viewBox of S7.3. Four grid hairlines in `line`, 1 px: at 0 and at 1, 2 and 3 steps.
- The step is the smallest of 1, 2 and 5 times a power of ten that is at least the highest point
  divided by 3. With no point above 0 the step is 1. The labels are `strings::per_minute` of each
  line, right aligned 6 px left of the plot, mono 10.5 px in ink-2.
- The line: one `PathShape` per run, straight segments, 1.5 px in `ink`. `ago` maps the right end
  (zero) to the left end (the span). A run of one point is a dot of 1.5 px.
- The chart is one node with `Role::Image`, the name, and the summary as its description
  (`accesskit_node_builder`, as `ui/toast.rs` does). The summary is "Now 842, highest 1,020, lowest 120
  requests per minute." or "No rate yet." (not in the spec).
- The sparkline is a node with `Role::Image` and the name "Requests per minute of Catalog, last hour"
  (not in the spec).

## Interfaces between the tasks

Task 21.2 runs first. Tasks 21.3 and 21.4 run in parallel after it, each in its own worktree. 21.5 runs
last.

| Task | Owns | Must not touch |
|---|---|---|
| 21.2 | `mortise-core`: `metrics/figures.rs`, `metrics/sample.rs`, `lib.rs`, `testing.rs`; the app lines that the renames break (`LAST_MINUTE`, `ModuleFigures`, `Figures::modules`), with no new drawing | `mortise-app/src/metrics/view.rs` beyond those lines |
| 21.3 | `mortise-app/src/metrics/chart.rs` and its line in `metrics/mod.rs` | every other file |
| 21.4 | `metrics/view.rs`, `metrics/strings.rs`, `app/metrics.rs`, `app/mod.rs`, `app/workspace.rs`, `tests/metrics.rs` | `metrics/chart.rs`; it calls `chart` and `spark` as D7 gives them |
| 21.5 | `tests/images.rs`, the stored images, `tests/accessibility.rs`, `docs/deviations.md` | `mortise-core` |

`testing.rs` gains, for the images of 21.5:

```rust
impl StubActuator {
    /// Serves `path` with a reply made for each request from how many times it was asked before.
    pub fn serve(&self, path: &str, reply: impl Fn(usize) -> Reply + Send + Sync + 'static);
}
```

## Risks / Trade-offs

- [The 95% figure is coarse: "100 ms" covers 76 to 100 ms] → Decision 5, recorded as S68. The bound
  is what the app measured; nothing is guessed.
- [Database time can pass 100%] → Decision 3 names request time. Shown as it is.
- [A route that no read holds is not listed for its module] → D3. The Summary lists every route of a
  module from `mappings`.
- [The images of a 24 hour range need a day of reads] → The stub serves a reply per request and the
  clock steps 15 s per request, so a day is 5,760 reads of a short answer. If it takes over 10 s in a
  debug build, the image fills fewer hours and says so in the test.
- [Figures for 61 rows with lines are worked out on the interface thread] → Once per read (15 s) and per
  change of scope or range, within the budget of D4.
