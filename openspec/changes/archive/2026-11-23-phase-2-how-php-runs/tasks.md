# Tasks

## 1. Design

- [x] 1.1 Write `design.md` for the `run` tree of milestones 10 and 11 and verify `openspec validate phase-2-how-php-runs --strict` passes

## 2. Core groundwork (parallel)

- [x] 2.1 Add `run/process.rs` with `command()` (stdin null, `CREATE_NO_WINDOW` on Windows) and the feature `run` (D1, D2). `git.rs` and `editor.rs` use it. Move the splitter of `editor_arguments` into `editor::split_words`. Verify `cargo test -p mortise-core` and `cargo build -p mortise-core --no-default-features` without a warning
- [x] 2.2 Add `RunnerKind` with `prefix()` and `from_line()`, and `CommandLine` with `artisan()` and `display()` and both quoting functions (D3, D7, D8). Verify with the unit tables of D7 and one test of `from_line` for each kind
- [x] 2.3 Add `run/detect.rs` with `detect()` and `herd_config_dir()` (D11). Verify with temp-folder tests: parked, linked (`cfg(unix)`), Sail with and without a compose file, each compose name, the order, empty folder, broken `config.json`
- [x] 2.4 Read `APP_ENV` from `.env` with `APP_URL` and no other line; keep it on `AppSnapshot`. Remove deviation S15 from `docs/deviations.md`. Verify with the assertion added to the existing `.env` test in `app.rs`

## 3. Settings

- [x] 3.1 Add `PhpSettings` and `AppEntry::php` with `#[serde(default)]` (D8). Verify an old `settings.json` without `php` loads with no runner, the app folder and 10 minutes, and that a Docker runner is saved and loaded again, in the existing settings tests

## 4. Test

- [x] 4.1 Add the fake artisan binary and `testing::fake_artisan_main()` (D13), `process::capture` (D9, with the `PATH` folders) and `test_php` with `TestOutcome` and `Failure` (D12). Verify with `crates/mortise-core/tests/run.rs`: answered without colour codes, exited with the stderr line, not found, timed out, no command

## 5. Screens

- [x] 5.1 Settings get the PHP section (S8.2): the table of apps with runner, command and last test, and the detail with "Run PHP with", working folder and time limit. Text in `strings.rs`. Verify with one screen test in `tests/settings.rs` that edits a runner and saves it
- [x] 5.2 The Add panel gets "Run PHP with" (S1.3) with the proposal of `detect`, Test, and the failure of S1.4 c with "Add <name>, read only". Add saves the runner shown. Verify with one screen test in `tests/apps.rs` that a Custom program that does not exist shows the failure and the app is still added
- [x] 5.3 The palette opens Settings on the PHP section. Verify with an assertion in the existing palette test
- [x] 5.4 Add the deviation rows: `-T` in the Docker line (D5), Sail disabled on Windows (D6), "Not tested" after a restart (D8). Update README line 7. Verify `grep -n "needs no PHP" README.md` finds no match
- [x] 5.5 Add images for S8.2 and the "Run PHP with" part of S1.3, light and dark, with `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images`, and look at each changed or new image

## 6. Integration

- [x] 6.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [x] 6.2 Run `python3 tools/third-party-licenses.py` if a dependency changed, and `openspec validate phase-2-how-php-runs --strict`
- [x] 6.3 Check Herd detection and `herd.bat` on Windows if a Windows machine is reachable; else record them as not run in the report

## Workflow follow-up

- Open one pull request to `main`, get a Sol review, handle its findings.
- Merge when CI passes on Linux, macOS and Windows.
- Archive the change on a new branch through its own pull request, with no reviewer.
