# Tasks

## 1. Design

- [x] 1.1 Write `proposal.md`, the delta specs and `design.md`, and verify `openspec validate phase-2-destructive-actions --strict` passes

## 2. Core (13.1 sonnet and 13.2 opus, in parallel)

- [x] 2.1 (13.1) Add `git::uncommitted` and `Module::uncommitted` (D1), read in `app.rs` with the churn. Verify with one test on a temporary repository: a changed, a staged, an untracked and an ignored file, an app in a subfolder of the repository, and `None` outside a repository
- [x] 2.2 (13.2) Record the six outputs of D9 from a scratch app with 13.3.0 into `fixtures/output/`. Verify each file parses, its `args` are unique across the folder, and the fake artisan prints it
- [x] 2.3 (13.2) Add the destructive lists, the targets each action takes, the drop options, the reset dry run, `Ask::name_to_type`, `typed_name_matches` and `Action::from_artisan` to `catalog/actions.rs` (D2). Verify with the tests D2 names, each rule once
- [x] 2.4 (13.2) Add `output::prohibited` and `migration_count` in place of `migrations_done` (D3), and move the toasts of migrate and roll back to it. Verify with tests on the recordings of 2.2: the prohibited reset is read as prohibited, a plain failure is not, reset and refresh give their counts

## 3. Interface (13.3 opus, after 2)

- [x] 3.1 Add the closed group, the four panels, production and the keys to `actions_view.rs` (D5). Text in `strings.rs`, each string not in the spec marked so. Verify with `tests/actions.rs`: closed on open, J stops at the group, Enter opens it, another module closes it, production turns the run button off and keeps Copy
- [x] 3.2 Add `lines`, `code` and `typed` to `Dialog` and the four dialogs of S10.5 in `app.rs` (D6). Verify with `tests/actions.rs`: Ctrl Enter on Delete opens the dialog with focus on Cancel, the button stays off for "checkout", turns on for "Checkout", Enter does not confirm, the delete runs, the toast shows and the index selects "All modules"; a reset dry run runs without a dialog
- [x] 3.3 Run again of the four, the toasts, and the selection after a delete (D7); the prohibited sentence in `runner_view.rs` (D4). Verify with one Run again case in `tests/actions.rs` and one prohibited case in `tests/runner.rs`
- [x] 3.4 Add the word rule to `palette.rs` (D8). Verify with one case in `tests/palette.rs`: "checkout" lists no destructive row, "delete checkout" lists Delete module Checkout
- [x] 3.5 Add or extend deviations in `docs/deviations.md` for what differs from the spec (the P4 fallbacks of the four, the group row's "closed"), each with how it was checked
- [x] 3.6 Add images for S10.4 a to d and S10.5 a to d, light and dark, with `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images`, and look at each new or changed image

## 4. Integration

- [ ] 4.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [ ] 4.2 Run `openspec validate phase-2-destructive-actions --strict`

## Workflow follow-up

- Open one pull request to `main`, get a Sol review, handle its findings.
- Merge when CI passes on Linux, macOS and Windows.
- Archive the change on a new branch through its own pull request, with no reviewer.
