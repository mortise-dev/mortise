# Proposal

## Why

Phase 2 runs the commands of `nwidart/laravel-modules` in an app. Before any command runs, Mortise must
know how PHP runs for each app: through Herd, Sail, Docker or a command the user types. Milestone 10 of
`docs/plan/phase-2.md` builds that choice, its test and the screens that show it. Milestones 11 to 16
build on it.

## What Changes

- A new default feature `run` in `mortise-core` with a `run` module tree: the kind of runner, the command
  line, detection from the folder and the shared setup of a started program.
- The command line is the prefix of the runner, then the artisan arguments as a list. It always adds
  `--no-interaction` and `--ansi`. It never goes through a shell. It is shown and copied as one line,
  quoted for the platform.
- Herd, Sail and Docker are detected from the folder: a Herd site link, `vendor/bin/sail`, a compose
  file. Herd on Windows is `herd.bat`.
- `git.rs` and `editor.rs` start their programs through the same setup: no console window on Windows,
  no inherited standard input.
- `APP_ENV` is read from `.env` with `APP_URL`, and no other line (Q15). Deviation S15 is removed.
- Settings keep the runner, the working folder and the time limit of each app. Old `settings.json`
  files still load.
- "Test" runs `artisan --version` through the chosen runner and shows the first line of the answer, or
  the failure of S1.4 c.
- Screens: S8.2 (Settings, PHP), the "Run PHP with" section of S1.3 with the failure of S1.4 c, and the
  PHP section of Settings in the palette.
- README line 7 no longer says Mortise needs no PHP and changes nothing in the app.

## Capabilities

### New Capabilities

- `php-runner`: how PHP runs for an app: the kinds of runner, detection, the command line, the test and
  what Settings keep.

### Modified Capabilities

- `app-access`: adds the rule that Mortise reads `APP_URL` and `APP_ENV` from `.env` and no other line.
- `add-panel`: adds the "Run PHP with" section and its failure.

## Impact

- `mortise-core`: new `run/` tree (`mod.rs`, `detect.rs`, `process.rs`), `git.rs`, `editor.rs`, `app.rs`
  (`.env`), `settings.rs`, `lib.rs`, `Cargo.toml` (feature `run`).
- `mortise-app`: `add.rs`, `settings_view.rs`, `palette.rs`, `strings.rs`, a new `php_settings.rs`,
  tests and image snapshots.
- Docs: `README.md`, `docs/deviations.md`.
- A dependency only if the design needs one for signals in milestone 11; none for milestone 10.
- Not run here: Herd (not installed on this Mac), Windows and Linux. CI covers the build and tests there.
