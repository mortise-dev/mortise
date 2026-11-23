# Tasks

## 1. Design

- [x] 1.1 Write `proposal.md`, the delta spec and `design.md`, and verify `openspec validate phase-2-generate --strict` passes

## 2. Core (14.1 sonnet, 14.2 opus, 14.3 sonnet, in parallel)

- [ ] 2.1 (14.1) Write `crates/mortise-core/src/catalog/v13_3.json`: the 42 generators of 13.3.0 in the shape of P3 (D1), read from the package source at the tag `v13.3.0`. Verify it is valid JSON, has 42 entries sorted by name, and each entry's arguments and options match the source of its command
- [ ] 2.2 (14.2) Add `catalog/generators.rs` with the types, `Catalog::from_json`, `v13_3`, `get`, `grouped`, `shipped_for` (D1), `Filled`, `words`, the name rules (D2) and `replace_words` (D4). Verify with the tests of D8, each rule once
- [ ] 2.3 (14.3) Record the outputs of D8 into `fixtures/output/` from a scratch app with 13.3.0. Verify each file parses, its `args` are unique across the folder, and the fake artisan prints it
- [ ] 2.4 (14.3) Add `output::generated_files` and `autoload_failed` (D3). Verify with tests on the recordings: created, left, replaced and the model run that left a file

## 3. Interface (14.4 opus, after 2)

- [ ] 3.1 Add `generate_view.rs` with the picker, the forms and the result (D5, D6), and replace the placeholder of the Generate tab. Text in `strings.rs`, each string not in the spec marked so
- [ ] 3.2 Add the runs, the result, Replace and its dialogs, the selection after `module:make`, and the toasts in `app.rs` (D6)
- [ ] 3.3 Add "New module" to the index footer, Ctrl N (Cmd N), the keys 3 and G, and the palette group (D7)
- [ ] 3.4 Add `tests/generate.rs` and the palette case (D8) against the fake artisan
- [ ] 3.5 Add or extend deviations in `docs/deviations.md`, each with how it was checked
- [ ] 3.6 Add images for S9.1, S9.2, S9.4, S9.5 and S9.6, light and dark, with `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images`, and look at each new or changed image

## 4. Integration

- [ ] 4.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [ ] 4.2 Run `openspec validate phase-2-generate --strict`

## Workflow follow-up

- Open one pull request to `main`, get a Sol review, handle its findings.
- Merge when CI passes on Linux, macOS and Windows.
- Archive the change on a new branch through its own pull request, with no reviewer.
