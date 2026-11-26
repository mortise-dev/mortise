# Design

## Context

`read_app_cached` turns a folder into an `AppSnapshot` without PHP, and every type in it derives
`Serialize`. Milestone 24 built `mortise check` in `crates/mortise-cli` with a hand parser, exit codes
0 to 3 and 5, and escapes for every printed value. The cloud's contract
(`mortise-cloud/docs/spec/02-snapshot-contract.md`, `03-ingest-api.md`, requests R4, R5, R6 and R8)
fixes the push document, the endpoint, the answers and what the client does with each.

## Goals / Non-Goals

**Goals:**

- One projection, in `mortise-core`, from `AppSnapshot` to the push document. The cloud defines no
  second model of an app.
- A test that fails when a member of the document changes name, type or place.
- A client that never fails a build for an outage or a billing state, and always fails it for a
  wrong token or a wrong document.
- The token never leaves the `Authorization` header.

**Non-Goals:**

- Anything on the cloud's side of the line (plan section 1): comparison, history, GitHub API calls.
- `POST /api/v1/deploys`. `push --environment` puts the environment in the document and the cloud
  records the deploy from it (`03-ingest-api.md`). One call does both.
- Waiting for the cloud to process a snapshot. The client prints the URL and stops.
- Release binaries (milestone 26).

## Decisions

### The projection takes the snapshot by value and names every field

`push_document(app: AppSnapshot, context: PushContext) -> PushDocument` destructures `AppSnapshot`
without `..`. A new field of `AppSnapshot` then does not compile until the projection keeps or drops
it. The snapshot member reuses `Layout`, `Module`, `Link`, `Cycle`, `Rules`, `RulesSource`, `Findings`,
`Version` and `CoverageReport`; only the envelope types are new.

The core writes a path outside the app as it is, such as a reference file of a modules folder that lies
beside the app. The projection runs every path field through the same function: the layout, module
folders, the coverage report, reference files and the files and folders of findings. A match on `Detail`
names every variant, so a new variant with a path must be handled.

A path becomes a `PathBuf` made from a string with `/` between parts. serde writes a `PathBuf` as its
string, so the document has `/` on Windows too. A path outside the app folder, such as a coverage
report given with `--coverage ../build/clover.xml`, is written relative with `..`. On another
Windows drive, where no relative path exists, only the file name is written. No absolute path leaves
CI.

Members added to the core after the cloud's example stay in the document: `coverage_ignored` of the
snapshot and `fail_ci` of `Rules`. Version 1 allows an optional member anywhere, and the parser
message of `mortise.json` is on the contract's list of what is sent.

### The feature `push` has no dependency

The projection, the git facts and the limits need only serde. The feature keeps them out of the
window's build. HTTP lives in the command line tool, as section 5 of the plan draws: the tool owns the
retries, gzip and the answers, and the core owns the document.

### Git facts come from the core's `git` helper

`push::read_repository(root)` runs `git rev-parse --show-toplevel`, `rev-parse HEAD`,
`symbolic-ref --short -q HEAD` and `rev-parse --is-shallow-repository` through the helper that the
churn read uses: no lock, no prompt, no file system monitor. Reading git is not running a command in
the app; the PHP rule is untouched. `app_path` is the canonical app folder relative to the canonical
top level, so `/var` and `/private/var` on macOS agree.

### Where each git member comes from

A flag wins. The commit is `HEAD` of the checkout that was read, so the snapshot names the files it
holds; `GITHUB_SHA` is used only when git cannot say. A job that checks out the head of a pull request,
or a `pull_request_target` run, then sends the commit it read, and the tool warns with both SHAs. An
event file that cannot be read is an error only when `--pull-request`, `--base-branch` and
`--base-commit` do not stand in for it.

On GitHub Actions, the environment fills what no flag gave: `commit` when git gives none (`GITHUB_SHA`),
`repository` from `GITHUB_REPOSITORY`, the run from `GITHUB_RUN_ID`, `GITHUB_RUN_ATTEMPT`,
`GITHUB_SERVER_URL`. On `pull_request` and `pull_request_target` the event file gives the number,
`head.sha`, `head.ref`, `base.sha` and `base.ref`. On other events `branch` comes from `GITHUB_REF`
when it names a branch. Git fills what is still missing: `commit` from `HEAD`, `branch` from the
checked-out branch, and always `shallow` and `app_path`.

