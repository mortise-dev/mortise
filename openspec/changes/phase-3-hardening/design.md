# Design

## Context

See `proposal.md` for why. Facts this design builds on:

- `MetricsStore` (`metrics/sample.rs`) takes the time as a parameter (M20 D3). It keeps at most 519
  reads; `a_day_of_reads_is_thinned_and_keeps_its_figures` asserts `len() <= 520` over 7,000 reads of
  two subjects. Nothing measures bytes. M20 D1 states the bound
  `520 × subjects × (3 + bounds) × 8` bytes of totals: 47 MB for 60 modules of 10 routes and 60 jobs,
  about 145 MB in the worst case the caps allow.
- `follow_metrics()` in `app/metrics.rs` calls `LiveMetrics::figures` on the interface thread after each
  read and after a change of snapshot, answer, scope, range or selected row (M21 D4). A 24 hour range
  walks at most 519 steps per subject. Nobody measured how long that takes.
- The `workspace-unsafe` rule forbids `unsafe`, so a counting global allocator is not available.
- "Real PHP" in `ci.yml` builds a Laravel app with Composer and runs `tests/real_php.rs` with
  `--ignored`. `fixtures/actuator/README.md` says how the app of milestone 18 was made by hand.
- `draw::segmented` paints one track of fixed width. S7.3 puts the range switch on a second line when
  both switches do not fit; nothing handles an environment switch wider than the tab. The runtime box
  paints its switch right-aligned in the heading and can run over the title and the left edge.
- `docs/spec-build/build.mjs`, `shoot.sh` and `offs.sh` name an absolute folder outside the repository
  (`.../LaravelModules/docs/companion-app/design-fable`). `lib.py` was fixed in milestone 18.

## Goals / Non-Goals

**Goals:**

- A test that fails when a full window holds more bytes than the stated bound, or grows after it is full.
- A number for the figure cost, written below, and a rule that decides from it.
- A live check of the client against the three packages that warns, in CI, when a release changes an
  answer.

**Non-Goals:**

- No change to what the client asks for or how often.
- No change to the thinning, the caps or the figures.
- No look on Windows by hand. No release.

## Decisions

### D1. Bytes held, counted by the store

`MetricsStore` gains a `#[cfg(test)]` method that sums the heap the store owns: the capacity of every
`Vec` of totals and bases times 8, the bytes of each subject's strings and label keys, and the capacity
of the queues and maps times the size of their entries. It counts capacity, not length, so slack from
growth counts. It is a lower bound of the real heap (allocator overhead is not seen), so the design
also records one outside measure.

Rejected: a counting `#[global_allocator]`. `GlobalAlloc` needs `unsafe impl`, which the workspace forbids.
Rejected: reading the resident size of the process from a test. It is not portable and counts every
other test running in the same binary.

The test lives in `sample.rs` beside `a_day_of_reads_is_thinned_and_keeps_its_figures`. The Tests rule
says to add an assertion to the test that already builds the fixture; that test has two subjects, too
few for a bound worth stating. So a new test with a new fixture: readings of S subjects with the 14
bounds of the recorded app, built once as `Reading` values with growing counters, pushed every 15 s by
the fake clock for 26 hours (6,240 reads). It asserts:

- after every push, bytes held `<= 520 × S × 17 × 8 + identity`, where identity is the bytes of the
  subject strings counted twice (column and index), plus a fixed slack the test names for the maps;
- from read 5,760 on, bytes held do not pass the value at read 5,760 plus one read's row
  (`S × 17 × 8`), so a full window is flat.

S is the largest size that runs in under 2 s in a debug build on the M3 Max (aim: 660, the 60 module case;
fewer if needed, with the reason in "Measured"). The test does not parse text per read: the parse cost
is the requirement of M20 and has its own test.

The outside measure: `/usr/bin/time -l` (macOS) on the release test binary running that test alone,
against a test that pushes nothing, as M20 did. Both numbers go under "Measured".

### D2. The figure cost, measured once, with a rule

