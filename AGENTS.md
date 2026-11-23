# AGENTS.md

Instructions for coding agents that work on Mortise. `README.md` describes the project for people.

## Orchestration

Act as the lead orchestrator for this project.

Review the entire specification and break it down into small, independently executable tasks with clear dependencies, acceptance criteria, and expected outcomes.

Before starting any task, evaluate its complexity, reasoning requirements, risk, and implementation scope. Determine the most appropriate model for each change rather than defaulting everything to Fable 5.2.

Use Sonnet for simple, repetitive, and well-defined changes that require minimal reasoning.

Use Opus for standard implementation, feature development, refactoring, test creation, and moderately complex debugging.

Use Fable 5.2 for architectural decisions, difficult debugging, ambiguous requirements, complex reasoning, and changes with significant system-wide impact.

Assign work to the most appropriate model wherever model-specific delegation is supported. Run independent tasks in parallel when appropriate, respect dependencies, and avoid conflicting changes.

Maintain responsibility for coordination, integration, code review, and final verification. Escalate work to a stronger model only when complexity, risk, or failed attempts justify it.

Before execution, show me the complete task breakdown, the model selected for each task, and a brief explanation of why that model is the best choice.

Your goal is to complete the entire specification with maximum quality while minimizing unnecessary token usage, execution time, and cost. Never use Fable 5.2 for work that Sonnet or Opus can reliably handle.

## What Mortise is

A desktop app, written in Rust with egui and eframe, for Laravel apps that use
`nwidart/laravel-modules`. It opens the folder of an app, lists its modules and shows what is in each one.
Phase 1 (read only) is built. The plan is `docs/plan/phase-1.md`.

## Rules that must hold

- Mortise runs a command in the app only when the user starts it, with the exact command on screen first,
  through the PHP the user chose for the app. Reading an app still needs no PHP.
- The only file Mortise writes in an app is `mortise.json`, and it asks before it creates it.
  A save goes through a short-lived temporary file beside it (`docs/deviations.md`, S28).
  Other files in the app are written by the package's commands, not by Mortise.
- The actuator client asks for the actuator base path (for its list of links), `health`, `info`, `mappings`
  and `scheduledtasks` only. It never asks for `env` and follows no redirect.
- A token or a password goes to the credential store of the operating system. It is never written to a file or a log.
- `unsafe` code is forbidden in the workspace (`unsafe_code = "forbid"`). Find a safe route or record a deviation.
- `mortise-core` has no interface dependency. Drawing belongs in `mortise-app` only.
- All use of the PHP parser (`mago-*`, pinned `=1.59.0`) stays in `mortise_core::php`.
  Moving the pin past 1.59 needs Rust 1.100: move the pin and `rust-toolchain.toml` together.
- Every change must work on macOS, Windows and Linux. Say what was not run on a platform.

## Commands

```
cargo run -p mortise-app -- /path/to/laravel-app
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs these on Linux, macOS and Windows (`.github/workflows/ci.yml`). Run all of them before a commit.
CI also checks the commit subjects, `licenses/THIRD-PARTY.md` (run `python3 tools/third-party-licenses.py`
after a dependency changed), `cargo audit`, secrets in the history and OpenSpec.
A job that is not required runs the package's commands in a Laravel app it creates with Composer.
To run it here, make the app as the job "Real PHP" in `ci.yml` does, then
`MORTISE_REAL_APP=/path/to/app cargo test -p mortise-core --test real_php -- --ignored --test-threads=1`.

The installers come from cargo-packager, configured in `crates/mortise-app/Cargo.toml`:

```
cargo install cargo-packager --locked --version 0.11.8
cargo packager -p mortise-app --release --formats app,dmg
```

The image tests in `crates/mortise-app/tests/images.rs` run on macOS only. After a change that is meant
to be seen, make the images again and look at each changed image before you commit it:

```
UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images
```

`cargo test -p mortise-app --test screens -- --ignored` writes every screen to `target/screens`.

Environment for work on Mortise itself:

- `MORTISE_CONFIG_DIR` names another folder for `settings.json`.
- `MORTISE_DATA_DIR` names another folder for the history of runs.
- `MORTISE_SECRETS=memory` keeps secrets out of the credential store.

## Tests

- Before adding a test, add the assertion to the test that already builds that fixture, payload or render. A new test needs a new fixture or a new branch, not a new field.
- Test a rule once, at the layer that owns it. A screen test in `mortise-app` proves the wiring once; it does not replay the cases of the `mortise-core` function behind it. Do not repeat a shared code path once per fixture app or once per caller.
- Do not test data the test defines, constants, removed features, egui or eframe defaults, or test helpers.
- Do not export a symbol only so a test can reach it.
- Advisory: a PR adding more than 400 net test lines should offset them elsewhere or say why in its body.

## Layout

| Path | Holds |
|---|---|
| `crates/mortise-core` | Reading an app into a snapshot, links, rules, findings, the scan, the actuator client, settings. Features `watch`, `actuator`, `os-secrets` are on by default; `testing` gives a stub actuator |
| `crates/mortise-app` | The window, theme, components and screens. Binary `mortise` |
| `crates/mortise-app/src/strings.rs` | All interface text |
| `fixtures` | Laravel apps for the tests. They are never installed or run |
| `docs/spec` | The design specification (HTML). Screen ids such as S4.2 refer to it |
| `docs/spec-build` | The scripts that build `docs/spec` |
| `docs/deviations.md` | Where the build differs from the specification, and why |
| `licenses`, `tools` | Third-party licences, the script that writes the list, the icon script and the release scripts |
| `.github/workflows` | `ci.yml` checks every change. `installers.yml` builds the installers. `release.yml` and `tag-release.yml` make a release (see `CONTRIBUTING.md`) |
| `openspec` | Specifications and change proposals (see below) |

## Design and text

- `docs/spec` is the source of truth for screens, tokens, components and text.
- When egui or the platform forces a difference, add a row to `docs/deviations.md`.
  Give the reason and say how the fact was checked (source read, compiled, run).
- Interface text follows section 10 of the specification: sentence case, no exclamation marks, no "please".
  An error says what happened, then what to do next. Text that is not in the spec is marked "not in the spec".
- Every control has an accessible name and a target of at least 24 by 24 px (`tests/accessibility.rs`).

## Writing style

Code comments, docs, commit messages and specs use plain, short English:
one idea per sentence, active voice, present tense, no marketing words.

Commit messages follow Conventional Commits with a crate scope where it fits:
`feat(core): ...`, `fix(app): ...`, `test(app): ...`, `docs: ...`, `ci: ...`.
Write the subject as what the change does, in lower case.

## OpenSpec

Planned changes go through OpenSpec (`openspec/`, schema `spec-driven`).

- `openspec/specs/` holds the specifications of what is built.
- `openspec/changes/` holds proposed changes; finished ones move to `openspec/changes/archive/`.
- `openspec/config.yaml` holds the project context given to every artifact.

Workflow: propose a change (`/opsx:propose`), build it (`/opsx:apply`), then archive it (`/opsx:archive`).
Use `/opsx:explore` to think a change through before a proposal. The `openspec` command line tool
(`openspec list`, `openspec show`, `openspec validate`) reads and checks the same files.
