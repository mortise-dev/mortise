# Tasks

Groups 1, 2 and 3 run in parallel, each in its own worktree. `design.md`, "Interfaces between the
tasks", says which files each owns. Group 4 starts when 1, 2 and 3 are merged. Then 5, then 6, then 7.
Each group ends with `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
and `cargo test --workspace`.

## 1. The text format (20.2, Opus)

- [x] 1.1 Create `metrics/mod.rs` with the doc comment and the `prometheus` line of `design.md`, and add `#[cfg(feature = "actuator")] mod metrics;` to `lib.rs` after `mod generate;`. Verify with `cargo check -p mortise-core --lib --no-default-features --features actuator` and the same with no feature, both under `RUSTFLAGS="-D warnings"`
- [x] 1.2 Write `metrics/prometheus.rs`: `Kind`, `Part`, `Sample`, `Sample::label` and `samples`, with the rules of `design.md`. No dependency is added. Verify with `git diff --stat Cargo.toml Cargo.lock` (empty)
- [x] 1.3 Test on `fixtures/actuator/prometheus-busy.txt`, in one test: the job label reads `Modules\Blog\Jobs\PublishPost`; a `le="+Inf"` bucket is `Part::Bucket(f64::INFINITY)`; `_sum` and `_count` of the `/blog` series have the recorded values; a counter and a gauge are read; the number of samples of the request histogram is the number of its lines. In the same test, `prometheus-quiet.txt` gives gauges and counters only and `prometheus-empty.txt` gives nothing. Verify with `cargo test -p mortise-core metrics::prometheus`
- [x] 1.4 Test on a short written answer: a `summary` family between two histograms is skipped; a sample with no `# TYPE` line is skipped; a line with no value and a line with an open quote are skipped and the lines around them are read; `\"` and `\n` in a label value; a timestamp after the value; `\r\n` line ends. Verify with the same command

## 2. The reads (20.3, Opus)

- [x] 2.1 Create `metrics/mod.rs` with the doc comment and the `sample` line of `design.md`, and the `mod metrics;` line in `lib.rs`, as in 1.1. Verify as 1.1
- [x] 2.2 Write `metrics/sample.rs`: `Subject`, `Cell`, `Reading`, `Change` and `MetricsStore` with `new`, `push`, `len` and `change`, as D1 and D2 say. Tests build `Reading`s by hand
- [x] 2.3 Test the rules of D2, one test per rule: the first read gives no change; 100, 3, 5 adds 0 then 2 and no figure is negative; a subject that appears in a run counts from zero, with its errors; a missing subject keeps its totals and its base; a read 46 s after the last starts a new run and one 30 s after does not; a rate divides by the covered time; buckets match by bound in any order, a missing bucket keeps its base, a new bound widens the rows, and the first bounds widen them too. Verify with `cargo test -p mortise-core metrics::sample`
- [x] 2.4 Test the window with times as parameters: 7,000 pushes 15 s apart keep at most 520 reads and the number does not grow after push 5,760; the change over 24 hours equals the sum of what was pushed in that span; a subject that stopped changing 24 hours ago is dropped. The test takes under a second. Verify with the same command

## 3. The client (20.5, Opus)

- [x] 3.1 In `runtime/actuator.rs`, add `METRICS_BODY_LIMIT`, `NoMetrics` and `ask_metrics` beside `fetch()`, as D7 says. `fetch()` behaves as before. Update the doc comment of the file: the client now asks for `prometheus`. Re-export the three names in `lib.rs`
- [x] 3.2 In `runtime/live.rs`, add `Pace::metrics`, `LiveRuntime::open_with_metrics`, `LiveRuntime::metrics_seen` and the metrics branch of the worker loop, as D4 says. `open` and `open_with` keep their signatures. No line names `crate::metrics`. Verify with `grep -rn "crate::metrics" crates/mortise-core/src/runtime` (nothing)
- [x] 3.3 Add `StubActuator::recorded()` to `testing.rs`: it serves `fixtures/actuator/` and lists `env` among the links. No second stub
- [x] 3.4 Test `ask_metrics` in `tests/actuator.rs` against the stub, in one test: the busy answer is `Ok` with its text and the request carried `Accept: application/json`; the empty answer is `Off`; the 503 is `Unavailable` with the recorded line; 401 is `NotAuthorized`; a redirect is `Unexpected` with its address; a body of exactly the limit is `Ok` and one byte more is `TooLarge` with the limit. Verify with `cargo test -p mortise-core --test actuator`
- [x] 3.5 Test the pace in `tests/actuator.rs` with a short `Pace` and `StubActuator::recorded()`: with `seen()` only, `health` is asked again and again and `prometheus` is never asked; with `metrics_seen()` too, `prometheus` is asked again and again; when `metrics_seen()` stops, the count of `prometheus` stops and `health` goes on; when `seen()` stops, both stop; with `prometheus` taken out of the links, it is not asked; `env` is never asked in the whole test. Add the `metrics` field to the `Pace` of the three tests that exist. Verify with the same command

## 4. Placing and figures (20.4, Opus)

