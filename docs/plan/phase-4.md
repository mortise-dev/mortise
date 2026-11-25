# Phase 4 plan

Outline, version 2, 26 November 2026. Version 1 was dated 23 November.

Status: approved, 26 November 2026. The owner accepts every recommendation in section 4. Milestone 24 and the P6 package work have started. A crates.io token is in the `release` environment, and the repository `nWidart/setup-mortise` exists. Version 2 draws the line between Mortise and Mortise Cloud, takes in the requests of the cloud, and renumbers the milestones after phase 3.
The published copy is https://6h4zaivwsd5c.postplan.dev/phase-4.html (version 13).

Phase 4 makes the boundary rules a contract that CI enforces. It adds one program, the command line tool `mortise`, with two commands. `mortise check` reads an app and fails the build when a rule breaks (M1). `mortise push` sends what it read to Mortise Cloud. The window gets the accepted public surface of a module (P6).
Everything reads the files of one app at one commit, as phase 1 does. Screen ids refer to the specification.

Phase 4 builds 2 screen ids: S5.8 and S5.9. It changes 2 of phase 1: Accept on S5.7, and "Fail CI when a rule breaks" on S5.3. The command line tool has no screen.

## 1. Mortise and Mortise Cloud

Mortise is free and MIT licensed. Mortise Cloud is the paid web service beside it, specified in the `mortise-cloud` repository (`docs/spec`, read at `8370ba8`). Phase 4 builds nothing that the cloud sells.

**The line.** Mortise covers one app, at one commit, on one machine. The cloud covers anything across time, across commits, across apps or across people.

| Mortise, free | Mortise Cloud, paid |
|---|---|
| Read an app: modules, links, cycles, statistics, coverage, findings | History of those reads over time, per app and per module |
| The two boundary rules, and later a third (cloud request R9). A rule is a check of one commit | Comparison of a pull request with its base, and the regression signal |
| `mortise check`: fails the build on any problem, and marks each problem on its line in the CI log (`--format github`) | Pull request comments and check runs through its GitHub App |
| The accepted public surface of a module (P6), stored in the app | Module owners, alerts by email, deploy markers |
| Several apps on the rail, each read on its own | One table across the apps of an organization |
| Live metrics of a running app, kept in memory only (phase 3) | Hosted runtime metrics and their history (out of cloud v1) |
| `mortise push`: the open source client that sends one read to the cloud | The ingest API that receives it |

**What Mortise leaves out, in phase 4 and later.** Each item would give away a feature the cloud sells:

- No baseline file and no "new problems only" mode. `mortise check` fails on every problem at the commit it reads. Only the cloud compares a commit with its base.
- No call to the GitHub API. The tool posts no comment, no check run and no review. It writes to standard output only.
- No report for GitHub code scanning (SARIF). Code scanning keeps alerts across commits, which is history.
- No stored history of reads, findings or coverage. The window shows the app as it is now. Nothing about past reads is written to disk.
- No module owners. The window does not read `CODEOWNERS` and has no owner field.
- No table that sums several apps, no alerts and no notifications.
- No record of deploys.

A change to the line is the owner's decision. A new item goes into this list, or into the cloud's scope, before it is built.

## 2. What changed from version 1

| Version 1 | Version 2 | Why |
|---|---|---|
| Milestones 22 to 24 | Milestones 24 to 28 | Phase 3 used 18 to 23. The push and its distribution are milestones of their own |
| One command, the rule check | `mortise check` and `mortise push` | The cloud asks for both (requests R1 to R8). The client is free; the service is not |
| Exit codes 0, 1, 2 | Exit codes 0 to 4, as cloud request R6 | CI runs both commands and must tell a broken rule from a broken setup |
| "A form for other programs behind an option" | `--format text\|json\|github` | Cloud request R2. Each stays one commit; none is history |
| An action for GitHub left for later | An action `nwidart/setup-mortise` in milestone 26 | Cloud request R7. The onboarding of the cloud needs it |
| A report for code scanning left until a team asks | Left out on purpose | Section 1 |
| Coverage limit is a desktop setting | Also in `mortise.json` | Cloud request R3. CI has no desktop settings; one limit for the team |
| Phase 3 first | Phase 3 is done | Built 24 November 2026 |
| File paths `rules.rs`, `surface_view.rs` | The folders of milestone 17 | Package by feature |

