# Mortise phase 1: build plan

Status: approved by the owner on 22 November 2026, with these decisions: MIT licence; the `git`
program for git data; on Windows the custom title bar by default with a setting for the platform's
bar; a flat module list without user-made groups in phase 1. Milestone 1 is built.

Phase 1 is read only. It needs no PHP on the machine and no change to `nwidart/laravel-modules`.
The design is in `docs/spec/` (version 3, no open questions). Screen ids such as S4.2 refer to it.

## 1. What was checked before writing this

Three checks were run against real sources, not memory. The scratch code is not part of the repository.

| Subject | How | Result |
|---|---|---|
| egui and eframe | Read the 0.37.0 source in the cargo registry, compiled a probe crate that uses every API this plan relies on, ran two headless `egui_kittest` tests on macOS | Most of what the spec needs exists. Seven points differ from the spec's notes; see `docs/deviations.md` |
| PHP parser | Wrote the extraction with two parsers and ran both on the two real apps and on their `vendor/` folders (16,000 files) | `mago-syntax` with `mago-names`: 0 failures, resolves names itself, 35,000 files a second |
| File formats | Read the package source at 13.3.0 and 12.0.6, both real apps, and laravel-actuator at tag 1.0.0 with its tests | Formats and defaults are recorded in section 4 |

Not checked: anything on Windows or Linux (this machine is macOS), how 10.5 to 13 px text looks on a 1x display, and a real actuator answer (neither real app has the actuator installed).

## 2. Workspace layout and crates

```
Cargo.toml                  workspace, edition 2024
rust-toolchain.toml         1.99.0, with rustfmt and clippy
crates/
  mortise-core/             library `mortise_core`. No UI dependency.
  mortise-app/              egui and eframe. Binary `mortise`.
assets/
  fonts/asap/               Asap 400, 500, 600, 700 static, with OFL.txt
  fonts/inconsolata/        Inconsolata 400, 500, 600 static, with OFL.txt
  icon/                     app icon
fixtures/                   fixture Laravel apps for tests
docs/
  spec/  spec-build/        copied from phase 0, unchanged
  plan/phase-1.md           this file
  deviations.md             where egui forces a difference from the spec
.github/workflows/ci.yml
LICENSE
```

Crate names. `mortise` is taken on crates.io. `mortise-core` and `mortise-app` are free (checked on 22 November 2026). A later command line tool takes `mortise-cli`.

Two crates, not more. The core crate holds everything that is not drawing: reading an app, statistics, the PHP analysis, links, rules, findings, the public surface scan, the actuator client, Mortise's own settings, the palette matcher and the live refresh. The app crate holds the window, the theme, the components and the screens. Three parts of the core are behind cargo features that are on by default, so the command line tool and the hosted service can leave them out: `watch` (file watcher), `actuator` (HTTP client) and `os-secrets` (credential store).

## 3. Dependencies

