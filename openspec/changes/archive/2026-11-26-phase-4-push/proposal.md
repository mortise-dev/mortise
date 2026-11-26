# Proposal

## Why

Mortise Cloud keeps the history of reads, compares a pull request with its base and comments on it.
It needs one read of an app per CI run, in a shape it can rely on. Its requests R4, R5 and R8
(`mortise-cloud/docs/spec/04-mortise-requests.md`) ask Mortise for that read and for the client that
sends it. Milestone 25 of the phase 4 plan builds both. The client is free; the service is not
(plan section 1).

## What Changes

- `mortise-core` gains a feature `push` and a folder `push/`. It turns an `AppSnapshot` into the push
  document of `02-snapshot-contract.md`, version 1: the envelope (`schema_version`, `generated_at`,
  `tool`, `app_path`, `git`, `ci`, `environment`) and the projected snapshot. The projection drops
  `root`, `app_url`, `app_env`, `git` and `package_data`, makes every path relative to the app with `/`,
  sets `uncommitted` to null, and sets `churn` to null on a shallow clone. `SCHEMA_VERSION` is a public
  constant. The folder also reads the git facts of the envelope (app path, commit, branch, shallow) and
  holds the size limits of the contract.
- The document's shape is frozen by a test: the document of `fixtures/shop-api` with a fixed envelope
  equals a committed file, is valid against a copy of the cloud's `push-document-1.schema.json`, and
  matches the cloud's `example-shop-api.json` except `churn`.
- `mortise push [PATH]` with `--coverage`, `--environment`, `--dry-run`, `--output`, `--commit`,
  `--branch`, `--pull-request`, `--base-branch`, `--base-commit` and `--repository`. It reads the token
  from `MORTISE_TOKEN` only and the host from `MORTISE_CLOUD_URL` (default `https://mortisecloud.com`).
  On GitHub Actions it fills the git and CI members from the environment and the event file. It checks
  the size limits, sends the document gzipped to `POST /api/v1/snapshots`, retries an outage, and
  prints the snapshot URL.
- Exit code 4: the cloud refused the document, or the document is over a size limit. An outage, a rate
  limit after retries and a plan limit warn and exit 0. A pull request from a fork without a token
  prints a notice and exits 0. No token elsewhere exits 2.
- `AGENTS.md` gains the rule for `mortise push`. The window gets no push button.

Out of scope, by section 1 of the plan: no comparison with a base, no history, no call to the GitHub
API, no comment, no check run. The client sends one read and prints the answer. Release binaries are
milestone 26.

## Capabilities

### New Capabilities

- `push-document`: the push document, its projection of `AppSnapshot`, its frozen shape and its size
  limits.
- `cli-push`: the command `mortise push`: arguments, token, CI environment, sending, retries and exit
  codes.

### Modified Capabilities

- `code-layout`: the core gains the module `push`, placed after `app` in the order of modules.

## Impact

- `mortise-core`: new feature `push` (no new dependency) and folder `push/`; `modules/git.rs` shares its
  `git` helper with it. New dev-dependency `boon` for the schema test.
- `mortise-cli`: new `push.rs` and `github.rs`; the hand parser is reshaped for flags without a value.
  New dependencies: `ureq` (already in the workspace), `flate2` (already in the lock through `ureq`),
  `jiff` (already in the workspace).
- CI: the job "Command line tool" also runs `mortise push --dry-run` on shop-api on three platforms.
- Docs: README, AGENTS, the documentation site (a how-to and the command reference),
  `licenses/THIRD-PARTY.md`.