## 3. What was checked before writing this

| Subject | How | Result |
|---|---|---|
| Mortise | `main` at `2cb46f6` (v0.2.0 and the docs site). Read the Cargo files and the folder tree | The desktop binary is named `mortise`. `mortise-core` has the default features `watch`, `actuator`, `os-secrets` and `run`. Boundary code is in `mortise-core/src/boundary/` and `surface.rs`, and in `mortise-app/src/boundary/` |
| Mortise Cloud | `docs/spec` at `8370ba8`: scope, requests to Mortise, build plan, open questions | v1 sells history, pull request comments, owners, the multi-app view, alerts and deploy markers. `mortise check` and module extraction are named free. Requests R1 to R9 to Mortise. R4 and R5 are needed by week 9 of its build, R7 by week 15. The build starts in early December 2026, so about early February and mid-March 2027 |
| Crate name | crates.io API, 26 November 2026 | `mortise-cli` is free |
| `nwidart/laravel-modules` | Releases | 13.4.0 is the latest. No public surface yet |
| Release secrets | `gh secret list` | `RELEASE_TOKEN` and the six Apple secrets exist. No crates.io token |

Not checked: no code was written or run for this outline. Nothing was run on Windows or Linux.

## 4. Questions for the owner

Settled by earlier decisions:

- **A phase of its own.** Done. The spec names phase 4 for P6 and M1.
- **Where a public surface is stored (P6).** Phase 2, question 6: a `surface` key in each module's `module.json`, with three lists (classes, route patterns, console commands), read without PHP.
- **How Accept stores it.** Phase 2, question 6: a package command, `module:surface Billing --set=...`, run through the runner of milestone 11. Mortise still writes only `mortise.json`.

Open. Each has a recommendation; the outline below assumes it.

1. **The line with the cloud.** Recommended: as section 1, including the list of what Mortise leaves out.
2. **The name of the binary.** The desktop binary is `mortise`, and the cloud docs use `mortise check`. Recommended: the command line tool takes `mortise`; the desktop binary becomes `mortise-desktop`. The installers and the app name stay "Mortise".
3. **What the tool reports.** Recommended: cloud requests R2 and R6. Problems first, one line each with module, file and line. `--format text|json|github`. Exit 0 no problem, 1 a problem fails the check, 2 usage, 3 the folder cannot be read as an app, 4 the cloud refused a push.
4. **Where "Fail CI" is kept, and its default.** Recommended: a `ci.fail` key in `mortise.json`, on when absent, so the cloud's YAML works without it. Off: `check` reports and exits 0. The switch on S5.3 writes it.
5. **Coverage in `mortise.json` (cloud request R3).** Recommended: an optional `coverage` object with `limit` and `report`. When present it wins in the window and in the tool; the desktop setting is used only when it is absent. A flag of the tool wins over both.
6. **A rule for `mortise push` in `AGENTS.md`.** Recommended: "`mortise push` sends only the push document, to the cloud host, when CI or the user runs it. The token comes from `MORTISE_TOKEN` only and is never printed." The window gets no push button.
7. **How a team gets the tool.** Recommended: cloud request R7. Binaries for Linux x86_64 and arm64 (static, musl), macOS arm64 and Windows x64 with SHA-256 sums on each GitHub release. The action `nwidart/setup-mortise`, in its own repository. `cargo install mortise-cli` from crates.io.
8. **Signing the macOS binary.** Recommended: sign and notarize it with the app from the first release. The Apple secrets exist, and an unsigned binary would be stopped when a developer downloads it.
9. **Order and dates.** Recommended: milestones 24, 25 and 26 first, to meet the cloud's weeks 9 and 15 (cloud question Q10). Milestone 27 waits for P6 in a laravel-modules release.

## 5. Workspace changes

