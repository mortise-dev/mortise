# Tasks

## 1. Core

- [x] 1.1 `Rules` gains `fail_ci`. `rules::load_file` reads `rules`, `ci` and `coverage` in one pass; a limit over 100 is invalid. `Rules::save` sets `ci.fail` and keeps the other keys. Verify with the tests of `boundary/rules.rs`
- [x] 1.2 `CoverageSettings` and `ReadOptions::coverage_given`; `read_app_cached` applies the order given, file, window, default. Verify with one test in `tests/read_app.rs`
- [x] 1.3 `cargo check -p mortise-core --no-default-features` still passes

## 2. Command line tool

- [x] 2.1 New crate `crates/mortise-cli`, binary `mortise`, `mortise-core` with `default-features = false`. Verify with `cargo tree -p mortise-cli -e features -i mortise-core`
- [x] 2.2 Arguments, `--help`, `--version`, usage errors (exit 2)
- [x] 2.3 `check`: read (exit 3 on `ReadError`), places of findings, text, json and github formats, exit 0 or 1, `ci.fail` off
- [x] 2.4 Text in `src/strings.rs`
- [x] 2.5 Tests on shop-api: one per exit code and one per format. Verify with `cargo test -p mortise-cli`

## 3. Window

- [x] 3.1 Rename the binary to `mortise-desktop`: `[[bin]]`, `default-run`, cargo-packager `binaries`, the Wayland app id
- [x] 3.2 The S5.3 switch shows and writes `fail_ci`; the hint is removed. Verify with an assertion in the existing rules test of `tests/boundary.rs`
- [x] 3.3 Images of S5.3 and S5.4 in both themes, looked at

## 4. CI and docs

- [x] 4.1 Job "Command line tool" in `ci.yml` on Linux, macOS and Windows: build the tool alone and expect exit 1 on shop-api. Verify with `actionlint`
- [x] 4.2 README, CONTRIBUTING, AGENTS, the S5.3 drawing of `docs/spec`, the documentation site (a how-to, a reference page, mortise.json, Boundary, Settings, Findings, architecture, build from source)
- [x] 4.3 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` on macOS; Windows and Linux through CI
- [x] 4.4 `openspec validate phase-4-cli-check --strict` passes

## Workflow follow-up

- Pull request to `main`; merge once CI passes on Linux, macOS and Windows.
- Archive the change through its own pull request.
