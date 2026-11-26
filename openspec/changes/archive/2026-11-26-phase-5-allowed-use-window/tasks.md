# Tasks

One branch, `feat/allowed-use-window`, in its own worktree. One pull request to `main`.

## 1. Specification

- [x] 1.1 The third row of S5.3 and the text of a break in `docs/spec-build`, and `docs/spec` built from it. Verify with `python3 build.py`

## 2. Core

- [x] 2.1 `Link::outside_allowed`, set where the links are built, read by `is_problem` and by the findings of the rule. Verify with the test of `fixtures/layered` in `tests/read_app.rs`

## 3. Window

- [x] 3.1 The third row of S5.3 with its switch and its line. Verify with one test in `tests/boundary.rs` on `layered`, and the line without the key on shop-api
- [x] 3.2 The break on the graphs, in "Links that break a rule", in the files of the link and in the table of links. Verify in the same test
- [x] 3.3 The text of the spec for the findings of the rule, in the window and in `mortise check`
- [x] 3.4 Images of S5.3, S5.4 and the new `s5.1-outside-list` in both themes, each looked at. The switch in `tests/accessibility.rs`

## 4. Docs

- [x] 4.1 The docs site pages on Boundary, Findings, `mortise.json` and adding rules
