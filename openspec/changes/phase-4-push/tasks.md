# Tasks

## 1. Core

- [x] 1.1 Feature `push` and folder `push/`: envelope types, `SCHEMA_VERSION`, `push_document`, relative paths. Verify with `cargo check -p mortise-core --no-default-features --features push`
- [x] 1.2 `push::read_repository`: app path, commit, branch, shallow, through the `git` helper of `modules/git.rs`
- [x] 1.3 Limits of the contract on `PushDocument`
- [x] 1.4 Copies of `push-document-1.schema.json` and `example-shop-api.json`; one test on shop-api: committed file, schema with `boon`, the cloud's example except `churn`. Verify with `cargo test -p mortise-core --test push`

## 2. Command line tool

- [x] 2.1 Reshape the hand parser for flags without a value; parse `push` and its flags; validate values (exit 2)
- [x] 2.2 GitHub Actions environment and event file; git facts; fork detection; token rules
- [x] 2.3 Dry run and `--output`; limits (exit 4)
- [x] 2.4 Sending: gzip, no redirect, `https` only, answers and exit codes, retries with an injected wait
- [x] 2.5 Text in `src/strings.rs`; help lists both commands and exit code 4
- [x] 2.6 Tests: a unit test of the retries; integration tests against a stub server (accepted, 401, 422, 429 and 503 then exit 0, fork, no token, limit, dry run with the event fixture). Verify with `cargo test -p mortise-cli`

## 3. CI and docs

- [x] 3.1 The job "Command line tool" also runs `mortise push --dry-run` on shop-api. Verify with `actionlint`
- [x] 3.2 AGENTS rule; README; the documentation site (a how-to for sending to Mortise Cloud, the command reference)
- [x] 3.3 `python3 tools/third-party-licenses.py` after the dependency change
- [x] 3.4 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` on macOS; Windows and Linux through CI
- [x] 3.5 `openspec validate phase-4-push --strict` passes

## Workflow follow-up

- Pull request to `main`. The end-to-end test of R5 waits for the cloud's staging.
- Archive the change through its own pull request.
