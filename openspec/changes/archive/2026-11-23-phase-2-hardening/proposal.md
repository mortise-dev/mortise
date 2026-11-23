# Proposal

## Why

Milestones 10 to 14 of `docs/plan/phase-2.md` built how PHP runs, the runner drawer, the actions and
Generate. Milestone 16 hardens them before milestone 15 (package data), which waits for
`laravel-modules` 13.4. Their reviews left layout bugs, a missing glyph, a missing palette group, runs
that outlive Mortise, a flaky settings test, and no check against a real PHP app.

## What Changes

- Images in both themes for every phase 2 screen id, and the accessibility harness on every phase 2
  screen. Settings, Keyboard lists Ctrl \`, Ctrl ., Ctrl Enter and Ctrl Shift C (Cmd on macOS).
- A Linux job in CI, not required, with PHP 8.4 and Composer. It creates a Laravel app with
  `nwidart/laravel-modules` `^13.3` and runs core tests marked `#[ignore]`: `module:make`, one
  generator, `module:migrate` on SQLite and `module:delete`, through the real runner, with the output
  reader checked on the real output.
- Layout fixes: graph nodes as wide as their stamp in a chain of 12 modules; the one-module graph keeps
  the names of its neighbours whole; Cancel stays on the Add panel at 150 % with a long app name;
  "Rules for platform" and "kept in mortise.json" do not overlap at 1100 by 700.
- The drawer draws `⇂` (printed by `--pretend` and `module:lang`) instead of a box.
- "Recent" in the empty palette (S12.2 a) lists the recent actions and generators of the app, from its
  run history.
- Closing Mortise stops the run of each app, as Ctrl . does, and waits at most the time limit of the stop.
- On Windows, a `herd.bat` argument that the standard library refuses to pass to `cmd.exe` shows as
  "The command did not start" with its reason.
- Text: the dry run option reads "Dry run: print the SQL, run none of it". Deviation S40 is accepted by
  the owner.
- The README describes phase 2. Deviations S32 to S50 are checked against the code as built.
- The flaky settings test `the_fields_of_an_app_apply_when_they_lose_focus` is fixed at its cause.

Out of scope: everything that needs P2 to P8 (milestone 15), a release.

## Capabilities

### New Capabilities

- `real-php-check`: the CI job and the ignored core tests that run the package's commands in a real app.

### Modified Capabilities

- `module-actions`: the text of the dry run option; recent actions in the empty palette.
- `command-runs`: closing Mortise stops every run.
- `runner-drawer`: a command the platform refuses to start.
- `boundary-graph`: node width and neighbour names.
- `add-panel`: Cancel stays reachable with a long app name.

## Impact

- `mortise-core`: `run/process.rs` (stop and wait), `run/history.rs` reads, `catalog/actions.rs`
  comment, new ignored tests.
- `mortise-app`: `app.rs` (close, palette source), `palette.rs`, `boundary.rs`, `add.rs`,
  `settings_view.rs`, `runner_view.rs`, `theme.rs` or fonts, `strings.rs`, tests and images.
- `.github/workflows/ci.yml`: one job, not required.
- Docs: `README.md`, `docs/deviations.md`, `docs/spec-build/p_write.py` and the rebuilt `docs/spec`.
- Possibly `licenses/` for a fallback font.
- Not run here: Windows. Linux only through CI unless the thinkcentre is reachable.
