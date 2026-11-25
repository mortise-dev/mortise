# Proposal

## Why

Phase 4 makes the boundary rules a contract that CI enforces (plan v2, approved 26 November 2026).
Today the rules exist only in the window: nothing fails a build when a pull request breaks one, and
the switch "Fail CI when a rule breaks" on S5.3 is disabled with "Needs the command line tool, which
does not exist yet." Mortise Cloud also asks for a command line tool (its requests R1, R2, R3 and R6)
so that its onboarding YAML can run `mortise check`. Milestone 24 builds it.

## What Changes

- A new crate `crates/mortise-cli`, binary `mortise`, with one command: `mortise check [PATH]
  [--coverage <clover.xml>] [--coverage-limit <0-100>] [--format text|json|github] [--fail-on warnings]`.
  It depends on `mortise-core` with `default-features = false`. It runs no PHP, needs no window
  system and writes nothing in the app.
- Exit codes as cloud request R6: 0 no failing finding, 1 a finding fails the check, 2 usage error,
  3 the folder cannot be read as an app. 4 is kept for `mortise push` (milestone 25).
- **BREAKING** for people who run the binary by name: the desktop binary is renamed from `mortise`
  to `mortise-desktop`. The installers, the bundle and the app name stay "Mortise".
- `mortise.json` gains `ci.fail` (boolean, on when absent). Off, `check` prints its report and
  exits 0. The S5.3 switch works and writes it through the existing save path. Its hint goes.
- `mortise.json` gains an optional `coverage` object `{ "limit": <0-100>, "report": "<path>" }`.
  Each key that is set wins over the desktop setting, in the window and in the tool. A flag of the
  tool wins over both.
- A CI job builds the tool alone on Linux, macOS and Windows and checks `fixtures/shop-api`,
  expecting exit 1.

Out of scope, by section 1 of the plan (the line with the paid Mortise Cloud): no baseline file or
"new problems only" mode, no comparison with a base branch, no GitHub API call, no SARIF, no stored
history of reads, no owners. `check` judges one commit. Release binaries are milestone 26.

## Capabilities

### New Capabilities

- `cli-check`: the command line tool and its command `check`: arguments, formats, exit codes, and
  what it does not do.
- `rules-file`: the keys of `mortise.json` beyond the rules: `ci.fail` and `coverage`, how they are
  read, how they are written, and which source of the coverage settings wins.

### Modified Capabilities

- `code-layout`: the command line tool keeps its text in `mortise-cli/src/strings.rs`.

## Impact

- `mortise-core`: `boundary/rules.rs` reads and writes `ci` and reads `coverage`; `Rules` gains
  `fail_ci`; new public `CoverageSettings`; `ReadOptions` gains `coverage_given`; `read_app_cached`
  applies the order flag, file, window, default. No new feature and no new dependency. The crate
  already builds with no default features.
- `mortise-cli`: new. Dependencies `mortise-core` and `serde_json`, both in the workspace already.
  The arguments are parsed by hand: one command with four flags does not justify a parser crate.
- `mortise-app`: binary renamed; the Wayland app id follows it; the S5.3 switch; images of S5.3 and
  S5.4 in both themes.
- CI: a new job "Command line tool". Docs: README, CONTRIBUTING, AGENTS, the documentation site,
  the S5.3 drawing of the specification.
