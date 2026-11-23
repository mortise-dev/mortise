# Proposal

## Why

Phase 3 reads the running app per module. `nwidart/laravel-modules` 13.4.0 (P9) names the health
indicators of a module `module:{lower case module name}:{key}` and adds a `modules` component that
lists each module and reports one that is enabled but missing on disk. Mortise today lists all of these
as app components in the runtime box of S4.2, and shows the `modules` component as raw JSON. The spec
draws the runtime status of each module on its index row (S3.1, sixth drawing) and in a runtime box on
its Summary (S4.3). Milestone 19 of `docs/plan/phase-3.md` builds both.

## What Changes

- The core works out the runtime status of each module from the health components of the chosen
  environment (decision 4 of the plan): the worst of the module's own indicators in the order Down,
  Out of service, Up, Unknown. A module the `modules` component reports in error reads Down with the
  app's own sentence. A module with no indicator of its own and no error has no status.
- What decides it is what the environment answers (decision 7), not the version in `composer.lock`.
- The index draws a runtime stroke mark on each module row that has a status: on two-line rows,
  on one-line rows (above 20 modules) and on the strip. The finding mark stays. A Down module raises
  no pip on the rail (Q11).
- The Summary of a module that has a status draws a runtime box: the status of the module, then its
  own indicators. Without one, no box is drawn.
- The runtime box of the app (S4.2) stops listing module indicators and the `modules` component. A
  module that is not Up is named there instead, so a Down app still shows why. An indicator or an error
  for a module the app's files do not have stays in the app box.
- `StubActuator` grows the module components of the recorded `health.json`.

## Capabilities

### New Capabilities

- `module-runtime`: the runtime status of each module, read from the health components of an
  environment, and where the interface shows it.

### Modified Capabilities

None. `actuator-client` is unchanged: no new path is asked for.

## Impact

- `mortise-core`: new `runtime/modules.rs` behind the feature `actuator`, re-exported from `lib.rs`.
  `testing.rs` grows the recorded module components. `runtime/actuator.rs` and `runtime/live.rs` are
  not changed.
- `mortise-app`: `index/view.rs` (the mark), `summary/view.rs` (the module box), `runtime/view.rs`
  (the app box and the module box items), the wiring that passes the runtime to the index, and the
  `strings.rs` of those features.
- Images of the marks and of the module box, light and dark. Rows in `docs/deviations.md`.
- No new dependency. No release (decision 11).