| Need | Choice | Reason |
|---|---|---|
| UI | `egui`, `eframe`, `egui_extras` 0.37.0, wgpu renderer | Decided. 0.37.0 is the current release. wgpu is eframe's default since 0.34. `egui_extras` gives the virtualised table. eframe's `default_fonts` feature is turned off so only Asap and Inconsolata ship |
| PHP parser | `mago-syntax`, `mago-names`, `mago-database`, `mago-span`, `mago-allocator`, all pinned `=1.59.0` | The only candidate that resolves names (aliases, grouped use, relative names). No failure on 16,000 real files, and its error recovery does not invent references. 12 times faster than tree-sitter-php, which also reported a false syntax error on valid PHP. Cost: 90 crates, and the API changes inside 1.x, so the version is pinned and all use sits in one module. `php-rs-parser` (11 crates) is the fallback |
| Git | The `git` program, read only: `git log`, `git rev-parse`, `git --no-optional-locks status` | The spec already assumes git is installed ("otherwise churn is absent"). Commit counts per folder are one `git log` call. No extra crates. The alternative is `gix` 0.90: no program needed, but about 150 more crates and hand-written path filtering. The owner can choose `gix` |
| File watcher | `notify` 9.0.0 with `notify-debouncer-full` 0.8.0 | The standard crate on all three platforms. Mortise watches named paths, not the whole app: the modules folder, `composer.lock`, `modules_statuses.json`, `config/modules.php`, `mortise.json`, `.env`, the clover file and `.git/HEAD`. `vendor/` and `node_modules/` are never watched |
| Folder walk | `ignore` 0.4 | Respects `.gitignore`, so `node_modules` inside a module is not counted as files |
| Parallel work | `rayon` | One parser arena per file works from rayon workers (measured) |
| JSON | `serde`, `serde_json` | `module.json`, the status file, `composer.lock`, `mortise.json`, actuator answers, Mortise's own settings. Its errors carry line and column, which the finding sentence needs |
| XML | `quick-xml` 0.42 | Streaming read of `clover.xml` |
| HTTP | `ureq` 3.5 with rustls | Four GET requests on a worker thread. No async runtime |
| Settings | One JSON file in the platform config folder, found with `directories` 6.0, written through a temporary file and a rename | One format in the whole app. Holds apps, environments (kind of credentials, user name, header name, never a secret), codes, groups, order, theme, text size, editor |
| Window size and place | eframe's `persistence` feature | Built in |
| Secrets | `keyring` 4.2.0: Keychain, Credential Manager, Secret Service | As the spec asks. When no store answers, the secret is kept in memory until Mortise closes and the form says so |
| Folder dialog | `rfd` 0.18 | The system dialog; the portal dialog on Linux |
| macOS menu bar | `muda` 0.21, macOS only | eframe offers no menu bar. Built in milestone 6 |
| Dates | `jiff` | "tomorrow 06:00 UTC", "2 days ago" |
| Errors | `thiserror` in the core | Typed errors that the interface turns into the spec's sentences |
| Tests | `insta`, `tempfile`, `egui_kittest` 0.37.0 | See section 7 |

Written by hand because the need is small: the palette matcher (letters in order), toasts, icons and state marks (painted strokes, no icon font), the graph layout.

Rust toolchain. `mago` 1.59.0 needs Rust 1.99, which is the version installed here; `mago` 1.60 and later need Rust 1.100. The plan pins Rust 1.99.0 and `mago` 1.59.0. Both move together later in one commit.

## 4. The model of the core crate

### What is read, and from where

| Fact | Source | Rule |
|---|---|---|
| Laravel app | `artisan`, `composer.json` | Missing: the sentence of S1.4 a |
| Laravel and package versions | `composer.lock`, in `packages` and `packages-dev` | A leading `v` is dropped. Dev versions (`dev-master`, `12.x-dev`) are shown as they are, with the branch alias when there is one |
| Modules folder, namespace, app folder, status file | `config/modules.php`, read with the PHP parser | String literals and `base_path('...')` are understood. Anything else falls back to the package default and is marked "assumed". No file means the defaults: `Modules`, `Modules`, `app/`, `modules_statuses.json` |
| Modules | `<modules folder>/*/module.json`, one level, as the package does | The name is the `name` key, not the folder. Invalid JSON is a finding, and the module is still listed by its folder |
| On or off | `modules_statuses.json` | Only the JSON value `true` for the exact name is on. A missing entry is off. If `activator` is not `file`, the state is unknown and the header says so |
| Module namespace | the module's `composer.json` `autoload.psr-4`, then the config, then `providers` in `module.json` | Covers the layout before v11 (no `app/` folder) |
| Kind of class | the folder of the file, mapped through `paths.generator` (app config, else defaults) | As the spec says: from the folder a file is in |
| Tests | test methods in `tests/`, found with the parser | PHPUnit methods and attributes, Pest `it` and `test` calls |
| Coverage | `clover.xml`: the path set in Settings, else the path in `phpunit.xml`, else four common places | Files are matched to modules by the path after the modules folder, because a report written in a container has other absolute paths |
| Git | `git` | Branch, commits in 30 days and last commit per module, files with changes that are not committed |
| Links | every PHP file of every module | A reference whose resolved name starts with another module's namespace: `use`, `new`, static calls, type hints, attributes, `::class`, `instanceof`, `catch`, `extends`, `implements`, trait use. Blade files are skipped |
| Runtime | actuator 1.0.0: the discovery links, `health`, `info`, `mappings`, `scheduledtasks` | Never `env`. `Accept: application/json` is always sent. A 503 from `health` is an answer |

### The types

