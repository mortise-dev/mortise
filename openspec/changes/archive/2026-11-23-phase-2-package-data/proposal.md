# Proposal

## Why

`nwidart/laravel-modules` 13.4.0 is released (tag `v13.4.0`, 23 November 2026). It gives the data that
milestones 12 to 14 worked without: P3 `module:generators --json`, P2 `--dry-run --json` and P5 `--json`
on every generator, P4 `module:migrate-status --json`, P7 `module:test`, P8 `module:lang --json`.
Milestone 15 of `docs/plan/phase-2.md` uses them. Screens that now show a fallback (deviations S39, S41,
S42, S44) can show what the spec draws, for an app whose package has the data.

## What Changes

- A version gate: Mortise uses P2 to P8 only when the app's `composer.lock` gives a released
  `nwidart/laravel-modules` 13.4.0 or later. A dev version, a branch, or a pre-release keeps the
  behaviour of milestones 10 to 14. Mortise never guesses a feature from a failed command.
- Reads. A read is a package command that only gives data: `module:generators --json`, a generator with
  `--dry-run --json`, `module:migrate-status --json`. The user starts each one with its exact line on
  screen, as for every command. A read runs through the app's runner and its one-run-per-app rule, and
  shows its result where the data goes, not in the runner drawer.
- P3: the picker reads the app's own generators, cached per app and package version in Mortise's data
  folder. Generators of the app's own code show under "Other". Until the user reads them, an app with
  13.4 or later uses the shipped description of 13.4.0; an older app keeps `v13_3.json`.
- P2: a form offers "List files", which runs the dry run of the line on screen. The file list of S9.2
  and S9.4 shows the files, and S9.3 takes a sub-generator whose file exists out of the command, in
  plain sight. The primary button counts the files.
- P5: a generator run of a 13.4 app carries `--json`; its result comes from the JSON document, not from
  the sentences of the output. A missing or malformed document falls back to the output lines, and the
  result says so.
- P4: "Waiting" and "Last batch" in S10.1 and S10.2, the counts on Migrate and Roll back, the migrations
  in the roll back dialog, "9 ran" and the tables named in S10.4 and S10.5, the migration line and
  "Migrate (1)" on S4.3, and "To do" in Findings (S6.4). The state is read only when the user asks. A migrate,
  roll back, reset, refresh or fresh forgets the state it may have changed, so the user reads it again.
- P7: Run tests runs `module:test <module> --json` (or `--all`), with `--filter` and `--parallel`.
  The placeholder of S10.2 e goes for a 13.4 app. The toast counts the tests.
- P8: Check translations runs `module:lang --json`. Its report gives the check "Translations are
  complete" and one warning per language that lacks keys or files, in Findings (S6.4) and the Summary.
- Recordings of each command with 13.4.0 in `fixtures/output/`, replayed by the fake artisan, and the
  real-PHP tests extended with a dry run, a JSON run, `module:migrate-status --json`, `module:test` and
  `module:lang --json`.
- Images for every changed screen and S9.3, deviations updated (S39, S41, S42 and S44 now hold for an
  older package only), README.

## Capabilities

### New Capabilities

- `package-data`: the version gate, reads and their rules, how a JSON document is taken from a run and
  what happens when it is missing or malformed, the generator catalog cache, the migration state and
  when it is read, the translation report in Findings.

### Modified Capabilities

- `module-generators`: the catalog from P3, the file list and S9.3 from P2, the result from P5.
- `module-actions`: Run tests runs `module:test`; the data that needs the migration state.
- `destructive-actions`: the panel and the dialogs name the migrations and tables with P4.
- `real-php-check`: the ignored tests also run the 13.4 commands.

## Impact

- `mortise-core`: a `package` module (gate, JSON types, document reading, catalog cache), a read runner
  beside `Runs`, `ReadOptions` and `AppSnapshot` carry the package data, `findings.rs` (translations,
  to do), `php.rs` (tables a migration creates), `catalog/` (v13_4.json, `module:test`, `--json`
  words). No new dependency.
- `mortise-app`: `generate_view.rs`, `actions_view.rs`, `summary.rs`, `findings_view.rs`, `app.rs`,
  `runner_view.rs` (standard output kept apart), `strings.rs`, tests and images.
- `fixtures/output/`: recordings with 13.4.0. `tests/real_php.rs`.
- Docs: `docs/deviations.md`, `README.md`.
- Not run here: Windows and Linux. CI runs the build and tests there.
