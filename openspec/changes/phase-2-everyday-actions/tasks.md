# Tasks

## 1. Design

- [x] 1.1 Write `design.md` for the action list, the confirmation table, the output reading, the Actions tab, the dialogs, the toasts, the drawer sentence and the palette, and verify `openspec validate phase-2-everyday-actions --strict` passes

## 2. Core (12.1, opus)

- [x] 2.1 Record real output of `nwidart/laravel-modules` 13.3.0 into `fixtures/output/` in the form of milestone 11 (D9): `module:disable Checkout`, `module:enable Checkout`, `module:migrate Checkout` with one migration done, `module:migrate Shipping` failing at a migration, `module:migrate-rollback Checkout`, `module:lang Checkout`. Use a scratch app outside the repository and no agent variables in the environment. Verify each file parses as JSON, its `args` are unique across the folder, and the fake artisan prints it
- [x] 2.2 Add `catalog/mod.rs` and `catalog/actions.rs` with `Action`, `Target`, `Options`, `OptionSet`, `Context`, `Ask`, `Plan` and `plan` (D1, D2); export them from `lib.rs`. Verify with one table test of D2, one test of the `--force` invariant over every action, `production` and `pretend`, and one test of the word order with trimmed and empty text options
- [x] 2.3 Add `output.rs` with `FailedMigration`, `failed_migration`, `migrations_done` and `migration_file` (D3). Verify with tests on the recordings of 2.1: the failing migrate names its migration and module, the successful migrate counts one, a plain failure gives `None`, a missing file gives `None`

## 3. Interface (parallel after 2)

- [x] 3.1 (12.2, opus) Add `actions_view.rs` and wire it into `workspace.rs` and `app.rs` (D4 to D7): the list and panels of S10.1 and S10.2 a to f, options, the command line, Copy command, the run through `plan`, the busy tooltip, the no-runner tooltip, Run tests as the placeholder, J, K and Ctrl Enter; `Dialog::danger` and `Dialog::left` with the dialogs of D5; the toasts of D6 with "Show output"; the failure sentence of S11.1 c with "Open migration file" and "Dry run"; Migrate and Run tests in the header of S4.3 (D8). Text in `strings.rs`, each string not in the spec marked so. Verify with `tests/actions.rs` (D9) against the fake, the accessibility test, and the existing runner and keyboard tests
- [x] 3.2 (12.3, sonnet) Add the group "Actions for <module>" to `palette.rs` with `Command::Action` (D8); a row opens the Actions tab on its action. Verify with one case in `tests/palette.rs` through `entries`: the query "check" gives Migrate Checkout under "Actions for Checkout"
- [x] 3.3 Add deviations S37 to S41 (D4, D8) to `docs/deviations.md`, each with how it was checked. Verify `grep -n "S4[01]" docs/deviations.md`
- [x] 3.4 Add images for S10.1, S10.2 a to f and S10.3 in light and dark, and S11.1 c again, with `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images`, and look at each new or changed image

## 4. Integration

- [x] 4.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [x] 4.2 Run `openspec validate phase-2-everyday-actions --strict`

## Workflow follow-up

- Open one pull request to `main`, get a Sol review, handle its findings.
- Merge when CI passes on Linux, macOS and Windows.
- Archive the change on a new branch through its own pull request, with no reviewer.
