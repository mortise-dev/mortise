# Design

## Context

See `proposal.md` for why. Milestones 10 to 14 (`openspec/changes/archive/2026-11-23-phase-2-*`) built
`run/` (`Runs::start`, `Run::stop` as a flag the worker thread reads, SIGTERM then SIGKILL after 3 s,
`taskkill /T` on Windows), `run/history.rs` (`Entry { args, label, started_ms, ended }`, last 50 per
app), `catalog/actions.rs`, `catalog/generators.rs`, the drawer (`runner_view.rs`, `AppRuns`), the
Actions and Generate tabs, and the palette (`palette.rs`, whose empty query lists one recent module).

Facts found before this design:

- The flaky test is `the_fields_of_an_app_apply_when_they_lose_focus` (`tests/settings.rs:194`), Linux,
  CI of PR #15, attempt 1: "Harness::run exceeded max_steps (4). It would have settled after 6 steps",
  with the repaint cause `crates/mortise-app/src/app.rs:2747` (log read with `gh api`).
- `refuse_batch_with_percent` in `run/process.rs` already refuses a batch path that holds `%`. The
  refusal of the standard library itself (arguments it cannot escape for `cmd.exe`) is a spawn error.
- The text "Dry run: print the SQL, change nothing" is in `p_write.py` (four places), `strings.rs`
  (`DRY_RUN_OPTION`), `tests/actions.rs` (two places), `openspec/specs/module-actions/spec.md`, and a
  comment in `catalog/actions.rs`.

## Goals / Non-Goals

**Goals**: the twelve tasks of milestone 16 in `docs/plan/phase-2.md` and the orchestrator's brief.

**Non-Goals**: anything that needs P2 to P8; a release; a job object on Windows.

## Decisions

### D1. Dry run text (16.9)

Change the words everywhere listed above, then rebuild `docs/spec` with the build script of
`docs/spec-build`. The package runs each `down()` inside `$connection->pretend()` since 13.1.0, so no SQL
reaches the database; "run none of it" says only that. S40 gets "Accepted by the owner, 23 November
2026" in its Raised column.

### D2. The missing glyph (16.5)

Prefer mapping over a new font when the drawer can map `⇂` (U+21C2) to a glyph the bundled fonts have,
because a font adds bytes and a licence for one character. A fallback font is added only if no
bundled glyph reads as the same mark. The choice and the reason go in `docs/deviations.md`. The map, if
chosen, lives where the drawer turns output into spans, and is tested there once.

### D3. Batch refusal (16.8)

A spawn error becomes `NotStarted { error }` already. The task checks that the error text of the
standard library reaches "The command did not start" unchanged, and adds one unit test on a path that
runs on all platforms (the `%` refusal or the error mapping, whichever is reachable without Windows).
Deviation row: Rust 1.77.2 escapes batch arguments or refuses (`std::sys::pal::windows::process`,
source read).

### D4. Layout fixes (16.4)

Each fix goes to the layer that owns the size: node width from the stamp width in `boundary.rs`; the
one-module graph leaves a neighbour name out when it does not fit, as the node rule does; the Add panel
wraps or truncates the app name so the buttons keep their room; the Rules header lays out its two texts
so the second wraps below the first when they do not fit. Each gets an assertion or an image in the test
that already renders that screen.

### D5. Recent actions (16.6)

The palette source gets the recent run entries of the current app (`History::entries`, newest first).
An entry maps to an action or a generator by its artisan words (`args[0]` against the action catalog
and the generator catalog, the module from the arguments). Entries that map to nothing are left out.
Duplicates (same action and module) keep the newest. At most three rows. Choosing a row does what the
palette's action and generator rows already do (S40: open the panel or the form). The age text reads
"ran <n> minutes ago" in the style of the existing time strings; text not in the spec is marked.

### D6. Close stops every run (16.7)

eframe calls `App::on_exit` before the window goes. There Mortise calls `stop` on the run of each app
and waits for each run's end, up to the stop's own limit (3 s to SIGKILL, plus a short margin for the
end event). No dialog: closing the window is the user's choice to stop, and a command whose Ctrl .
asks first is not left running behind a closed app. On Windows `taskkill /T` ends the tree from the
launcher. When `herd.bat` already exited, its PHP child has no parent to walk from and goes on: there is
no job object. Record that limit as a deviation.

### D7. The flaky settings test (16.12)

Find what repaints at `app.rs:2747` after focus leaves a field (a repaint request that depends on time
or on a timer). Fix the cause in the app if a repaint is requested without need; else the test steps
until settled for a known reason. Not a larger `max_steps` alone.

### D8. The real-PHP job (16.3)

`crates/mortise-core/tests/real_php.rs`, every test `#[ignore]`, reads the app folder from
`MORTISE_REAL_APP`. It starts commands through `Runs` with a `Custom` runner line `php artisan`
(no shell), waits for the end, and checks `output::generated_files` and the exit. The job in `ci.yml`
uses `shivammathur/setup-php` (PHP 8.4, sqlite), `composer create-project laravel/laravel`, `composer
require nwidart/laravel-modules:^13.3`, an SQLite database, then `cargo test -p mortise-core --test
real_php -- --ignored`. `continue-on-error: true` and not in the required list. Run once on macOS.

### D9. Accessibility and keys (16.2)

`tests/accessibility.rs` walks each phase 2 screen (Settings PHP, drawer states, Actions, the dialogs,
Generate picker, form, result). The Keyboard section lists the four keys with their spec wording.

### D10. Images (16.1)

One light and one dark image per phase 2 screen id, made after every fix. Each new or changed image is
looked at before it is committed.

## Risks / Trade-offs

- [Wait on close freezes the window up to about 3 s] Only when a command ignores SIGTERM; usual
  commands end at once.
- [Real-PHP job breaks on a package release] It is not required; the failure is the signal.
- [Windows not run] Recorded in each report and deviation.
