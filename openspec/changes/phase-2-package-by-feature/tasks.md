# Tasks

Every step: no behaviour change, no new test, no changed assertion. Files move with `git mv`. After each commit run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS, and check that `git status --porcelain crates/mortise-app/tests/snapshots` prints nothing. After each core commit also run `cargo check -p mortise-core` with `--no-default-features` and with each of `--features watch`, `actuator`, `os-secrets`, `run`, `testing`. Windows and Linux are checked by CI only; say so in each pull request. Commit subjects are `refactor(core): ...` and `refactor(app): ...`.

## 1. P0 Propose (sonnet)

- [x] 1.1 Write `proposal.md`, `design.md`, `tasks.md` and the `code-layout` delta. Accept when `openspec validate phase-2-package-by-feature --strict` and `openspec validate --all --strict --no-interactive` pass

## 2. C0 CI feature matrix (sonnet)

- [ ] 2.1 In `.github/workflows/ci.yml`, build `mortise-core` with `--no-default-features` and with each of `watch`, `actuator`, `os-secrets`, `run`, `testing`. Accept when the new job is green on the current code, and fails if one gate is broken

## 3. C1 Program (sonnet, PR 1)

- [ ] 3.1 New `program.rs`. Move `command()` out of `run/process.rs` and `split_words()` out of `editor.rs`. Update `git.rs`, `editor.rs`, `run/mod.rs`. Accept when checks are green, `git.rs` and `editor.rs` no longer name `run`, `run` no longer names `editor`, and `command()` is still ungated

## 4. C2 Modules (sonnet, PR 1)

- [ ] 4.1 Create `modules/`: move `module.rs`, `layout.rs`, `kind.rs`, `composer.rs`, `clover.rs`, `git.rs`, `code.rs`. Move `Target` from `catalog/actions.rs` to `modules/target.rs`. Accept when checks are green and `lib.rs` exports the same names

## 5. C3 App, boundary, runtime (sonnet, PR 1)

- [ ] 5.1 Create `app/` (`app.rs`, `live.rs`), `boundary/` (`links.rs`, `rules.rs`) and `runtime/` (`actuator.rs`, `runtime.rs` as `live.rs`, `environment.rs`, `secrets.rs`). Accept when checks are green and the feature matrix builds. The `actuator` and `watch` gates move from `lib.rs` to two `mod.rs` files

## 6. C4 Actions and generate (opus, PR 1)

- [ ] 6.1 Dissolve `catalog/` and `output.rs` into `actions/` and `generate/` as in design section 3.2. Move `ships()` and `generator_gives_data()` from `package.rs` to `generate/catalog.rs`; `lib.rs` still exports `generator_gives_data` under the same name, and the test `gives_data` keeps its assertions. Split the tests of `output.rs` with the functions they test. Keep `pub use generate::cache as catalog_cache` in `lib.rs`. Accept when checks are green, the feature matrix builds, the count of `#[test]` in the crate is unchanged (123), the `run` gate covers both halves of `output.rs`, and `package.rs` names neither `Catalog` nor anything in `generate/`

## 7. C5 Palette (sonnet, PR 1)

- [ ] 7.1 Create `palette/`: move `matcher.rs` and `run/recent.rs`. Accept when checks are green and `run/` names neither `Action` nor `Catalog`. The core follows the dependency direction of "Measured again on main"

## 8. A1 Shared drawing helpers (sonnet, PR 2)

- [ ] 8.1 Move `alert_box` from `summary.rs` to `draw.rs`. Move `EmptyMark` and `empty_state` from `workspace.rs` to `draw.rs`. Accept when checks are green, `dialog.rs`, `actions_view.rs` and `generate_view.rs` no longer name `summary`, and `surface_view.rs` no longer names `workspace`

## 9. A2 Selection and WorkAction (sonnet, PR 2)

- [ ] 9.1 Move `Selection` from `app.rs` to `index.rs`. Rename `BoundaryAction` to `WorkAction` and move it to `workspace.rs`. Update the test imports. Accept when checks are green and `index.rs` no longer names `app`

## 10. A3 ui folder (sonnet, PR 2)

- [ ] 10.1 Create `ui/`: move `theme.rs`, `fonts.rs`, `draw.rs`, `dialog.rs`, `toast.rs`. Add one `../` to each `include_bytes!` and `include_str!` in `fonts.rs` and `theme.rs`. Accept when checks are green and the token test in `theme.rs` still reads `docs/spec/tokens.html`

## 11. A4 Feature folders (sonnet, PR 2)

- [ ] 11.1 Move the 14 view files into feature folders as in design section 3.3, and add each `mod.rs`. Accept when checks are green and the tests import the new paths

## 12. A5 app folder (sonnet, PR 2)

- [ ] 12.1 Move `app.rs` to `app/mod.rs`, and `chrome.rs`, `rail.rs`, `menu.rs`, `workspace.rs` beside it. Fix the two `include_bytes!` of the icon. Update `main.rs` and the tests (`workspace::Tab`, `chrome::Chrome`). Accept when checks are green

## 13. A6 Split the wiring (opus, PR 2)

- [ ] 13.1 Split `app/mod.rs` into `keys.rs`, then `generate.rs`, `actions.rs`, `runs.rs`, then `apps.rs`, `settings.rs`, `palette.rs`, `runtime.rs`. One commit each. Method bodies do not change. A moved method becomes `pub(super)` only where another file calls it. The `cfg` lines move with their methods. Measure the line ranges of `app.rs` again first. Accept when checks are green after each commit, `git diff --color-moved` shows moved blocks, `use` lines and visibility only, and `app/mod.rs` is under about 1,500 lines

## 14. S1 Split strings (opus, PR 3)

- [ ] 14.1 Split `strings.rs`. A name with one user moves to that feature's `strings.rs`. A name used by `app/` only goes to the feature its wiring file belongs to, or to `app/strings.rs`. A name with users in two features stays in `src/strings.rs` with the shared formatters. Name shared text as `crate::strings::X`, with no glob re-export. The 10 tests move with their functions. One commit per feature. Accept when checks are green, the count of `pub const`, `pub fn` and `pub static` over all `strings.rs` files is 604, no name is defined twice, every "not in the spec" mark is kept, and the 140 snapshot images are unchanged

## 15. D1 Documents (sonnet, PR 3)

- [ ] 15.1 Update `AGENTS.md` (Layout table, the strings rule, the path of `mortise_core::php` if written as a file), `README.md`, `CONTRIBUTING.md` and `docs/deviations.md` for every path that moved. In `docs/plan/phase-2.md` add milestone 17 to the table and leave section 3 as history. Accept when `grep -rn '_view\.rs\|catalog/' AGENTS.md README.md CONTRIBUTING.md docs/deviations.md` finds nothing stale

## 16. Archive

- [ ] 16.1 Archive `phase-2-package-by-feature` after the three pull requests merge. Accept when `openspec validate --all --strict --no-interactive` passes
