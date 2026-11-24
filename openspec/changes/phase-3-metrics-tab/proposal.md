# Proposal

## Why

Milestone 20 of `docs/plan/phase-3.md` reads `prometheus` into a store and shows a plain table of
requests and jobs per minute. S7.3 draws more: a chart, three ranges, seven columns, one module with its
routes, and J and K. Milestone 21 draws S7.3 in full on the store of milestone 20, with no second store.

## What Changes

- Core: four figures per row on top of the rates of milestone 20. "95% under" is the smallest bucket
  bound under which 95% of requests fall (decision 5). "Errors" is the share of requests with a 5xx
  status (decision 6). "Database time" is database time against request time, only from series with a
  `module` label (decision 3), else unknown.
- Core: the ranges 15 min, 1 hour and 24 hours. Every figure of the tab is worked out over the chosen
  range. The range switch keeps the reads.
- Core: figures per route of one module, and a line per row: requests per minute between neighbouring
  kept reads, over the range and over the last hour.
- Core: `MetricsState` says how long ago the newest kept read was, by the clock of the reads.
- App: S7.3 in full. The environment and range switches, "read every 15 s, last read 8 s ago" that
  counts up, a chart drawn with the painter in `metrics/chart.rs`, and a table of seven columns: Module,
  Requests/min, 95% under, Errors, Jobs/min, Database time, Last hour. With a module selected in the
  index, the chart shows that module and the table lists its routes. J and K move through the table and
  the chart follows the selected row.
- `docs/deviations.md`: a row for the 95% figure, a row for the chart's time labels, and S67 closed.

Not in this change: limits or colours on a figure (Q16), storing metrics, reading `modules`, the
24 hour memory run and the real-actuator job (milestone 23).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `metrics`: the plain table of milestone 20 becomes the full S7.3: ranges, five figures per row,
  routes of one module, the chart, J and K, and the time since the last read.

## Impact

- `mortise-core`: `metrics/figures.rs` gains `Range`, `Scope`, `Row`, `RowOf`, `Line`, `Point`, `Under`
  and the new fields of `Rates` and `Figures`. `LAST_MINUTE` and `ModuleFigures` go.
  `metrics/sample.rs` gains `MetricsStore::steps`, the bounds in `Change`, and `MetricsState::ago`.
  `LiveMetrics::figures` takes a scope and a range. `testing.rs` gains a reply made per request.
  `lib.rs` re-exports the new names.
- `mortise-app`: new `metrics/chart.rs`. `metrics/view.rs`, `metrics/strings.rs` and `app/metrics.rs`
  draw S7.3. `OpenApp` keeps the range and the selected row of the tab.
- Images: `s7.3-metrics-table` is replaced by S7.3 for all modules, one module and a 24 hour range, in
  both themes.
- No new crate and no new dependency. No release (decision 11).
- Run on macOS. CI runs Linux and Windows. Nothing here depends on the platform.
