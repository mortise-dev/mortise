# Proposal

## Why

Phase 3 shows requests, jobs and errors per module, read from the `prometheus` endpoint of the running
app. Milestone 20 of `docs/plan/phase-3.md` builds the reading half: the text format, the reads kept in
memory, the rates, the placing of a series in a module, and the request itself. Milestone 21 draws S7.3
on top of it and must not need a second store.

## What Changes

- A new core folder `metrics/`, behind the feature `actuator`: the Prometheus text format (counter,
  gauge, histogram), the reads kept in memory for 24 hours, the reset and gap rules, the placing of a
  series in a module (decision 2 of the plan), and requests per minute and jobs per minute per module.
- The actuator client reads `prometheus` as text, with a body limit of 16 MiB that counts the text
  after decompression. A body over the limit is an answer that says so.
- The worker of the live runtime asks for `prometheus` every 15 s, only while the Metrics tab is open
  and the window is visible, and only when the app exposes it. `health` keeps its own 15 s.
- The Metrics tab stops showing the S7.4 block. It shows S7.1 (not connected), S7.2 (not exposed), a
  sentence for a 503, for an empty answer and for an answer that is too large, and a plain table:
  Module, Requests/min, Jobs/min, and a row "Not in a module".
- Nothing from metrics is written: not to disk, not to `mortise.json`, not to `settings.json`.
  Switching the environment empties the reads.
- `docs/deviations.md` records decision 2 and the states of the tab the spec does not draw.

Not in this change (milestone 21): the chart, the ranges, the 95% bound, the error share, database time,
figures per route, J and K.

## Capabilities

### New Capabilities

- `metrics`: how Mortise reads the metrics of a running app, what it keeps, how it turns counters into
  rates, how it places a series in a module, and what the Metrics tab shows for each answer.

### Modified Capabilities

- `actuator-client`: the text read of `prometheus`, its body limit, its pace, and that it is asked
  only when exposed. The `Accept` header stays `application/json` on every request, on purpose.
- `code-layout`: `metrics` joins the order of core modules, after `runtime`.

## Impact

- `mortise-core`: new `src/metrics/` (`mod.rs`, `prometheus.rs`, `sample.rs`, `place.rs`, `figures.rs`).
  `runtime/actuator.rs` gains `ask_metrics` beside `fetch()`. `runtime/live.rs` gains one branch in the
  worker loop, `metrics_seen()`, `metrics_closed()` and `open_with_metrics()`. `Pace` gains the field
  `metrics`.
  `lib.rs` re-exports the new public names. `testing.rs` serves the recorded answers.
- `mortise-app`: new `src/metrics/` (`mod.rs`, `view.rs`, `strings.rs`) and `src/app/metrics.rs`.
  `app/workspace.rs` drops the S7.4 block. `app/runtime.rs` opens the runtime with a metrics sink.
  `Config` gains `pace` and `metrics_clock`, so a test gets two reads without waiting 15 s.
  Sentences that the runtime box and the Metrics tab both show move to `src/strings.rs`.
- Milestone 19 edits `runtime/actuator.rs` and `runtime/live.rs` in parallel. This change keeps its
  edits there to the new read and the new branch.
- Images: `s7.4-metrics` goes; S7.1, S7.2 and the table come, in both themes.
- No new crate and no new dependency. No release (decision 11).
- Run on macOS. CI runs Linux and Windows. Nothing here depends on the platform, except the clock
  while the computer sleeps (see `design.md`, D3).
