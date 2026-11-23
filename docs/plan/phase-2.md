# Mortise phase 2: build plan

Status: approved, 23 November 2026. The owner accepts every recommendation in section 2, except
question 8: no release is cut yet.

How the work runs:

- Stage 0 builds the groundwork: the new rule of question 5, section 12.2 of the spec (question 9)
  and the six points of question 7. Then milestones 10 to 16 follow, in order.
- Each stage and milestone is one OpenSpec change, built in its own T3 thread and merged through one
  pull request. A Sol reviewer is attached to that pull request. The pull request that archives the
  change gets no reviewer.
- The package work of question 1 goes to the `laravel-modules` project as a brief.

Phase 2 writes. Mortise runs the commands of `nwidart/laravel-modules` in the app: it creates modules
and files, and it runs the actions of a module. Every command is on screen before it runs, can be
copied instead of run, and its output is shown as the command wrote it. The design is in `docs/spec/`.
Screen ids such as S9.2 refer to it.

Phase 2 builds 13 screen ids: S8.2, S9.1 to S9.6, S10.1 to S10.5, S11.1, the "Run PHP with" part
of S1.3, the "New module" footer action of S3.1, and the Migrate and Run tests buttons of S4.3.
S5.8 and S5.9 wait for P6 (section 2, question 6), in whichever phase that lands.

## 1. What was checked before writing this

| Subject | How | Result |
|---|---|---|
| Mortise | `main` at `a122a07`, 75 commits, 277 tests | Phase 1 is built and merged. No release is cut yet |
| `nwidart/laravel-modules` | Read `master` at `5b0307af`, 41 commits after 13.3.0, not released | P9 is met in part: a `modules` health indicator and per module indicators named `module:{module}:{key}`. A `modules` endpoint lists name, enabled, version, description, priority and providers. That is not P10: it has no routes or tasks. New generator `module:make-health-indicator`, so 43 generators. Each destructive command has its own prohibit flag. P1 to P8 do not exist |
| `nwidart/laravel-actuator` | Read `main` at `b77ae76`, tag 1.3.0 | A1 is met: custom endpoints with read, write and delete operations. A2 is met in part: a `prometheus` endpoint on `nwidart/laravel-metrics`, with no module label. New `migrations` health indicator: Out of service while migrations wait, for the whole app |

Not checked: no package command was run for this plan. The command lines of Herd, Sail and Docker
were not run. Nothing was run on Windows or Linux.

## 2. Questions for the owner

Each question has a recommendation. The plan below assumes the recommendation.

1. **Order of the work.** Recommended: add P2, P3, P4, P5, P7 and P8 to `laravel-modules` first and
   release them as 13.4. Mortise phase 2 runs in parallel and uses each addition when the app's
   package version has it. Without it, Mortise uses the fallback the spec names under each screen.
   The owner keeps both packages, so the package work is not a wait on someone else.
2. **Preview without a dry run (P2).** The spec gives no fallback for the file list of S9.2 and S9.4.
   Recommended: without P2, the preview shows the command and the sentence "The files are listed
   after the run" (not in the spec). Mortise does not copy the package's path logic.
3. **The test command (P7).** Recommended: `module:test {module*} {--all} {--filter=} {--parallel}`.
   It runs `php artisan test` (or `vendor/bin/pest` or `vendor/bin/phpunit` when `artisan test` is
   absent) on the `tests` folder of each module. With `--json` it also prints one JSON line with
   passed, failed and skipped counts, which the toast "31 tests passed in Checkout" needs.
4. **Data from the package (P2 to P5, P8).** Recommended: one `--json` option on each command
   (`module:make-*`, `module:migrate-status`, `module:lang`), and one new `module:generators --json`
   for P3. JSON on standard output, text unchanged without the option. P1 is left out: no screen uses it.