A `#[ignore]` test in `sample.rs` (or a throwaway example not committed, as milestone 22 did) builds
a full 24 hour window of the 60 module case and times `Figures::of` for: all modules, range 24 hours,
with the total selected; one module with its 10 routes, range 24 hours, one route selected; and both at
1 hour. Release build, median of 21 runs after one warm-up. The numbers go under "Measured".

The rule: if the slowest case is under 8 ms (half a frame at 60 Hz) on the M3 Max, the work stays on
the interface thread and the design says so. Else the figures are worked out on the worker after each
read and handed over with the state, and the interface thread works them out only for a change of
scope, range or row. A slower computer runs perhaps 3 to 4 times slower; 8 ms leaves room for that at
one frame per 15 s.

### D3. The real-actuator job

A script, `tools/real-actuator-app.sh <dir>`, builds the app; the job and a person on a Mac run the same
script. It follows `fixtures/actuator/README.md` with versions as ranges, so a new release is caught:
`laravel/laravel` `^13.0`, `nwidart/laravel-modules:^13.4`, `nwidart/laravel-actuator:^1.3`,
`nwidart/laravel-metrics:^1.1`. It writes two modules, `Blog` and `Billing`, each with one indicator,
one web route with a parameter and one job; an app route; the exposure of `health`, `info`,
`mappings`, `scheduledtasks` and `prometheus`; and `.env` with `METRICS_ENABLED=true`,
`METRICS_STORE=cache`, `METRICS_CACHE_STORE=file`. The PHP files it writes sit in
`tools/real-actuator/` as plain files, copied in. The script prints `composer show --direct` for the
log.

The job "Real actuator (Linux, not required)" sits beside "Real PHP": `continue-on-error: true`,
`timeout-minutes: 30`, the same pinned `setup-php`. It runs the script, starts three servers with
`php artisan serve` on 127.0.0.1:

| Port | Environment | Answer of `prometheus` |
|---|---|---|
| 8931 | as built | 200 with metrics |
| 8932 | `METRICS_ENABLED=false` | 200, empty |
| 8933 | `METRICS_CACHE_STORE` set to a store whose database file does not exist | 503 |

A process variable wins over `.env` in Laravel, so one app serves all three. The builder checks that
`php artisan serve` passes the variable to the worker; if it does not, it uses `php -S` with
`server.php`. The job waits for each `health` with a bounded `curl --retry`, makes the traffic with
`curl` (Blog route several times, one app route, one 500 if cheap), dispatches the jobs and runs
`php artisan queue:work --stop-when-empty`, then runs
`cargo test -p mortise-core --test real_actuator --locked -- --ignored --test-threads=1` with
`MORTISE_REAL_ACTUATOR=http://127.0.0.1:8931/actuator`, `MORTISE_REAL_ACTUATOR_EMPTY=...8932...`,
`MORTISE_REAL_ACTUATOR_503=...8933...` and `MORTISE_REAL_APP=<dir>`.

`tests/real_actuator.rs` reads through the client Mortise uses (the same calls as the worker, not raw
HTTP), reads the app with `read_app` for the modules, and checks: the base path lists `prometheus`;
`Blog` and `Billing` read Up from their indicators, which leave the app's components; the figures place
the Blog route in `Blog`, the Billing job in `Billing` and the app route in "Not in a module"; the empty
answer reads as metrics off; the 503 reads as a 503 with the app's line. Each test returns early with a
line on stderr when its variable is unset, as `real_php.rs` does, and is `#[ignore]`.

A failing package release then fails this job only. `fixtures/actuator/` stays as recorded; the job is
the warning that it may be stale.

### D4. Images

Each missing id gets an image in `tests/images.rs`, both themes, built from the fixtures the screen tests
already use: S2.1, S4.1, S5.2, S5.6, S6.5 b to i, S8.1, S8.5, and the scheduled tasks box with its Module
column (S4.2). An id the app cannot reach without a state the tests cannot make gets a line in
"Images not stored" below.