```
crates/mortise-cli/            new crate, binary mortise
  src/main.rs                  arguments, exit codes
  src/check.rs                 mortise check: text, json, github
  src/push.rs                  mortise push: environment of CI, retries, gzip
                               mortise-core with default features off,
                               plus a feature for the push client
crates/mortise-core/src/
  boundary/rules.rs            ci.fail and coverage in mortise.json
  surface.rs                   reads the surface key of module.json
  push/                        NEW, behind feature push: the push document,
                               SCHEMA_VERSION, relative paths
crates/mortise-app/
  Cargo.toml                   binary renamed mortise-desktop
  src/boundary/                the switch of S5.3; Accept on S5.7, S5.8, S5.9
```

The tool reads the app through the same core as the window. A rule is checked in one place, so the tool and the window cannot disagree. HTTP for the push reuses `ureq`, already used by the actuator client.

## 6. Milestones

| # | Milestone | Core and tool | Screens | Needs |
|---|---|---|---|---|
| 24 | Command line rule check | `mortise-cli`, `mortise check`, its formats and exit codes. `ci.fail` and `coverage` in `mortise.json`. The desktop binary renamed. Cloud R1, R2, R3, R6 | The switch of S5.3 | nothing |
| 25 | Push document and `mortise push` | The push document in `mortise-core`, its frozen shape and schema test. `mortise push` with `--dry-run`. Cloud R4, R5, R8 | none | The cloud's host (cloud Q1) for the default; its staging for the last test |
| 26 | Distribution | Release binaries with sums, signed on macOS. `nwidart/setup-mortise`. crates.io. Cloud R7 | none | nothing: token and repository exist |
| 27 | Accepted public surface | Read the `surface` key. The second rule checks it in the window and in the tool | Accept on S5.7, S5.8, S5.9 | P6 in a laravel-modules release |
| 28 | Hardening | The tool on three platforms in CI, on the fixture apps. Paths on Windows | Images for S5.8 and S5.9 in both themes, accessibility, docs site, deviations | nothing |

Cloud request R9, a rule of allowed dependencies, is a free rule and fits after phase 4. The cloud does not need it for its launch.

## 7. External dependencies

| Dependency | What phase 4 needs from it | Exists today | Owner | Blocks |
|---|---|---|---|---|
| `nwidart/laravel-modules` | P6: the `surface` key and `module:surface` | In progress | nwidart | Milestone 27 |
| Mortise Cloud | Its host name (Q1), a staging environment, and the push schema version 1 | Schema written, host not chosen | nwidart | The default host and the last test of milestone 25 |
| `nwidart/setup-mortise` | A repository for the action | Exists, empty | nwidart | Milestone 26 |
| crates.io token | To publish `mortise-cli`, as a secret of the `release` environment | `CARGO_REGISTRY_TOKEN` set | nwidart | Publishing there only |
| Apple secrets | Signing the macOS binary | Exist | nwidart | nothing |
| PHP for the app | Only to accept a surface through the package command | Runner built | The user | nothing |
| CI | A job that runs the tool on the fixture apps on three platforms | `ci.yml` exists. No new secret | Mortise | nothing |

## 8. Tests

- The rules are tested in `mortise-core` already. The tool is tested once at its own layer: its exit codes and its output on the fixture `shop-api`, which has a circle and three reads of Billing internals.
- One test per exit code of `check`: no problem, a problem with Fail CI on, a problem with Fail CI off, a folder that is not an app.
- The push document of `shop-api` is compared with a committed file and checked against a copy of the cloud's schema (cloud R8). The sending is tested against a local stub server: retries, a refused token, a fork without a token.
- Milestone 27 adds a fixture module with a `surface` key.

## 9. What is uncertain

| Uncertain | Why it matters | What settles it |
|---|---|---|
| When P6 ships in laravel-modules | Milestone 27 cannot start before | The package change |
| A request from teams for a feature on the cloud's side of the line | Building it free would undercut the cloud | The owner, by section 1 |
| Whether the cloud's weeks 9 and 15 hold | Milestones 25 and 26 are on the cloud's path | The cloud's start date and question Q10 |
| Paths in the output on Windows | Lines must open in editors and CI logs | The job of milestone 28 |