5. **The rules in `AGENTS.md` and `openspec/config.yaml`.** "Mortise needs no PHP and runs nothing in
   the app" stops being true. Recommended new rule: "Mortise runs a command in the app only when the
   user starts it, with the exact command on screen first, through the PHP the user chose for the app.
   Reading an app still needs no PHP." "The only file Mortise writes in an app is `mortise.json`"
   stays true for Mortise itself; files are written by the package's commands.
6. **Where a public surface is stored (P6).** Recommended: a `surface` key in `module.json`, with
   three lists: classes, route patterns, console commands. Mortise reads it from the file, without
   PHP. Accepting runs a package command (`module:surface Billing --set=...`), so Mortise still
   writes no file but `mortise.json`. Built as its own change, not in this plan.
7. **The six points left from the council review (PR #3).** Recommended:
   - Turned-off modules get boundary problems while their row shows no state. Count a rule finding
     only when both modules of the link are on. Boundary still draws the link, in the off style.
   - Files in `tests/` count toward links and circles. Leave them out of links, circles and rules.
     A test that builds another module's model with its factory is not a boundary of the running app.
   - The other members of a circle read "Good". Keep the count of Q4. Add one line to their Summary:
     "Part of a circle counted on Checkout" (not in the spec).
   - The app name from Settings does not reach Findings, Boundary or the palette. A bug: one
     function gives the display name, and every screen uses it.
   - The graph of 12 modules cuts every name. Draw the code stamp in the node and the name only
     when it fits; the full name is the tooltip and the accessible name. Record a deviation.
   - The Add panel does not scroll at 150% text size. A bug: put the panel in a scroll area.
8. **The first release.** The `release` environment and the repository have no secrets. The Release
   workflow needs `RELEASE_TOKEN`, and Tag release needs the six Apple secrets, by design.
   Recommended: add all seven and cut v0.1.0 before phase 2 starts. An unsigned app is stopped by
   Gatekeeper on the first open, which is a poor first run.
9. **Section 12.2 of the spec is behind the packages.** Recommended: update it in one docs commit:
   P9 and A1 met, P10 and A2 in part, 43 generators, the prohibit flags. S9.1 then reads
   "42 generators for Billing, and New module".

## 3. Workspace changes

No new crate. The core gets one module tree for running commands, behind a new default feature `run`,
so a command line tool that only reads can leave it out.

```
crates/mortise-core/src/
  php.rs          unchanged: the PHP parser
  run/
    mod.rs        RunnerKind, CommandLine, how PHP runs for an app
    detect.rs     Herd, Sail, Docker from the folder
    process.rs    spawn, stream, stop, time limit
    ansi.rs       SGR codes to four styles
    history.rs    the last 50 runs per app
  catalog/
    mod.rs        generators and actions as data
    v13_3.json    the description of the 42 generators of 13.3, used without P3
  output.rs       reading "Generating file ... DONE" and the other lines, used without P5
crates/mortise-app/src/
  runner_view.rs  S11.1
  generate_view.rs S9.1 to S9.6
  actions_view.rs S10.1 to S10.5
  php_settings.rs S8.2 and the "Run PHP with" part of S1.3
```

`git.rs` and `editor.rs` already start programs. Their `Command` setup (no console window on Windows,
no inherited standard input) moves into `run/process.rs` and all three use it.

## 4. The model

```
RunnerKind      Herd | Sail | Docker { service } | Custom { line }
                stored per app in Settings, with the working folder and the time limit

CommandLine     the prefix of the runner, then artisan arguments as a list.
                Always adds --no-interaction and --ansi. Never goes through a shell.
                Shown and copied as one line, quoted for the platform.

Run             one process: id, app, command line, started, state, output
  state         Running { elapsed } | Done { ms } | Failed { code, ms } | Stopped | NotStarted { error }
  output        lines of spans: text, dim, success, error

Catalog         per package version: generators (name, arguments, options, group) and actions.
                From `module:generators --json` (P3) when the app's package has it, else v13_3.json.

DryRun          (P2) the files a generator would write, and whether each exists
MigrationState  (P4) per module: ran with batch, waiting
GeneratorResult (P5) per file: created, replaced, left because it exists
```

Rules the core holds, each tested once there:

- One run at a time per app. A second start is refused with the name of the running command.
- A run that asks for input cannot hang: `--no-interaction`, standard input closed, and the time limit.
- `APP_ENV` is read from `.env` with `APP_URL`, and no other line (Q15). Deviation S15 is removed.
- Production: migrate and seed ask first and pass `--force`. The four destructive commands do not run.
- A command that needs confirmation in the package gets `--force` only after Mortise asked.
- Stop: on macOS and Linux SIGTERM, then SIGKILL after 3 s. On Windows the process is ended.
  Through Sail or Docker, the PHP process in the container may go on; the drawer says so.

## 5. Milestones

Each ends in an app that runs. Each is one OpenSpec change (`/opsx:propose`), built, reviewed,
merged and archived before the next. Each is reported with what works, what was checked and how,
deviations, and what was not checked.

| # | Milestone | Core | Screens and components |
|---|---|---|---|
| 10 | How PHP runs | `run/mod.rs`, `detect.rs`, `APP_ENV`, settings fields, Test runs `artisan --version` | S8.2, "Run PHP with" in S1.3 with the failure of S1.4, the PHP section of Settings in the palette |
| 11 | Runner drawer | `process.rs`, `ansi.rs`, `history.rs`. Output kept to 1 MB per run in Mortise's data folder | S11.1 a to g. Ctrl \` shows and hides, Ctrl . stops, the handle between 120 px and half the workspace, history with Up, Down, Enter. "The command did not start" with "Open settings" |
| 12 | Everyday actions | The action list, the confirmation rules of S10.3, the production rule | S10.1, S10.2 a to f, S10.3 and its other dialogs. Migrate and Run tests in the header of S4.3. Actions in the palette. Toasts. Run tests shows the placeholder of S10.2 until P7 |
| 13 | Destructive actions | Git count of files that are not committed in a module, the typed-name check | S10.4 a to d, S10.5 a to d. The closed group, the five ways of keeping them apart, the refusal when the app prohibits a command. After a delete the index selects "All modules" |
| 14 | Generate | `catalog`, `output.rs`, name validation | S9.1, S9.2 without the file list, S9.4, S9.5, S9.6. "New module" in the index footer, Ctrl N, generators in the palette, "Replace" with its confirmation |
| 15 | Package data | Read P2, P3, P4, P5, P7 and P8 when the package has them, by its version in `composer.lock` | The file list of S9.2, S9.3, S9.4. "Waiting" and "Last batch" in S10.1 and S10.2, tables named in S10.3 to S10.5, migration state on S4.3, "To do" and translations in S6.4. Run tests runs |
| 16 | Hardening | | Images for each new screen id in both themes, accessibility names, Windows and Linux looked at, the real-PHP CI job, README, deviations |
| 17 | Package by feature | One folder per feature in `mortise-core/src`: `modules`, `app`, `boundary`, `runtime`, `actions`, `generate`, `palette`. `lib.rs` keeps its public names. CI builds each feature alone | One folder per feature in `mortise-app/src`, `ui/` for the theme and components, `app/` for the wiring, a `strings.rs` per feature. No screen changes |

Milestone 15 starts when `laravel-modules` 13.4 is released. Milestones 10 to 14 need nothing new
from the package.

The package work, in `laravel-modules`, one pull request each: P3 `module:generators --json`, P2
`--dry-run --json` on every generator, P5 `--json` on every generator, P4
`module:migrate-status --json`, P7 `module:test`, P8 `module:lang --json`. Then release 13.4.

## 6. Tests and continuous integration

### Core

- A fake artisan: a small binary in `mortise-core` behind the `testing` feature. It reads its
  arguments and prints recorded output of the real package, with ANSI codes and exit codes, and can
  wait, ask for input or ignore SIGTERM. It works the same on the three platforms, unlike a shell
  script. Tests reach it with `CARGO_BIN_EXE_`.
- The recorded outputs come from running the commands once on a real app with 13.3.0. They are
  stored in `fixtures/output/` with the command that made each one.
- Unit tests for the parts with rules: the command line and its quoting on each platform, detection,
  the ANSI parser, the output reader, the history limit, the production rule, the confirmation table.
- A test asserts that a fixture app is not changed by any test that does not run a command.

### Interface

- `egui_kittest` drives each screen against the fake artisan: run, stop, fail, history, the dialogs,
  the typed name, the keys of S10.4. The harness keeps failing on a control without a name.
- Image tests for each new screen id, light and dark, on macOS, as in phase 1.

### Continuous integration

A new job on Linux, not required, runs real commands: PHP 8.4, Composer, a Laravel app created at
test time with `laravel-modules`, and the core tests marked `#[ignore]` that run `module:make`,
a generator, `module:migrate` on SQLite and `module:delete`. It is the only job that needs the network
for Composer. It catches a change in the package's output before a user does.

## 7. Risks

- Output order. Standard output and standard error are read on two threads, so lines of the two can
  swap places. Recorded as a deviation if it is seen.
- Stop through Docker and Sail does not reach the container's PHP process.
- Herd on Windows is `herd.bat`. Detection of a Herd site uses a different folder on each platform.
  Both are checked on Windows in milestone 10, or recorded as not run.
- Output reading without P5 breaks when the package changes a sentence. The real-PHP job is the guard.
- Phase 2 makes Mortise run code. A command line from settings is never passed through a shell, and
  no part of a module name or option is joined into a string that a shell reads.

## 8. Milestone 17: package by feature

Milestone 17 is one OpenSpec change,
`phase-2-package-by-feature`, merged through three pull requests. It moves code and changes no behaviour.
Nothing a user sees changes.

Goal: the code is organised by feature, not by layer, and follows the domain ideas that fit a single-user
desktop app of this size.

### 8.1 What was there

- `mortise-core` was half by feature: `run/` and `catalog/` were folders and 24 files were flat.
  Four module cycles existed: `run` and `editor`, `git.rs` and `run::process::command`,
  `catalog` and `package`, and `catalog/cache.rs`, `run/recent.rs` with `run::history`.
- `catalog/` and `output.rs` each held two features, Actions and Generate.
- `mortise-app` was by layer: `*_view.rs` for screens, `draw.rs`, `dialog.rs` and `toast.rs` for components,
  one `strings.rs` of 3,385 lines, and one `app.rs` of 3,912 lines that wired every feature.
- Three helpers sat in the wrong module (`summary::alert_box`, `workspace::empty_state`, `app::Selection`),
  and `BoundaryAction` was what every tab returns.
- CI never built the core with `--no-default-features`, so a feature gate in the wrong place went unseen.

### 8.2 Domain model

The feature folders use the words of Mortise: App and module index, Boundary, Findings, Runtime, Runner,
Package data, Actions, Generate, Settings, Palette. The shared kernel is `php.rs`, starting a program and
`Version`.

- Each outside format keeps one reader: `package.rs` (the package's JSON), the output readers (artisan's text),
  `runtime/actuator.rs` (the actuator's JSON), `php.rs` (PHP source).
- `AppSnapshot` stays plain data that the screens show.
- Three aggregates guard a rule: Settings, Rules and Runs. Everything else is values and functions.
- Not used: repository traits, ports and adapters, domain events, buses, CQRS, `domain/`, `application/` and
  `infrastructure/` folders, and renaming code to the words on screen. Each would add code and no behaviour.

### 8.3 Target structure

Two crates stay. A crate per feature would turn about 87 `pub(crate)` items into public API.

- `mortise-core/src`: `lib.rs` (the flat list of public names), `php.rs`, `program.rs` (`command`,
  `split_words`), `package.rs`, `findings.rs`, `surface.rs`, `editor.rs`, `settings.rs`, `testing.rs`, and the
  folders `modules/`, `app/`, `boundary/`, `runtime/`, `run/`, `actions/`, `generate/`, `palette/`.
  `surface.rs` stays out of `boundary/` to avoid a cycle.
- A feature uses only features above it in this order: `php` and `program`, `modules`, `boundary`, `findings`,
  `run`, `package`, `actions`, `generate`, `app`, `surface`, `runtime` and `palette`, `editor`, `settings`.
- `mortise-app/src`: `strings.rs` (text that more than one feature shows), `ui/` (theme, fonts, draw, dialog,
  toast), `app/` (the window, and one wiring file per feature), and one folder per feature with its `view.rs`
  and its `strings.rs`. The wiring stays under `app/`, so no field of `MortiseApp` becomes `pub(crate)`.
- New rule in `AGENTS.md`, approved by the owner: all interface text is in a file named `strings.rs`.

### 8.4 Steps

Every commit passes the three cargo checks, adds no test and changes no assertion. Files move with `git mv`.
The snapshot images stay byte for byte the same.

| Step | Scope |
|---|---|
| P0 | Propose the OpenSpec change, with a small `code-layout` capability |
| C0 | CI: build the core with no default features and with each of `watch`, `actuator`, `os-secrets`, `run`, `testing` |
| C1 | `program.rs`: `command()` and `split_words()` |
| C2 | `modules/`: the seven reading files and `Target` |
| C3 | `app/`, `boundary/`, `runtime/`, and the gates `watch` and `actuator` |
| C4 | Split `catalog/` and `output.rs` into `actions/` and `generate/` |
| C5 | `palette/` |
| A1 | Move `alert_box`, `empty_state` and `EmptyMark` to `draw.rs` |
| A2 | `Selection` to the index. `BoundaryAction` becomes `WorkAction` |
| A3 | `ui/` |
| A4 | The feature folders, one `mod.rs` each |
| A5 | `app/mod.rs`, with chrome, rail, menu and workspace beside it |
| A6 | Split `app/mod.rs` into the wiring files, one commit each |
| S1 | Split `strings.rs` per feature, one commit per feature |
| D1 | Update `AGENTS.md`, `README.md`, `CONTRIBUTING.md`, `docs/deviations.md` and this plan |
| D2 | Archive the change |

The core track (C) and the app track (A) touch different files and run in parallel. Text and documents (S, D)
follow the app track.

### 8.5 Risks and what is left out

- Conflicts with work in flight: milestone 17 starts after milestone 15 is archived.
- A feature gate in the wrong place: the CI job of C0 catches it.
- Include paths: a missed `include_bytes!` or `include_str!` path fails the build.
- Visibility: nothing becomes more private, because `-D warnings` turns dead code into errors.
- A move that seems to need a test changed behaviour: stop and look.

Left out on purpose: more crates, a controller per feature in place of `impl MortiseApp`, splitting the long
frame function and the big views, one action enum per tab, gating all of `run/process.rs`, and merging the test
binaries of `mortise-app`.

### 8.6 What the build did

The change `phase-2-package-by-feature` records the design and, at the end of its `design.md`, how the build
differed from this plan. The main differences:

- A feature `mod.rs` declares its files and re-exports nothing, so each item has one path. The core's `lib.rs`
  still re-exports every public name.
- `ships()` and `generator_gives_data()` moved to `generate/catalog.rs`.
- The CI feature check builds the library and the binaries only (`--lib --bins`), with `-D warnings`, because
  the tests turn every feature on.
- `app/mod.rs` is 1,542 lines, a little over the aim, because `set_rules`, `save_rules` and shared helpers stay.
- 604 names were placed in `strings.rs` files, not 561: 96 shared, the rest in twelve feature files.
- The check of the snapshot images covers 140 files, not 122.
