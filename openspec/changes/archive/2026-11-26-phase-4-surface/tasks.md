# Tasks

## 1. Core

- [x] 1.1 `AcceptedSurface` in `surface.rs`: read from `module.json`, normalized lists, `has_class`, `has`, `to_stdin`, `with_differences`; `surface_args`; `SurfaceAnswer`
- [x] 1.2 `Module::surface`, read with the manifest; the second rule checks it in `boundary/links.rs`
- [x] 1.3 `Proposal::surface`, `take_choices_of` and `differences`
- [x] 1.4 `stores_surface` for 13.5 and later
- [x] 1.5 Standard input for a run and a read
- [x] 1.6 Tests: `boundary/links.rs`, `tests/read_app.rs`, `tests/surface.rs`, `tests/run.rs`; the committed push document. Verify with `cargo test -p mortise-core`

## 2. Window

- [x] 2.1 Accept on S5.7 and S5.9: the dialog with the exact line, the read with standard input, the toasts
- [x] 2.2 S5.8: the accepted surface, Change, Scan again, what is left out
- [x] 2.3 S5.9: the differences, Space and click, Keep the accepted surface, Accept the changes
- [x] 2.4 Accept off with its reason; the rule sentence and the count of accepted modules in the Boundary tab
- [x] 2.5 Text in `strings.rs`; tests in `tests/surface.rs`, `tests/boundary.rs`, `tests/accessibility.rs`; images S5.3, S5.8 and S5.9 in both themes. Verify with `cargo test -p mortise-app`

## 3. Command line tool, fixtures and docs

- [x] 3.1 The sentence of a finding names an accepted surface
- [x] 3.2 `Modules/Users` of shop-api with a `surface` key; the recording of `module:surface` 13.5.0
- [x] 3.3 `docs/deviations.md` S71 to S75; the documentation site
- [x] 3.4 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` on macOS; Windows and Linux through CI
- [x] 3.5 `openspec validate phase-4-surface --strict` passes

## Workflow follow-up

- Pull request to `main`.
- Archive the change through its own pull request.