- [x] 4.1 Add `Reading::of` to `metrics/sample.rs` with the table of `design.md`, and remove both `allow(dead_code)` from `metrics/mod.rs`. Verify with `grep -rn "dead_code" crates/mortise-core/src/metrics` (nothing)
- [x] 4.2 Write `metrics/place.rs`: `Placing::new` and `Placing::place` with the rules of D6
- [x] 4.3 Write `metrics/figures.rs`: `Rates`, `ModuleFigures`, `Figures`, `LAST_MINUTE` and the function that turns a `Change` and a `Placing` into `Figures`, as D9 says
- [x] 4.4 Add `MetricsClock`, `MetricsState` and `LiveMetrics` to `metrics/sample.rs`, with the real clock of D3. The text is folded before the lock is taken. Re-export the public names in `lib.rs`. Test once, with written durations, that the real clock adds the time the wall clock ran ahead and never goes back
- [x] 4.5 Test placing once, on the recorded answers: read `fixtures/shop-api`, add a module Blog with the namespace `Modules\Blog` to the snapshot, take the routes of `mappings.json` and push `prometheus-quiet.txt` then `prometheus-busy.txt` 15 s apart. Billing and Blog get the requests of their routes and their job; `/portal` counts for Billing; `/`, `/hello/{name}`, `/status`, `/up` and the 404 and 405 series count as "Not in a module"; the sum over all rows is the sum of the recorded counts, as rates per minute. In the same test, with written input: a second route `portal` of Blog makes `/portal` "Not in a module"; `module="billing"` wins over a Blog route; `module="none"` and an unknown name are "Not in a module"; with no routes, requests are "Not in a module" and jobs are still placed. Verify with `cargo test -p mortise-core metrics::`
- [x] 4.6 Test the read of a large answer: generate 60 modules of 10 routes and 60 jobs in the shape of the recorded lines (about 3.3 MB), fold it with `Reading::of`, and assert 660 cells and under 1 s. Verify with the same command

## 5. The app (20.6, Opus)

- [x] 5.1 Add `pace` and `metrics_clock` to `Config`, `metrics: LiveMetrics` to `Connected`, and open the runtime with `open_with_metrics` in `app/runtime.rs`
- [x] 5.2 Move `TRY_AGAIN`, `cannot_reach`, `refused_credentials`, `answered_unexpected` and `NOT_IN_A_MODULE` to `mortise-app/src/strings.rs`. No name is defined twice and no image changes. Verify with `cargo test -p mortise-app --test images`
- [x] 5.3 Write `metrics/strings.rs` with the text of `design.md`, each line that is not in the spec marked so, and the number format of D9
- [x] 5.4 Write `metrics/view.rs` and `app/metrics.rs`: the states of D7, the table of D9, `metrics_seen()` or `metrics_closed()` from the state at the end of each frame (D4), figures worked out again only when the serial or an input changed. Every control has an accessible name and a target of 24 by 24 px or more
- [x] 5.5 In `app/workspace.rs`, draw the Metrics tab from `metrics/view.rs`. Remove `Tab::explanation` and the two `METRICS_UNAVAILABLE` strings. Update `tests/workbench.rs` and `tests/screens.rs`, which name the old heading and image
- [x] 5.6 Test the wiring in a new `tests/metrics.rs`, once per state, against `StubActuator::recorded()`: with no environment the tab reads the S7.1 title and paragraph, and "Connect an environment" opens Settings on the environments of the app; without `prometheus` in the links the tab reads the S7.2 title and paragraph, "Open settings" opens Settings, and "Try again" after the link is added shows the table; the 503 and the empty answer show their sentences, the 503 with the recorded line; with a stepping clock the table lists every module and "Not in a module" with figures after two reads. In the table test, compare the files and their content in the config folder and the data folder before and after: they are the same. Verify with `cargo test -p mortise-app --test metrics`

## 6. Images and measurement (20.7, Sonnet)

- [x] 6.1 Add images to `tests/images.rs` for S7.1, S7.2 and the table, light and dark, with the stepping clock. Remove `s7.4-metrics`. Run `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images` and look at each new and each changed image
- [x] 6.2 Add the controls of the Metrics tab to the sweep of `tests/accessibility.rs`, in the test that already walks the tabs. Verify with `cargo test -p mortise-app --test accessibility`
- [x] 6.3 Measure the generated answer of 4.6 in a release build: the time of `Reading::of`, and the peak size of the process with and without the read (`/usr/bin/time -l` on macOS). Write the numbers and the computer under "Measured" in `design.md`. They are within the budget of D8, or the lead is told

## 7. Integration

- [x] 7.1 Add rows to `docs/deviations.md`: decision 2 (Mortise places a series itself until the package adds the `module` label), the states of the Metrics tab that the spec does not draw, and S6 (the Metrics tab no longer uses the S7.4 block). Say what was checked and that nothing ran on Windows or Linux
- [x] 7.2 Check the gate as the CI job `core-features` does: `RUSTFLAGS="-D warnings" cargo check -p mortise-core --lib --bins --no-default-features --features actuator --locked`, and the same with no feature
- [x] 7.3 Check the rules by search: `grep -rn "crate::metrics" crates/mortise-core/src/runtime` finds nothing; `grep -rn '"env"\|/env' crates/mortise-core/src/runtime crates/mortise-core/src/metrics` finds no request; no interface text of metrics is outside a `strings.rs`
- [x] 7.4 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass. `python3 tools/third-party-licenses.py` changes nothing
- [x] 7.5 `openspec validate phase-3-reading-metrics --strict` passes