```
AppSnapshot                    immutable, one per read
  identity     root, name, Laravel version, package version, actuator version,
               modules folder, namespace, what was assumed
  modules      Vec<Module>
  links        Vec<Link>       from, to, references (file, line, class, kind, internal or not)
  cycles       Vec<Cycle>      with the module that carries the finding
  rules        Rules           from mortise.json, or the defaults
  findings     Vec<Finding>    kind, module, sentence parts, where Enter opens
  checks       Vec<Check>      the checks that passed, for the Findings list
  coverage     where the report is and when it was written
  git          branch, or the reason git data is absent
  totals

Module
  name, code, folder, on or off, description, priority, namespace
  stats        files, lines of PHP, kinds, tests, coverage, commits, last commit
  finding state  problem with count, warning, good, or none when off

RuntimeSnapshot                one per environment
  connection   one of the nine states of S6.5
  health, info, routes by module, scheduled tasks
```

A snapshot is plain data with `serde` on it. The interface, a command line tool and a service all read the same value.

Mortise's own settings are a second value, `Settings`, with load and save. `mortise.json` is a third, `Rules`, and it is the only thing the core writes inside an app folder, and only when the interface asks after the user confirmed.

### How the app keeps the model fresh

The core offers `LiveApp`: it reads the app on a worker thread, publishes an `Arc<AppSnapshot>`, and calls a function when a new one is ready. The app crate passes a function that asks egui to repaint, and swaps the pointer at the start of the next frame. The interface never changes a snapshot.

1. The watcher reports changed paths, collected for 300 ms.
2. The worker reads again. Per file facts (lines, declared classes, references) are kept in a cache keyed by path, size and change time, so only changed files are parsed again. Links, rules and findings are computed again from the facts; that is cheap.
3. The new snapshot replaces the old one. Selection is kept by module name.

Ctrl R reads everything again without the cache. While a first read runs, the core reports "Reading 312 of 551 files" through the same channel.

The actuator is polled by a second worker: `health` every 15 s while the window is visible, the other three when the app is opened, every 5 minutes and on Ctrl R.

## 5. Milestones

Each ends in an app that runs. Each is reported with: what works, what was checked and how, deviations, what is not checked.

| # | Milestone | Core | Screens and components |
|---|---|---|---|
| 1 | Workbench and module list | Detect one app; versions; modules with on or off; file counts | Window and title bar T1 to T3; rail with one app; index rows of S3.1 (default, turned off); selection joint and peg; J and K; workspace header; five tabs with keys 1 to 5; S7.4 for Generate, Actions, Metrics; Slate light and dark; Asap and Inconsolata. The app folder is a command line argument. C1, C6, C7, C10, C11, C18 |
| 2 | Apps and live refresh | Settings file; add and remove; the failures of adding; watcher and `LiveApp` | S1.1, S1.2, S1.3 (folder and "Found in this folder"), S1.4 a and b with the two other failures, S2.1 with 1 and 10 apps; flow F1; Ctrl O, Ctrl R, Ctrl Alt 1 to 9. C2, C13, C17 |
| 3 | Statistics and Summary | Lines, kinds, tests, coverage, git | S4.1, S4.2 and S4.3 as far as files give them; S3.1 complete: coverage text, one line rows above 20 modules, filter, order, Ctrl B. C3, C8, C16 |
| 4 | Boundary | PHP analysis; links; circular use; rules and `mortise.json` | S5.1, S5.2, S5.3, S5.4; Dependencies box of S4.3; flow F5. C4, C5, C14 |
| 5 | Findings | The four file checks, the two rules, the coverage limit | S6.4; alerts on S4.1 to S4.3; state marks in the index; pip on the rail; problems first with "Needs attention"; flow F2. C9, C19 |
| 6 | Palette, Settings, window, keyboard | Palette matcher; settings for theme, text size, motion, editor | S12.1, S12.2, S8.1 (apps), S8.3, S8.4, S13.1; focus order K1; the full shortcut table of phase 1; "Open in editor"; macOS menu bar; fallback title bar T4. C12 |
| 7 | Runtime, optional | Actuator client; environments; secrets | S1.3 runtime section, S8.1 environments, S8.5, S6.5 a to i, runtime box, routes by module and scheduled tasks on S4.2, routes and tasks on S4.3, runtime line in the index header |
| 8 | Public surface scan | The scan and its proposal | S5.5, S5.6, S5.7 with Accept disabled and the reason in its tooltip; the proposal can be copied as text; flow F6 up to accepting |
| 9 | Hardening | | Accessibility pass (names, roles, focus ring on every control), image tests for every screen id in both themes, CI on three platforms, README, licences folder |

