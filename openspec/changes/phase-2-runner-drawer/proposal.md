# Proposal

## Why

Milestone 10 chose how PHP runs for each app, but Mortise can still run only `artisan --version`, and
only to its end. The generators and actions of milestones 12 and 14 need a place where a command runs,
streams its output, can be stopped and is kept in a history. Milestone 11 of `docs/plan/phase-2.md`
builds that place: the runner drawer of S11.1. It also fixes two findings of the review of milestone 10:
the time limit of the test ends only the launcher, and a child that keeps the pipes open keeps the
reader threads alive.

## What Changes

- `run/process.rs` runs a command to its end on three threads: two read standard output and standard
  error, one owns the program, its time limit and its stop. It sends each line as it comes.
- Stop and the time limit end the whole process tree: SIGTERM to the process group, then SIGKILL after
  3 s, on macOS and Linux (through `rustix`); `taskkill /T /F` on Windows. The test of milestone 10 uses
  the same stop. A run that has ended leaves no reader thread behind.
- One run at a time per app. A second start is refused with the name of the running command.
- `run/ansi.rs` maps SGR codes to the four styles of the runner: text, dim, success, error.
- `run/history.rs` keeps the last 50 runs per app in Mortise's data folder, with their output up to
  1 MB each.
- The fake artisan of the tests prints recorded output of the package, can wait, ask for input and
  ignore SIGTERM. The recordings come from real runs of `nwidart/laravel-modules` 13.3.0 and are kept in
  `fixtures/output/`.
- The runner drawer (S11.1 a to g): running, finished, failed, history, collapsed and empty. Ctrl `
  shows and hides it, Ctrl . stops (asking first for a migration, rollback or seed), the handle sets
  its height, Up, Down and Enter move through the history. "The command did not start" offers
  "Open settings".

## Capabilities

### New Capabilities

- `command-runs`: how a command runs in an app: output as it comes, stop, time limit, one run per app,
  ANSI styles, and the history of runs.
- `runner-drawer`: the drawer of S11.1 that shows a run, its history and its keys.

### Modified Capabilities

- `php-runner`: the test ends the whole process tree at its limit, not only the launcher.

## Impact

- `mortise-core`: `run/process.rs`, new `run/ansi.rs` and `run/history.rs`, `run/mod.rs`, `lib.rs`,
  `testing.rs` (the fake artisan), `Cargo.toml` (`rustix` for `cfg(unix)` under the feature `run`).
- `mortise-app`: new `runner_view.rs`, `app.rs`, `workspace.rs` layout, `strings.rs`, `Cargo.toml`
  (`jiff` for the local clock time of the history), tests and images.
- `fixtures/output/`: recorded output of the package commands.
- Docs: `docs/deviations.md`, `licenses/THIRD-PARTY.md`.
- Not run here: Windows and Linux. CI runs the build and tests there. Herd is not installed on this Mac.
