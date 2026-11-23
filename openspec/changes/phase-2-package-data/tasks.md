# Tasks

## 1. Design (15.1 opus)

- [x] 1.1 Write `proposal.md`, the delta specs and `design.md`, and verify `openspec validate phase-2-package-data --strict` passes

## 2. Recordings (15.2 sonnet)

- [x] 2.1 Make a scratch app with `nwidart/laravel-modules` 13.4.0 (PHP 8.4, Composer, SQLite), with modules Billing and Checkout as D10 needs, and record each command of D10 into `fixtures/output/` with `made_with` naming 13.4.0 and a `root` key. Verify each file parses, `args` are unique across the folder, and note the byte size of the P3 line in the README
- [x] 2.2 Teach the fake artisan to replace `root` with its working folder, written with `/`. Verify with one test in `testing.rs` or the existing replay test
- [x] 2.3 Correct the package facts of `design.md` where the recordings differ
- [x] 2.4 Extend `crates/mortise-core/tests/real_php.rs` (ignored) with a dry run that leaves the app unchanged, a `--json` generator run, `module:migrate-status --json`, `module:test` and `module:lang --json` (D10). Run it against the scratch app with `MORTISE_REAL_APP`

## 3. Core groundwork (15.3a opus, after 2)

- [x] 3.1 Add `mortise_core::package`: `gives_data` (D1), `DataError` and `document` (D2), the document types of P2/P5, P4, P7 and P8, and `with_json`. Test the gate once and `document` on the recordings
- [x] 3.2 Add `run::Read` (D3) and keep standard output apart in `AppRuns`. Raise the line limit of standard output only if 2.1 measured a P3 line over 64 KiB
- [x] 3.3 Add `PackageData` to `ReadOptions` and `AppSnapshot` (D7) with the merge rules of a state read and a translation report, and `php::created_tables`. Test the merge and `created_tables` once each

## 4. Package data in the screens (after 3; 4.1, 4.3, 4.4, 4.5 in parallel; 4.2 after 4.1)

- [x] 4.1 (15.3 opus) P3: `catalog/v13_4.json` from the recording, `Catalog::shipped`, the read of the app's generators with its line, the cache, the picker lines, "Other" (D4). Test in `tests/generate.rs`
- [x] 4.2 (15.4 opus) P2 and P5: "List files", the file list of S9.2 and S9.4, S9.3, the counted buttons, the own-file case (D5); `--json` runs and their result with the fallback (D6). Test against the fake artisan, with the app unchanged after a dry run
- [ ] 4.3 (15.5 opus) P4: the state box and its reads, the read after a migration command, S10.1 to S10.5 with counts, migrations and tables, S4.3 (D7). Test in `tests/actions.rs`
- [ ] 4.4 (15.6 opus) P7: `module:test` in `plan` with `--filter` and `--parallel`, the panel and the toasts (D8). Test the plan once in core, the toast once in `tests/actions.rs`
- [ ] 4.5 (15.7 opus) P8 and To do: `module:lang --json`, the report into `PackageData`, the check, the warnings and the To do group in Findings and the Summary (D9). Test the findings once in core, the wiring once in `tests/findings.rs`

## 5. Images, accessibility, docs (15.8 sonnet, after 4)

- [ ] 5.1 Images, light and dark, for each changed screen and S9.3, with `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images`; look at each new or changed image
- [ ] 5.2 Accessible names and 24 px targets of the new controls in `tests/accessibility.rs`
- [ ] 5.3 `docs/deviations.md`: S39, S41, S42 and S44 hold for an app before 13.4; add the rows of the reads, "List files", the state box, the toasts and sentences not in the spec, each with how it was checked
- [ ] 5.4 README: what 13.4 adds

## 6. Integration

- [ ] 6.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [ ] 6.2 Run `openspec validate phase-2-package-data --strict`
- [ ] 6.3 Two pull requests to `main`: generate data (1 to 4.2 and their images and deviations), then action data (4.3 to 5). A Sol review on each. Archive after the second