Not built in phase 1, as the spec says: S5.8 and S5.9 (need the package to store a surface), S7.1 to S7.3 (Metrics shows S7.4), S8.2 (PHP settings), S9, S10 and S11, the "Run PHP with" part of S1.3, runtime status per module, the Module column of scheduled tasks for anything but a class name.

## 6. Points the owner should decide or know

1. Licence. See section 8.
2. Git: the `git` program (recommended) or `gix`.
3. Windows 11 snap layouts. The flyout on the maximise button cannot be kept with a custom title bar: winit 0.30 does not answer the hit test Windows needs. Dragging to an edge and Win with the arrow keys still snap. The spec leaves this choice to the owner. Recommended: the custom bar by default, and a setting for the platform's own bar (T4), the same setting the spec already offers on Linux.
4. Module groups. The spec says Mortise keeps groups per app, and draws them in the index, but no screen says where a group is made or changed. Recommended for phase 1: a flat list plus the pinned "Needs attention" group, and no user-made groups until a screen for them is drawn.
5. Tabular numbers. The spec asks for them everywhere. Whether egui 0.37 can switch on the font's tabular figures is not known yet; it is checked in milestone 1. If it cannot, numbers in tables are right aligned in Inconsolata, or the default figures of Asap are used; the owner sees both before one is chosen.
6. The two real apps hold one module each, are not git repositories, and have no coverage report and no actuator. They can only confirm that reading does not fail. Everything else is tested on fixture apps, and the actuator against a small test server that answers with the JSON from the 1.0.0 tests. A real app with the actuator installed would close that gap.
7. Windows and Linux are built and tested by CI but cannot be looked at from this machine. CI itself runs only after the repository is on GitHub.

## 7. Tests and continuous integration

### Core crate

- Fixture apps in `fixtures/`, small and written by hand:
  - `shop-api`: the seven modules of the spec's drawings, with the circular link between Checkout and Shipping, the three reads of `Billing\Models\Invoice`, a turned off module variant and a `clover.xml`. The numbers in tests are the fixture's own, not the invented ones of the drawings.
  - `legacy`: the layout before v11, without the `app/` folder.
  - `broken`: invalid `module.json`, a namespace that does not match its folder, a status entry without a folder, a PHP file with a syntax error.
  - `not-laravel`, `no-package`, `no-lock`: the failures of adding an app.
- Generated at test time in a temporary folder: an app with 60 modules (the table of S5.4, one line rows), and git history (`git init` and commits), so that no test needs a repository inside this one.
- Each fixture is read into a snapshot, and the snapshot is compared with a stored one (`insta`). Unit tests cover the parts with rules: the status file, version strings, the config reader, module codes, circles, the matcher.
- The actuator client is tested against a local test server built on `std::net`, for each of the nine states of S6.5.
- A smoke test reads the two real apps. It is ignored by default, takes the path from an environment variable, and only reads.
- A test asserts that reading a fixture changes no file in it.

### Interface

- `egui_kittest` runs the real app headless. Tests find controls by role and name, press keys and click: J and K, the tabs, the filter, the palette, the order of Esc, the focus order. The harness fails when a control has no accessible name, which is kept on.
- Image tests: each screen id, light and dark, at 1280 by 800 and at 960 by 600. The stored images are made on Linux with a software renderer, and compared only there, because images differ between drivers.
- By eye: each screen rendered to a PNG and compared with the spec drawing in the same theme. This is what each milestone report means by "looked at".

### Continuous integration

GitHub Actions, on `ubuntu-latest`, `macos-latest` and `windows-latest`: `cargo fmt --check`, `cargo clippy` with warnings as errors, `cargo test` for the workspace, a release build of the binary. Image tests run on Linux only. The workflow is committed in milestone 1 and first runs when the owner pushes.

## 8. Licence

Proposed: MIT, the same as `nwidart/laravel-modules`. One `LICENSE` file, and it is the licence people who use the package already know.

- Every dependency in section 3 allows it (MIT, Apache-2.0, CC0, BSD).
- The fonts keep their own licence: both `OFL.txt` files ship in `assets/fonts/` and in a `licenses` folder next to the binary, and Settings names them.
- The other common choice for Rust is "MIT OR Apache-2.0", which adds a patent grant. It is equally possible.
- MIT lets anyone run the core as a service. If the later cloud plan should be protected from that, the licence is the place to do it, and it is easier to choose now than to change later.

The owner confirmed MIT on 22 November 2026.