S7.3 day: the image used a 1 ms metrics pace so 5,760 reads took seconds, and the tab read that pace.
The fix stays in the test: it keeps the 15 s pace of `Config` and asks each read with Ctrl R, while the
fake clock steps 15 s per read. The view does not learn about tests. The test that makes the S7.3 images
now takes about 55 s on the M3 Max.

S7.3 sparklines: a "Last hour" line has up to 241 points in a cell about 58 px wide, so a jagged line
fills its box. The drawing keeps at most one point per pixel column: points that fall in one column are
averaged. The chart already has more pixels than points. The test data of the images covered 10 minutes
with a sharp zigzag; it now fills the whole hour with a smooth rise and fall per module, so the chart
and each sparkline read as lines.

### D5. A switch that wraps

`draw::segmented` gains a form that takes a width. When the track is wider, the segments wrap onto more
lines of the same height, 4 px apart, each line its own track; the order of segments, the Tab stop and
the arrow keys stay those of one control. A single segment wider than the width is cut with an ellipsis;
its full name stays its accessible name and shows on hover. `segmented_width` gets the matching height.

S7.3 lays out the environment switch at the width of the tab and the range switch after it or under it,
as now. The runtime box keeps one line when the switch fits right of the title; else the switch goes
under the title, at the width of the box. One image: S7.3 and a runtime box with eight environments of
30 characters, both themes.

Rejected: a horizontal scroll. Names past the edge are hidden, and a scroll bar inside a heading is
hard to reach. Rejected: a drop-down past N environments. The spec draws a segmented control.

### D6. The spec-build scripts

Each script finds the repository root from its own path (`git rev-parse --show-toplevel` from the script's
folder, or the folder two levels up). `build.mjs` writes to a folder inside the repository (the folder
`lib.py` uses for its pages, or `docs/spec-build/out/`, ignored by git). `shoot.sh` and `offs.sh` read
pages from that folder and write only to a path that resolves inside the repository; an output path
outside it stops the script with "Write inside the repository: <path> is outside <root>." and exit
status 1. Temporary files go in the same folder, not beside the script.

### D7. The text sweep

A grep over `crates/mortise-app/src/**/view.rs`, `app/*.rs` and the other drawing files for string
literals passed to drawing calls, labels, tooltips and accessible names. Each one moves to the
`strings.rs` of its feature, or `src/strings.rs` when two features show it. Format strings with
arguments become functions in `strings.rs`, as the existing ones are. Ids, salts, keys and log text stay.

### D8. The look on Linux

On the owner's Linux box, with the screen unlocked by the owner, after the orchestrator says go: the
branch is copied there, built, a fixture app opened, and shots taken of the module index with runtime
marks, a module Summary with its runtime box, S7.1 and, if the recorded answers can be served, S7.3.
Each difference from macOS becomes a deviation row checked "run on Linux". Windows: CI builds and tests
it; nothing is looked at by hand, and the rows say so.

## Measured

Measured on 2026-11-24 on an Apple M3 Max with 128 GB of memory (137438953472 bytes), macOS 27.2.1,
Rust 1.99.0. Nothing ran on Windows or Linux.

**The size of the test (D1).** `a_full_window_of_a_large_app_holds_its_bound_and_stays_flat` reads 8
modules of 10 routes and 8 jobs: S = 88 subjects. It runs in 1.88 s to 1.90 s in a debug build (three
runs). With the 60 modules of `large_answer` (S = 660) it runs in 13.6 s: `push` takes about 2 ms a read
in a debug build, 12.2 s of the 13.6 s. 9 modules (S = 99) take 2.08 s. So S is 88, not 660. The store
is the same code at both sizes; the bound grows with S.

**Bytes held (D1), by the count of the store.** The bound of the test is `520 × row + identity + maps`.
A row is `S × 17 × 8` bytes. `maps` is, per subject, its column, its entry in the index, its map of
bases with room for 3 and its base, and the two queues with room for 768 reads.

