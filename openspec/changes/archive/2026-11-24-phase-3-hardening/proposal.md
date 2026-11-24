# Proposal

## Why

Milestones 18 to 22 built the runtime per module and the Metrics tab, and each left checks for the end of
phase 3: the memory of a 24 hour window was bounded on paper only, the client never read a live
`prometheus`, several built screens have no stored image, and a few layout and text gaps were deferred.
Milestone 23 closes them so phase 3 can end.

## What Changes

- A 24 hour run of the read store with the fake clock asserts a memory bound once the window is full.
  The cost of working out figures on the interface thread is measured and written in the design. Work
  moves off the frame only if the measure shows a visible cost.
- A new CI job, not required, beside "Real PHP": it builds a Laravel app with laravel-modules,
  laravel-actuator and laravel-metrics, serves it, makes traffic and reads `health` and `prometheus`
  through Mortise's client in `#[ignore]` core tests. It also reads an empty answer and a 503.
- Stored images for the built screen ids that have none: S2.1, S4.1, S5.2, S5.6, S6.5 b to i, S8.1,
  S8.5 and the scheduled tasks box with its Module column (S4.2). An id that cannot get one is listed in
  the design with the reason.
- The S7.3 images are fixed: the day image reads the real 15 s pace, and the "Last hour" sparklines
  read as lines.
- An accessibility sweep over every control phase 3 added.
- An interface text sweep: literal text left in a `view.rs` or an `app/*.rs` moves to a `strings.rs`.
- The environment switch of S7.3 and of the runtime box stays inside its space with many long
  environment names.
- `docs/spec-build/build.mjs`, `shoot.sh` and `offs.sh` write inside the repository or stop with a
  clear message. They never write outside it.
- Linux looked at on the owner's Linux box; Windows through CI only. Differences become deviation rows.
- `README.md`, `docs/deviations.md` and `docs/plan/phase-3.md` updated. `cargo doc` passes with
  warnings denied.

## Capabilities

### New Capabilities

- `real-actuator-check`: a CI job, not required, and ignored core tests that read a live actuator of a
  Laravel app with the three packages through Mortise's client.

### Modified Capabilities

- `metrics`: the 24 hour requirement states a memory bound that a test asserts; the environment and
  range switches of the tab fit the width of the tab.
- `module-runtime`: the environment switch of a runtime box stays inside the box.
- `real-php-check`: "Real PHP" is no longer the only job that uses the network for Composer.

## Impact

- `crates/mortise-core`: `metrics/sample.rs` (test and, if needed, a measure of held bytes), a new
  ignored test file `tests/real_actuator.rs`.
- `crates/mortise-app`: `metrics/view.rs`, `runtime/view.rs`, `ui/draw.rs` (the switch), `strings.rs`
  files, `tests/images.rs`, `tests/accessibility.rs`, stored images.
- `.github/workflows/ci.yml`: one new job, not required. A script in `tools/` builds the app.
- `docs/spec-build`: three scripts.
- Docs: `README.md`, `docs/deviations.md`, `docs/plan/phase-3.md`.
- No new crate and no new dependency. No change to what the client asks for.
