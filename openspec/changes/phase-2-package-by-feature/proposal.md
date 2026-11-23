# Proposal

## Why

The code of both crates is organised by layer, not by feature. A feature lives in many places, and a change to one feature touches many files.

- `mortise-core` has four module cycles: `catalog` and `package`, `run` and `editor`, `git` and `run`, and `catalog` and `run` through `Target`.
- `app.rs` of `mortise-app` wires every feature in one `impl` block of about 3,900 lines.
- One `strings.rs` of about 3,400 lines holds the text of every feature. 476 of its names have one user, and the file is a conflict point for every feature change.
- CI builds `mortise-core` with default features only. A gate that breaks `--no-default-features` or one feature goes unseen.

Milestone 15 (`phase-2-package-data`) is archived, so no feature branch is open. This is the time to move the code.

## What Changes

- `mortise-core/src` groups its files in folders: `modules/`, `app/`, `boundary/`, `runtime/`, `package.rs`, `run/`, `actions/`, `generate/`, `palette/`, and the new `program.rs`. `catalog/` and `output.rs` dissolve into `actions/` and `generate/`. The modules then depend on each other in one direction only.
- `mortise-app/src` mirrors the core by name, one folder per feature, with `ui/` for the shared drawing and `app/` for the wiring. `app.rs` splits into one wiring file per feature. `strings.rs` splits into one file per feature folder, plus `src/strings.rs` for text that more than one feature shows.
- CI builds `mortise-core` with `--no-default-features` and with each of `watch`, `actuator`, `os-secrets`, `run` and `testing`.
- Three pull requests: PR 1 the core and the CI feature matrix, PR 2 the app, PR 3 the strings split and the documents. PR 1 and PR 2 touch disjoint files.
- No behaviour change. Nothing a user sees changes. The 140 snapshot images stay byte for byte. No test is added or changed beyond its import lines.
- The public names of `mortise-core` do not change. `lib.rs` keeps every re-export and the paths `ansi`, `history`, `catalog_cache` and `testing`.
- Two crates stay. A crate per feature costs more than it gives (`design.md`, section 3.1).

## Capabilities

### New Capabilities

- `code-layout`: the checkable rules of the target structure. It is added only because `openspec validate --strict` needs a spec delta.

### Modified Capabilities

None.

## Impact

- Files in both crates move or split: 22 in the core, 24 of 28 in the app.
- The `use` lines of the tests in `crates/mortise-app/tests` change, about 35 lines. The tests do not move.
- `AGENTS.md` (Layout table, strings rule), `README.md`, `CONTRIBUTING.md`, `docs/deviations.md` and `docs/plan/phase-2.md` (milestone 17 row) change.
- `.github/workflows/ci.yml` gains the feature matrix for `mortise-core`.
- Out of scope: more crates, controllers per feature, splitting the large views, one action enum per tab, renames to the interface vocabulary, merging the test binaries.