Elsewhere, `--pull-request` needs `--base-branch` and `--base-commit`, which the schema requires with
it. `head_commit` is then the commit that was read: outside GitHub no merge commit is made.

A commit that is not 40 or 64 lower-case hexadecimal characters, an environment that breaks the
contract's pattern, a pull request number below 1 or a repository that is not `owner/name` is a usage
error (exit 2). The cloud would refuse them with 422.

### A fork is a pull request whose head repository differs from its base

GitHub gives no secrets to `pull_request` runs from a fork. The tool reads `pull_request.head.repo` and
`pull_request.base.repo` from the event file. A different `full_name`, or a head repository that is
gone (`null`), is a fork. `pull_request_target` runs get secrets, so the rule does not apply there.

### Order of the steps

1. Parse the arguments (exit 2).
2. Without `--dry-run`: no token and a fork pull request, a notice and exit 0; no token elsewhere,
   exit 2. The app is not read.
3. Read the app (exit 3) and the git facts (exit 2 when the commit is unknown).
4. Build the document. Write it to `--output`, or print it with `--dry-run`.
5. Check the limits (exit 4), so a dry run also says when a document would be refused.
6. Send, unless `--dry-run`.

### Sending

`ureq` with rustls and the platform verifier, as the actuator client. `max_redirects(0)`: a redirect
could take the token elsewhere; it comes back as an answer the tool refuses (exit 4, the host is
wrong). A global timeout of 30 s per attempt. `MORTISE_CLOUD_URL` is parsed as a URI (`ureq::http`,
no new dependency). It names no user, and it is `https`, or `http` with the host exactly `localhost`,
`127.0.0.1` or `[::1]`, so the token never crosses a network in clear. A plain `http` request uses no
proxy: a proxy from the environment would read the token. Text from the cloud (the URL, warnings, the
problem and its errors) is printed with the token, and its secret part, replaced by `[MORTISE_TOKEN]`. The body is the
compact JSON, gzipped with `flate2` (already in the lock through `ureq`). The answer body is read up to
64 KiB.

| Answer | Exit |
|---|---|
| 200, 202 | 0. Print the URL and each warning |
| 402, 403 | 0. Warn with the title and detail of the problem |
| 429, any 5xx, no answer | Retry 3 times, then warn and exit 0 |
| Any other status | 4. Print the title, the detail and each error pointer |

The waits are 2, 4 and 8 s, or `Retry-After` in seconds when the answer gives it, at most 60 s. The
sender takes the wait as a function, so the unit test of the retries runs without waiting. The
integration tests answer with `Retry-After: 0`.

### Limits before sending

The core checks the counts of the contract (2,000 modules, 50,000 links, 100,000 references, 50,000
findings) and the length of every path the schema bounds and of each reference's class (1,024
characters). The tool checks the compact document against 16 MiB and the gzipped body against 4 MiB. Other
lengths are bounded by the app's own files and the cloud's schema check catches the rest (exit 4).

### Arguments stay a hand parser

`push` has ten flags with a value and one without. A separate value that starts with `-` is a flag
left without its value, as in `--output --dry-run`, and is a usage error; `--output=-x` stays possible. `clap` would add about ten crates, their licences
and audit entries, and it writes its own help and errors, which would bypass `strings.rs` and the text
rules of section 10. The parser is reshaped instead: an iterator gives each argument as a folder or a
flag, and a flag asks for its value only when it takes one. `check` and `push` share it.

### The schema test uses `boon`

A JSON Schema draft 2020-12 validator is needed for `prefixItems`, `if`/`then`, `oneOf` and `not`.
Writing one is out of scope. `boon` passes the draft 2020-12 test suite and has fewer dependencies
than `jsonschema`, whose default features pull an HTTP client. It is a dev-dependency of
`mortise-core` only: it is not in a build that ships.

## Risks / Trade-offs

- The end-to-end test of R5 (a snapshot in the cloud's staging from a GitHub Actions run) waits for
  that staging. Mitigation: the stub server tests every answer of `03-ingest-api.md`.
- `pull_request_target` runs read the base commit unless the workflow checks out the head. The tool
  sends `GITHUB_SHA` as R5 says; `--commit` corrects it.
- A Dependabot pull request also gets no secrets but is not a fork. It exits 2 without a token, as any
  other run. The how-to says to skip the step for it.
