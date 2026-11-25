# Design

## Context

`read_app_cached` already turns a folder into an `AppSnapshot` without PHP, and `Findings` already
sorts problems first. `mortise-core` builds with `--no-default-features`: the CI job "Check core
features" proves it. `Rules::load` and `Rules::save` own `mortise.json`; the save goes through a
temporary file beside it and keeps the other keys (S28). Coverage today comes from `ReadOptions`,
filled from the desktop settings of the app.

## Goals / Non-Goals

**Goals:**

- One read of an app, in `mortise-core`, for the window and the tool. A rule is checked in one place.
- The tool is small: one command, three formats, four exit codes.
- The keys a team shares live in `mortise.json`, read in the same place as the rules.

**Non-Goals:**

- `mortise push`, the push document and its frozen shape (milestone 25).
- Release binaries, the GitHub Action, crates.io (milestone 26).
- Anything on the cloud's side of the line (plan section 1).
- Showing in Settings that `mortise.json` overrides the coverage fields. The spec draws no text for
  it; the findings show the limit in use.

## Decisions

### `ci.fail` is a field of `Rules`

S5.3 edits the rules through `WorkAction::SetRules(Rules)`, and `Rules::save` is the one save path
that asks before it creates the file. A `fail_ci` field reuses both: the switch is one more checkbox
with the same action. `save` sets `fail` inside an existing `ci` object and keeps its other keys.

### `coverage` is read with the rules, applied in the read

`rules::load_file` reads the whole file once into `(Rules, CoverageSettings, RulesSource)`.
`Rules::load` keeps its signature. A file that cannot be read, or whose `coverage.limit` is over 100,
is `RulesSource::Invalid` and gives the defaults for all three keys, as a broken `rules` key does today.

`read_app_cached` picks each coverage value, first to last, from `ReadOptions::coverage_given` (the
flags of the tool), the file, then `ReadOptions::coverage_limit` and `coverage_report` (the window).
The order lives in one place, so the window and the tool cannot apply it differently.
`CoverageSettings` serves as both the file's `coverage` object and the given values, since both are
"a limit and a report, each optional".

Alternative rejected: each caller merges the file and its own settings before the read. That needs a
second read of `mortise.json` per read and puts the order in two callers.

### The flag `--coverage` is relative to the current folder

A path on a command line is completed by the shell from the current folder. The tool makes it
absolute before the read; a path that is not a file is a usage error (exit 2), because falling back
to another report in silence would check against the wrong numbers. `coverage.report` in the file is
relative to the app folder, like the desktop setting.

### Places of the findings

The text and github formats need a file and a line, which `Findings` does not carry. The tool takes
them from the snapshot:

- `CircularUse`: the first reference of the link from the carrier to the module of the circle it uses
  (`AppSnapshot::circle_neighbour`), the link the window opens the circle on.
- `ReadsInternals`: one line per file of the link that reads an internal class, at the first such
  line, with the internal classes that file names. This marks each file, as R2 asks.
- `InvalidManifest`, `NoManifestName`, `StaleStatus`: their file, and the line when there is one.
- `NamespaceMismatch`, `LowCoverage`, `MissingTranslations`: the module folder, which is not a file.
  The github format gives no `file=` for them.

Paths are those of the core: relative to the app, `/` between parts on every platform. In the github
format GitHub places a mark on its line only when the app folder is the repository root. Paths relative
to the repository need the app path from git, which belongs with `mortise push` (R5 `app_path`).

### JSON is `Findings` only

`--format json` prints `serde_json::to_string_pretty(&snapshot.findings)` and nothing else on standard
output, as R2 asks. The places above are not in it. The shape is not frozen here; R8 freezes the push
document in milestone 25.

### Output streams and exit codes

The report goes to standard output; a usage error, a read error, an unreadable `mortise.json` and the
note that `ci.fail` is off go to standard error. Exit codes: 0 pass, 1 fail, 2 usage, 3 `ReadError`.
A closed standard output (`| head`) is ignored.

### Arguments by hand

`check` has one positional argument and four flags, each with a value, given as the next argument or
after `=`. A hand parser is about 70 lines and adds no dependency, no licence and no audit entry.
Milestone 25 adds `push` with about ten flags; a parser crate can be weighed there.

### The desktop binary is `mortise-desktop`

The cloud's documents use `mortise check`, so the tool takes `mortise`. `default-run`, `[[bin]]` and
the cargo-packager `binaries` entry change. `product-name` stays "Mortise", so the DMG, `Mortise.app`,
the NSIS setup and the installer file names do not change. cargo-packager 0.11.8 names the Linux
`.desktop` file after the main binary (`src/package/deb/mod.rs`, read), so the Wayland app id becomes
`mortise-desktop` to keep matching it.

## Risks / Trade-offs

- Someone who runs `mortise` from a Linux package or `target/` gets the window no longer. Mitigation:
  README, CONTRIBUTING, AGENTS and the site name both binaries.
- A team that sets `coverage` in `mortise.json` sees its desktop coverage fields ignored with no note
  on Settings. The findings show the limit in use. A note needs text the spec does not have.
- GitHub marks for an app in a subfolder fall back to the run summary. Recorded on the site.