| S | Bytes held from read 5,760 to read 6,240 | Bound | Bound less one row |
|---|---|---|---|
| 88 | 6,336,648 | 6,347,952 | 6,335,984: fails at read 5,742 |
| 660 (run once, not committed) | 47,184,882 | 47,251,866 | 47,162,106: fails at read 5,742 |

Bytes held do not change from read 5,742 to read 6,240: the window is full and flat. The store keeps
517 reads at that pace (276 coarse, 241 fine). The 660 case holds 47.2 MB, as M20 D1 said (47 MB).

**Resident size (D1), outside.** `/usr/bin/time -l` on the release test binary, one test alone, three
runs each:

| Test | Maximum resident set size |
|---|---|
| `a_full_window_of_a_large_app_holds_its_bound_and_stays_flat`, S = 88 | 15.4 MB (15384576 to 15482880 bytes) |
| The same test with S = 660 (run once, not committed) | 66.5 MB to 70.3 MB (66519040 to 70270976 bytes) |
| `the_real_clock_adds_the_time_asleep_and_never_goes_back` (no read) | 7.06 MB (7061504 to 7077888 bytes) |

With S = 88, the run adds 8.3 MB to 8.4 MB; the store counts 6.34 MB. With S = 660, it adds 59.5 MB to
63.2 MB; the store counts 47.2 MB. The rest is the text of the answer and its `Reading` (built once),
the clone of that reading for each read, and what the allocator keeps around blocks of 90 KB and more.
So the count of the store is about 75% to 80% of what the process grows by.

**The figure cost (D2).** A throwaway `#[ignore]` test in `sample.rs`, not committed, pushed 26 hours of
reads of `large_answer(60)` (660 subjects, 517 reads kept) and placed them in an app of 60 modules
(`Module0` to `Module59`) through 600 routes of `mappings` and the job classes. Every series was placed.
Release build, one warm-up, then the median of 21 calls of `Figures::of`, in three runs of the test:

| Case | Median of each run | Slowest call |
|---|---|---|
| All modules, range 24 hours, the total selected | 2.15 ms, 2.31 ms, 2.15 ms | 3.06 ms |
| `Module7` and its 10 routes, range 24 hours, one route selected | 1.76 ms, 1.84 ms, 1.75 ms | 2.06 ms |
| All modules, range 1 hour, the total selected | 1.13 ms, 0.96 ms, 0.96 ms | 1.82 ms |
| `Module7`, range 1 hour, one route selected | 0.92 ms, 0.84 ms, 0.86 ms | 1.25 ms |

`Placing::new` with the 600 routes adds 0.18 ms to 0.27 ms. The slowest median is 2.31 ms, under the
8 ms of the rule. **The figures stay on the interface thread.** No code moved.

## Images not stored

- S6.5 e: the sentence quotes the address the stub listens on, and its port changes with every run,
  so the image would change with it. Cutting the sentence out would leave no image of the state.

## Risks / Trade-offs

- [Bytes counted by capacity miss allocator overhead] → The outside measure of D1 gives the real
  number once; the test guards growth and the order of size.
- [A package release breaks the job on a day nothing in Mortise changed] → The job is not required. Its
  failure names the step and the package versions in the log.
- [`php artisan serve` may not pass a process variable to its worker] → D3 falls back to `php -S`.
- [The job serves with one PHP worker; requests and the read of `prometheus` share it] → The tests read
  after traffic ends. Each request has the client's timeout.
- [Averaging per pixel hides a one-read spike in a sparkline] → The chart shows every point; the
  sparkline is a summary.
- [A wrapped switch makes the heading of the runtime box taller] → Only with names that do not fit; the
  image shows it.
- [In S7.3, choosing an environment shows the waiting state, which has no switch] → Keyboard focus
  leaves the switch after one arrow key, and Tab must reach it again. Every environment stays
  reachable. Found in group 3; it was there before this change. The runtime box keeps its switch while
  it waits, so its test checks the arrow keys in full. Left for a later change.
- [The Linux box can be locked] → No window opens on a locked screen. The look waits for the owner.
