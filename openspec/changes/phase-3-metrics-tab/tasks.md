# Tasks

Group 1 runs first. Groups 2 and 3 run in parallel after it, each in its own worktree. `design.md`,
"Interfaces between the tasks", says which files each owns. Group 4 runs last. Each group ends with
`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
`cargo test --workspace`.

## 1. Core (21.2, Opus)

- [x] 1.1 Add `Range` and `Scope` to `metrics/figures.rs` as D1 and D3 say. Remove `LAST_MINUTE`
- [x] 1.2 Add `bounds` to `Change` and `MetricsStore::steps` with `Step` to `metrics/sample.rs`, as D4 says
- [x] 1.3 Add `Under`, the fields of `Rates`, `RowOf`, `Row`, `Point`, `Line` with `Line::now`, and the fields of `Figures`, as D2 to D4 say. Remove `ModuleFigures`. Work out the 95% bound, the error share and the database share as D2 says, per module and per route
- [x] 1.4 Change `LiveMetrics::figures` to take a `Scope` and a `Range`. Add `MetricsState::ago` as D5 says. Re-export the new names in `lib.rs`
- [x] 1.5 Add `StubActuator::serve` to `testing.rs`
- [x] 1.6 Change the app lines the renames break, with no new drawing: the plain table reads `Figures::rows` and `Figures::outside` over `Range::Hour`
- [x] 1.7 Test in the core, once each: on the recorded quiet and busy answers 15 s apart, the 95% figure, the error share and "unknown" database time of each module, and the routes of one module (add the assertions to the placing test of M20 that already reads those answers); with written readings, `Above` past the largest bound, a database share with the label, and the line of a day of reads (a point every 5 minutes before the last hour and every 15 s in it, a break at a gap, and `Line::now`). Verify with `cargo test -p mortise-core metrics::`

## 2. The chart (21.3, Opus)

- [x] 2.1 Write `metrics/chart.rs` with `Plot`, `CHART_H`, `chart` and `spark` as D7 says: the painter only, straight segments in 1.5 px ink, four hairlines, the step of the scale, the accessible name and description. No interface text in the file
- [x] 2.2 Test the step of the scale once in `chart.rs`

## 3. The tab (21.4, Opus)

- [x] 3.1 Add `MetricsTab` to `OpenApp`, and pass the scope and the range to `LiveMetrics::figures`. Work the figures out again after a read, or for another snapshot, answer, scope or range
- [x] 3.2 Draw S7.3 in `metrics/view.rs` as D6 says: the two switches, the line of the last read, the chart box with `chart`, the table of seven columns with `spark`, the routes of one module, the selected row, "unknown" with its tooltip. Ask for a frame after one second while the table is shown
- [x] 3.3 Add the text of D6 and D7 to `metrics/strings.rs`, each line not in the spec marked so, with one unit test of the number formats beside the one of M20
- [x] 3.4 Test the wiring in `tests/metrics.rs`, in the test that already makes two reads with a stepping clock: the seven column headers; Database time reads "unknown" with its tooltip; "last read N s ago" counts up when the clock moves with no read; the range switch keeps the figures; a row clicked, then J, moves the chart title to the next row; selecting a module in the index lists its routes. In another test, the environment switch starts the table again with no read

## 4. Images and deviations (21.5, Sonnet)

- [x] 4.1 Replace `s7.3-metrics-table` in `tests/images.rs` with three images in both themes, with a fixed clock: all modules, one module, a 24 hour range. Use `StubActuator::serve` and a clock that steps 15 s per request for the 24 hour range. Run `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images` and look at each new and changed image
- [x] 4.2 Check that every new control has an accessible name and a target of 24 by 24 px (`tests/accessibility.rs`)
- [x] 4.3 In `docs/deviations.md`, add S68 (the 95% figure is a bucket bound) and S69 (the time labels of the chart, "Route" as the first column of one module, an en dash for jobs and database time of a route, the tooltip of "unknown"), and close S67. Say how each fact was checked
