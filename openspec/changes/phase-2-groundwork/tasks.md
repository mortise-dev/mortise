# Tasks

## 1. Rules and spec (docs)

- [x] 1.1 Replace the "runs nothing in the app" rule in `AGENTS.md` and `openspec/config.yaml` with the rule of question 5, keep the `mortise.json` rule and say that files are written by the package's commands. Verify with `grep -n "runs nothing" AGENTS.md openspec/config.yaml` (no match)
- [x] 1.2 Update section 12.2 of the spec through `docs/spec-build`: P9 and A1 met, P10 and A2 in part, 43 generators, one prohibit flag per destructive command, S9.1 reads "42 generators for Billing, and New module". Rebuild `docs/spec` and verify the text is in the built HTML

## 2. Core: links and rule findings

- [x] 2.1 Leave files in `tests/` out of links, circles and rules. Verify with an assertion in the test that already builds a fixture with a test file that uses another module
- [x] 2.2 Count a rule finding only when both modules of the link are on, and a circle only when every member is on. Boundary draws the link in the off style. Verify with a core test on a fixture with an off module

## 3. App: screens

- [x] 3.1 One function gives the display name of an app. Findings, Boundary and the palette use it. Verify in one screen test that a renamed app shows its new name
- [x] 3.2 Put the Add panel in a scroll area. Verify that the Add button is reachable at 150% text size
- [x] 3.3 Add the Summary line "Part of a circle counted on <module>" (not in the spec) for the other members of a circle. Verify in the summary test of a circle fixture
- [x] 3.4 Draw the code stamp in a graph node and the name only when it fits. The full name is the tooltip and the accessible name. Add a row to `docs/deviations.md`. Verify with the 12-module graph in `tests/boundary.rs` and the images

## 4. Integration

- [x] 4.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [x] 4.2 Run `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images` and look at each changed image before it is committed
- [x] 4.3 `openspec validate phase-2-groundwork` passes

## Workflow follow-up

- Open one pull request to `main`, attach a Sol reviewer, handle its findings.
- Merge when CI is green on Linux, macOS and Windows.
- Archive the change on a new branch through its own pull request, with no reviewer.
