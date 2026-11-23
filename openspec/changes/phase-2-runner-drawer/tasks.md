# Tasks

## 1. Design

- [x] 1.1 Write `design.md` for the runs, the history, the fake artisan and the drawer, and verify `openspec validate phase-2-runner-drawer --strict` passes

## 2. Core parts (parallel)

- [x] 2.1 (11.3, sonnet) Record real output of `nwidart/laravel-modules` 13.3.0 into `fixtures/output/*.json` in the form of D8, from an app made in a scratch folder outside the repository: `module:make`, a `module:make-*` generator, a generator that finds an existing file, `module:migrate`, `module:seed`, a failing command, `--help`. Verify each file parses as JSON and holds its command, args and exit code
- [x] 2.2 (11.4, sonnet) Add `run/ansi.rs` with `Style`, `Span`, `spans` and `plain` (D6); move `plain` out of `run/mod.rs`. Verify with one table test of D6 in `ansi.rs` and the existing `tests/run.rs`
- [x] 2.3 (11.5, sonnet) Add `run/history.rs` with `History`, `Entry`, `Ended`, `HISTORY_LIMIT` and `data_dir` (D7). Verify with temp-folder unit tests: the 51st run drops the oldest and its log, a damaged `runs.json` reads empty and the next record works, output past 1 MB is cut and `cut` is set
- [x] 2.4 (11.2, opus) Extend the fake artisan (D8): `hold`, `ignore-term`, `ask`, `orphan`, `orphan-wait`, and the recordings of `fixtures/output/`. Add the same binary to `mortise-app` behind its `testing` feature (D11). Verify the existing `tests/run.rs` passes and a test prints a recording through `CARGO_BIN_EXE_fake-artisan`

## 3. Process model

- [x] 3.1 (11.1, opus) Add `Runs`, `Run`, `RunEvent`, `RunEnd`, `StartError`, `OUTPUT_LIMIT` and `stop_asks_first` (D1, D2, D3, D5) with `rustix` for `cfg(unix)`; make `capture` use the same stop and join. Verify with the core tests of D11 in `tests/run.rs`, and `cargo build -p mortise-core --no-default-features` without a warning
- [x] 3.2 Run `python3 tools/third-party-licenses.py` after the dependencies changed. Verify `licenses/THIRD-PARTY.md` lists `rustix` and `jiff`

## 4. Drawer

- [x] 4.1 (11.6, opus) Add `runner_view.rs` and wire it into `app.rs` and the workspace layout (D9, D10): states a to g, Stopped, did not start with "Open settings", the handle, the keys, the stop dialog, history with Up, Down, Enter, recording into the history, `Config::data_dir`. Text in `strings.rs`. Verify with `tests/runner.rs` (run, stop, fail, history, did not start) and the accessibility test
- [x] 4.2 Add deviation S36 (D10), and S37 if lines of the two streams swap in a test or an image. Name `MORTISE_DATA_DIR` in `AGENTS.md`. Verify `grep -n "S36" docs/deviations.md`
- [x] 4.3 Add images for S11.1 a to g, light and dark, with `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images`, and look at each new or changed image

## 5. Integration

- [x] 5.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [x] 5.2 Run `openspec validate phase-2-runner-drawer --strict`

## Workflow follow-up

- Open one pull request to `main`, get a Sol review, handle its findings.
- Merge when CI passes on Linux, macOS and Windows.
- Archive the change on a new branch through its own pull request, with no reviewer.
