# Design

This is the research behind the change. It was measured on `95c00dc`, before milestone 15 merged. The section "Measured again on `main`" at the end holds where it differs from the text above it. Section 4 gives the steps; the tasks in `tasks.md` follow it.

---

## 1. Current state

### 1.1 Totals

| Crate | Source | Tests in `tests/` |
|---|---|---|
| `mortise-core` | 37 files, 13,314 lines | 7 files, 3,167 lines |
| `mortise-app` | 28 files, 25,927 lines | 17 files, 8,820 lines, 122 snapshot images |

### 1.2 `mortise-core` (`crates/mortise-core/src`)

`lib.rs` (102 lines) declares 26 private modules and re-exports every public name at the crate root.
`mortise-app` therefore sees a flat list of names, not module paths. Three paths are public as modules:
`mortise_core::ansi`, `mortise_core::history`, `mortise_core::catalog_cache`, plus `mortise_core::testing`.

| Module | Lines | Owns | Depends on (inside the crate) |
|---|---|---|---|
| `php.rs` | 946 | All `mago-*` use: declared classes, references, config values, created tables | none |
| `app.rs` | 536 | `AppSnapshot`, `read_app*`, `ReadOptions`, `ReadError` | clover, code, composer, findings, git, kind, layout, links, module, package, rules |
| `module.rs` | 618 | `Module`, `ModuleState`, `Manifest`, `ReadCache`, `Progress`, the file walk | clover, git, kind, layout, php |
| `layout.rs` | 240 | `Layout` read from `config/modules.php` | php |
| `kind.rs` | 193 | The kind of a file from its folder | php |
| `composer.rs` | 173 | `Version`, `composer.lock` | none |
| `clover.rs` | 253 | `Coverage` from a Clover report | none |
| `git.rs` | 238 | Branch, churn, uncommitted counts | `run::process::command` |
| `code.rs` | 229 | Stamp codes and monograms | none |
| `live.rs` | 414 | `LiveApp`: read on a worker, read again on change (feature `watch`) | app, module |
| `links.rs` | 630 | `Link`, `Reference`, `Cycle`, `columns` | php, rules |
| `rules.rs` | 317 | `Rules`, load and save of `mortise.json` | none |
| `surface.rs` | 559 | `SurfaceScan`, `Proposal` | app, kind, layout, module, php |
| `findings.rs` | 328 | `Finding`, `Check`, `Severity`, `Findings` | links, module, rules |
| `actuator.rs` | 510 | The HTTP client (feature `actuator`) | environment |
| `runtime.rs` | 251 | `LiveRuntime`, `routes_of`, `tasks_of` (feature `actuator`) | actuator, app, environment, module |
| `environment.rs` | 84 | `Environment`, `Credentials` | none |
| `secrets.rs` | 147 | `Secrets`: credential store or memory (feature `os-secrets` inside) | none |
| `run/mod.rs` | 501 | `RunnerKind`, `PhpSettings`, `CommandLine`, `test_php` | `editor::split_words` |
| `run/process.rs` | 1,020 | `Runs`, `Run`, stop, time limit; also `command()`, used without `run` | run |
| `run/detect.rs` | 195 | Herd, Sail, Docker from files | none |
| `run/ansi.rs` | 155 | SGR codes to four styles | none |
| `run/history.rs` | 237 | The last 50 runs per app on disk, `app_key`, `data_dir` | none |
| `run/read.rs` | 100 | `Read`: a package command run for its data | run |
| `run/recent.rs` | 197 | `recent_runs` for the palette (S12.2 a) | run::history, `Action`, `Catalog`, `AppSnapshot` |
| `catalog/actions.rs` | 879 | `Action`, `Target`, `Plan`, `plan`, `refuses`, `asks_again` | none |
| `catalog/generators.rs` | 1,017 | `Catalog`, `Generator`, `Filled`, name rules, `v13_3.json`, `v13_4.json` | composer, `package::gives_data` |
| `catalog/cache.rs` | 110 | The catalog read from an app, kept in the data folder | composer, `run::history::app_key` |
| `output.rs` | 452 | Reads run output: failed migration, refusal, counts, generated files (feature `run`) | package, `run::ansi` |
| `package.rs` | 514 | The JSON documents of laravel-modules 13.4, `PackageData` | `catalog::Target`, composer, module, php |
| `editor.rs` | 147 | Opening a file in the editor | `run::process::command` |
| `settings.rs` | 445 | `Settings`, `AppEntry`, `General`, load and save | app, code, editor, environment, findings, run |
| `matcher.rs` | 92 | `fuzzy` for the palette | none |
| `testing.rs` | 461 | Stub actuator, fake artisan (feature `testing`) | none |

Findings about the core:

- The core is half by feature already: `run/` and `catalog/` are folders, the other 24 files are flat.
- Four module cycles exist today:
  - `run/mod.rs` uses `editor::split_words`; `editor.rs` uses `run::process::command`.
  - `git.rs` (reading) uses `run::process::command`. Reading an app reaches into the runner for a helper.
  - `catalog/generators.rs` uses `package::gives_data`; `package.rs` uses `catalog::Target`.
  - `catalog/cache.rs` uses `run::history::app_key`; `run/recent.rs` uses `Action` and `Catalog`.
- `catalog/` groups two features (Actions and Generate) by what they are (data), not by feature.
- `output.rs` holds two features in one file: migration lines (Actions) and generated files (Generate).
- `run/recent.rs` is palette data that lives in the runner tree.
- `run/process.rs` carries 44 `#[cfg(feature = "run")]` lines because one function, `command()`, must build without `run`.
- CI runs no build with `--no-default-features` (`.github/workflows/ci.yml` has no such line). A feature-flag mistake is not caught there.

### 1.3 `mortise-app` (`crates/mortise-app/src`)

| Module | Lines | Owns | Depends on (inside the crate) |
|---|---|---|---|
| `app.rs` | 3,396 | `MortiseApp`, `OpenApp`, `Config`, `Selection`, the frame, the keys, and the wiring of every feature | every other module |
| `strings.rs` | 3,106 | 561 constants and functions of interface text, 9 tests | none |
| `draw.rs` | 1,507 | Components: text, icons, stamp, button, checkbox, text field, list navigation | theme |
| `theme.rs` | 565 | Tokens: palette, type, sizes | none |
| `fonts.rs` | 49 | Font bytes | theme |
| `dialog.rs` | 290 | Dialog (C14) | draw, theme, strings, `summary::alert_box` |
| `toast.rs` | 178 | Toast (C13) | draw, theme, strings |
| `chrome.rs` | 292 | Title bar | draw, theme, strings |
| `rail.rs` | 309 | The app rail | draw, theme, strings |
| `menu.rs` | 143 | The macOS menu bar | strings |
| `workspace.rs` | 498 | Header, the five tabs, `Tab`, `empty_state` | every tab view, `app::Selection` |
| `index.rs` | 709 | S3.1 | `app::Selection`, `runtime_view::Brief` |
| `add.rs` | 830 | S1.2 to S1.4 | environment_form, php_settings |
| `summary.rs` | 1,529 | S4.1 to S4.3, and `alert_box` | boundary (`BoundaryAction`), runtime_view |
| `boundary.rs` | 1,672 | S5.1 to S5.4, and `BoundaryAction` | surface_view, and types of actions_view, generate_view, runtime_view |
| `surface_view.rs` | 619 | S5.5 to S5.7 | boundary, `workspace::empty_state` |
| `findings_view.rs` | 322 | The Findings list | boundary |
| `runtime_view.rs` | 963 | S6.5, routes, tasks, `Brief` | none |
| `environment_form.rs` | 514 | The environment form | `runtime_view::Brief` |
| `runner_view.rs` | 1,626 | S11.1, `AppRuns`, `line_now` | none |
| `php_settings.rs` | 354 | S8.2 and "Run PHP with" | none |
| `actions_view.rs` | 1,118 | S10.1, S10.2 | boundary, `runner_view::line_now`, `summary::alert_box` |
| `generate_view.rs` | 2,401 | S9.1 to S9.6 | boundary, `runner_view::line_now`, `summary::alert_box` |
| `settings_view.rs` | 2,004 | S8.1 to S8.4 | environment_form, php_settings, runtime_view |
| `palette.rs` | 872 | S12.1, S12.2 | none |

Every module also uses `draw`, `theme` and `strings`.

Findings about the app:

- The crate is organised by layer: screens (`*_view.rs`), components (`draw.rs`, `dialog.rs`, `toast.rs`),
  text (`strings.rs`), wiring (`app.rs`).
- `app.rs` is the wiring of every feature in one `impl MortiseApp`. By line range:
  runtime 501 to 581, apps and the add panel 582 to 786 and 2251 to 2311, menu and settings 787 to 957,
  palette 969 to 1067, reads and runs 1068 to 1309, actions 1310 to 1621, generate 1622 to 1881 and the
  types at 142 to 260, rules 1882 to 1917, keys 1918 to 2250, the frame (`ui`) 2312 to 3029.
- `strings.rs` is one file for all features. 476 of its 561 names are used by exactly one other file.
  72 are used by two or more. 13 are used only inside `strings.rs`. Its sections are out of order: Actions
  text sits at line 13 and again at 2116 to 2575, tests sit at 1055, 1661 and 3091.
- Three helpers live in the wrong module and create sideways dependencies:
  - `summary::alert_box` is used by `actions_view.rs`, `generate_view.rs` and `dialog.rs`.
  - `workspace::empty_state` and `EmptyMark` are used by `surface_view.rs`.
  - `app::Selection` is used by `index.rs` and `workspace.rs`, so two views depend on the app.
- `boundary::BoundaryAction` is not about Boundary. It is what any tab asks the app to do. Its variants name
  `runtime_view::RuntimeAction`, `actions_view::Act` and `generate_view::GenerateAsk`. Seven files use it.
- `tests/` is already by feature: `actions.rs`, `apps.rs`, `boundary.rs`, `environments.rs`, `findings.rs`,
  `generate.rs`, `palette.rs`, `runner.rs`, `runtime.rs`, `settings.rs`, `surface.rs`, `workbench.rs`,
  `keyboard.rs`. Three files cut across: `accessibility.rs`, `images.rs`, `screens.rs`.
- The tests reach the crate by 7 module paths: `app` (18 uses), `workspace` (8), `theme` (3), `chrome` (3),
  `strings`, `runner_view`, `palette` (1 each).

### 1.4 One feature in many places

| Feature | Core | App | Tests |
|---|---|---|---|
| Generate | `catalog/generators.rs`, `catalog/cache.rs`, `catalog/v13_*.json`, half of `output.rs`, `GeneratorReport` in `package.rs` | `generate_view.rs`, `app.rs` 142 to 260 and 1622 to 1881, `strings.rs` 2577 to 3090, rows in `palette.rs` | `tests/generate.rs`, `images.rs`, `accessibility.rs` |
| Actions | `catalog/actions.rs`, half of `output.rs`, migration types in `package.rs`, `git.rs` (uncommitted) | `actions_view.rs`, `app.rs` 1310 to 1621 and 3150 to 3190, `dialog.rs`, `strings.rs` 13 and 2116 to 2575 | `tests/actions.rs`, `images.rs` |
| Runner | `run/*` | `runner_view.rs`, `php_settings.rs`, `app.rs` 1068 to 1309, `strings.rs` 1536 to 1660 and 1949 to 2114 | `tests/runner.rs`, core `tests/run.rs`, `tests/real_php.rs` |
| Runtime | `actuator.rs`, `runtime.rs`, `environment.rs`, `secrets.rs` | `runtime_view.rs`, `environment_form.rs`, `app.rs` 501 to 581, `strings.rs` 1157 to 1535 | `tests/runtime.rs`, `tests/environments.rs`, core `tests/actuator.rs` |

---

## 2. Domain model

The words come from `docs/plan/phase-2.md`, `openspec/specs/*` and `strings.rs`.

| Context | Words | Entities, value objects, aggregates | Domain services | Infrastructure |
|---|---|---|---|---|
| App and module index | app, module, on, off, layout, kind, stamp, code, monogram, coverage, churn | `AppSnapshot` is a read model: built whole on each read, never changed in place. `Module` is an entity inside it (identity: its name). Values: `Layout`, `Version`, `Coverage`, `Churn`, `KindCount`, `Manifest` | `read_app*`, `find_modules`, `propose_codes` | File system, the PHP parser (`php.rs`), the `git` program, the file watcher (`LiveApp`) |
| Boundary | link, reference, circle (`Cycle` in code), rule, public surface, proposal | `Rules` is a small aggregate: the one thing Mortise writes in an app. Values: `Link`, `Reference`, `Cycle`, `SurfaceEntry`, `Proposal` | `links::build`, `Link::is_problem`, `SurfaceScan` | `mortise.json` through a temporary file (deviation S28), the PHP parser |
| Findings | finding, problem, warning, check, good | Values only: `Finding`, `Check`, `Detail`, `Severity`, `FindingState` | `findings::find` | none |
| Runtime | environment, credentials, secret, health, up, down, out of service, route, scheduled task | `Environment` is an entity inside `AppEntry`. Values: `Answer`, `Status`, `Route`, `Task`, `RuntimeState` | `module_of`, `routes_of`, `tasks_of` | HTTP (`ureq`), the credential store (`keyring`), a worker thread (`LiveRuntime`) |
| Runner | runner, Herd, Sail, Docker, command line, run, read, stop, time limit, history, drawer | `Runs` is an aggregate: it holds the rule "one command per app". `Run` is an entity (id). `History` is an entity list with a limit of 50. Values: `RunnerKind`, `PhpSettings`, `CommandLine`, `RunEnd`, `Entry` | `detect`, `test_php`, `stop_asks_first` | Processes and process groups (`rustix`), the data folder |
| Package data | document, dry run, report, waiting, batch | Values: `GeneratorReport`, `MigrationStatus`, `TestReport`, `LangReport`. `PackageData` is session state inside the snapshot | `document`, `with_json`, `with_dry_run`, `gives_data` | None of its own. It reads lines a run gave |
| Actions | action, target, plan, ask, refuse, destructive, production, typed name | Values: `Action`, `Target`, `Options`, `Plan`, `Ask` | `plan`, `refuses`, `asks_again`, `typed_name_matches`, the output readers | none |
| Generate | generator, catalog, group, argument, option, form, file list, replace | `Catalog` is a value per package version. Values: `Generator`, `Filled`, `GeneratedFile`, `FileOutcome` | `Catalog::from_json`, `shipped_mark`, name rules, `generated_files`, `reported_files` | The catalog cache in the data folder |
| Settings | settings, app entry, theme, motion, index rows, editor | `Settings` is the one true aggregate with a root: it owns `AppEntry` (with its environments and PHP settings) and `General`, and is saved whole | `Settings::add`, `remove`, `propose_monogram` | `settings.json`, the editor program |
| Palette | command, recent, match | Values: `RecentRun`, `Ran` | `fuzzy`, `recent_runs` | none |

Shared kernel: `php.rs` (the parser), starting a program (`command()`, `split_words`), `composer::Version`.

Patterns that fit, and are already there under other names:

- **Anti-corruption layer.** `package.rs`, `output.rs`, `actuator.rs` and `php.rs` each turn something from
  outside (JSON of the package, text of artisan, JSON of the actuator, PHP source) into Mortise's own types.
  Keep each as the only place that knows the outside format.
- **Read model.** `AppSnapshot` is plain data that the interface shows. Keep it immutable.
- **Value objects.** Most core types are already plain values with `Clone`, `PartialEq` and serde.

Patterns that do not fit. Do not add them:

| Pattern | Why not here |
|---|---|
| Repository traits | `Settings::load` and `save`, `Rules::load` and `save`, `History::open` and `record` are plain functions on a path. Tests pass a temporary folder. A trait would have one implementation. |
| Ports and adapters for process, HTTP, file system | The tests use a real process (`fake-artisan`) and a real local HTTP server (`StubActuator`). That tests more than a mock would. A trait would exist only to be mocked. |
| Domain events, an event bus | egui draws every frame and polls. `LiveApp`, `LiveRuntime` and `Run` already hand results over a channel or a shared state. Views already return a plain action enum. |
| Command and query buses, CQRS | One user, one process, no concurrency between writers. |
| `domain/`, `application/`, `infrastructure/` folders in each feature | Feature files are 100 to 1,000 lines. Three layers per feature would give folders of one small file each. |
| Factories, specification objects | A function or a constructor does the job (`plan`, `Catalog::from_json`). |
| An aggregate class per context | Only `Settings`, `Rules` and `Runs` guard an invariant. The rest is values and functions. Forcing roots on them adds types with no rule to hold. |
| Renaming code to the interface words | `Cycle` is "circle" on screen. A rename touches serde names and many files for no behaviour. Leave it. |

---

## 3. Target structure

### 3.1 Decisions

| Question | Decision | Reason |
|---|---|---|
| More crates? | No. Keep two. | The one boundary that matters (no interface code in the core) is already a crate boundary. A crate per feature would turn about 87 `pub(crate)` items (counted on `main`) into public API, would need the features `watch`, `actuator`, `os-secrets`, `run`, `testing` forwarded across crates, and would need a second and third fake artisan binary. Inferred: the slow part of the build is linking 17 test binaries of `mortise-app` against eframe and wgpu; more crates do not change that. |
| Does `mortise-app` mirror the core? | Yes, by name, without a `features/` wrapper. | `mortise_core` has `generate/`, `mortise-app` has `generate/`. Thirteen folders at the top of `src` are few enough to read. |
| Where does the wiring go? | It stays in `app/`, split into one file per feature. | `impl MortiseApp` blocks can live in child modules of `app` and still reach its private fields. Moving them into the feature folders would need `pub(crate)` fields and would make each feature depend on the app. |
| Does `strings.rs` split? | Yes: one `strings.rs` per feature folder, and `src/strings.rs` for text that more than one feature shows. | 476 of 561 names have one user. The single file is a conflict point for every feature change. |
| Does the core's public surface change? | No. `lib.rs` keeps every name and the paths `ansi`, `history`, `catalog_cache`, `testing`. | `mortise-app` and the tests then need no change for any core step. |
| Does `package.rs` split by feature? | No. | It is one reader for one outside format, and `PackageData` merges across its parts. 514 lines. |

New wording for the `AGENTS.md` rule and Layout row:

> All interface text is in a file named `strings.rs`: one in each feature folder of `mortise-app/src`, and
> `mortise-app/src/strings.rs` for text that more than one feature shows. No other file holds interface text.

### 3.2 `mortise-core/src`

```
lib.rs            the flat list of public names, unchanged
php.rs            the PHP parser, unchanged (all mago-* use)
program.rs        NEW, small: command() from run/process.rs, split_words() from editor.rs
package.rs        unchanged
findings.rs       unchanged
surface.rs        unchanged
editor.rs         unchanged but for split_words
settings.rs       unchanged
testing.rs        unchanged (feature testing)
modules/          module.rs, layout.rs, kind.rs, composer.rs, clover.rs, git.rs, code.rs, target.rs (Target)
app/              mod.rs (was app.rs), live.rs (feature watch)
boundary/         links.rs, rules.rs
runtime/          actuator.rs and live.rs (was runtime.rs) (feature actuator), environment.rs, secrets.rs
run/              mod.rs, detect.rs, process.rs, read.rs, ansi.rs, history.rs
actions/          plan.rs (was catalog/actions.rs), output.rs (the migration half of output.rs, feature run)
generate/         catalog.rs (was catalog/generators.rs), v13_3.json, v13_4.json,
                  cache.rs (feature run), output.rs (the file half of output.rs, feature run)
palette/          matcher.rs, recent.rs (was run/recent.rs, feature run)
```

Each folder has a `mod.rs` that declares its files and re-exports what `lib.rs` and other features use.
Items stay `pub(crate)` where they are today.

`surface.rs` stays outside `boundary/` on purpose: it reads an `AppSnapshot`, and the snapshot is built from
`boundary/links.rs`. Inside `boundary/` it would make a cycle.

Dependency direction, lowest first. A feature uses only features above it in this list:

1. `php`, `program`
2. `modules` (uses php, program)
3. `boundary` (php)
4. `findings` (modules, boundary)
5. `run` (program)
6. `package` (modules, php)
7. `actions` (modules for `Target`, run for `ansi::plain`)
8. `generate` (modules, package, run)
9. `app` (modules, boundary, findings, package)
10. `surface`, `runtime`, `palette` (app, modules; palette also actions, generate, run)
11. `editor` (program), then `settings` (app, modules, editor, findings, run, runtime)

This removes the four cycles of section 1.2. Moving `Target` to `modules/target.rs` is what breaks the
`catalog` and `package` cycle.

Feature gates stay on the same code:

| Feature | Code after the move |
|---|---|
| `actuator` | `runtime/actuator.rs`, `runtime/live.rs` |
| `watch` | `app/live.rs` |
| `os-secrets` | inside `runtime/secrets.rs`, as today |
| `run` | `run/*` as today, `actions/output.rs`, `generate/output.rs`, `generate/cache.rs`, `palette/recent.rs` |
| `testing` | `testing.rs`, `bin/fake-artisan.rs` |

### 3.3 `mortise-app/src`

```
lib.rs
main.rs
bin/app-fake-artisan.rs
strings.rs        text that more than one feature shows, and the shared formatters (counts, "ago", lists)
ui/               theme.rs, fonts.rs, draw.rs, dialog.rs, toast.rs
app/              the window and the wiring
  mod.rs          MortiseApp, OpenApp, Config, new(), the frame (ui), run()
  keys.rs         the keys
  apps.rs         open, add, select, move, remove an app
  settings.rs     general settings, the menu actions
  palette.rs      what a palette command does
  runs.rs         start, stop, run again, reads of package data, the drawer actions
  actions.rs      act, the dialogs, refusals, the end of an action
  generate.rs     AppCatalog, Generating, SelectNew, generate, the end of a generator
  runtime.rs      the chosen environment
  chrome.rs  rail.rs  menu.rs  workspace.rs  strings.rs
apps/             view.rs (was add.rs), strings.rs
index/            view.rs (was index.rs, with Selection), strings.rs
summary/          view.rs, strings.rs
boundary/         view.rs, surface.rs (was surface_view.rs), strings.rs
findings/         view.rs, strings.rs
runtime/          view.rs, environment_form.rs, strings.rs
runner/           view.rs (the drawer), php_settings.rs, strings.rs
actions/          view.rs, strings.rs
generate/         view.rs, strings.rs
settings/         view.rs, strings.rs
palette/          view.rs, strings.rs
```

Public surface of a feature folder: its `mod.rs` re-exports, by name, the state struct, the inputs struct,
the action or ask enum and the paint function that `app/` uses today (for `generate/`: `GenerateState`,
`GenerateInputs`, `GenerateAsk`, `GenerateRun`, `GenerateResult`, `CatalogRead`, `generate`, `title`,
`file_path`, `read_catalog_words`). Nothing new becomes public.

Dependency direction:

1. `ui`, `strings` (use `mortise_core` only)
2. `runtime`, `runner`, `palette`, `boundary`, `findings` (ui)
3. `summary`, `index` (runtime), `actions`, `generate` (runner), `apps`, `settings` (runtime, runner)
4. `app` (everything)

One exception, stated and kept: the action enum that every tab returns (today `BoundaryAction`) moves to
`app/workspace.rs` as `WorkAction`. The tabs name it, and it names `RuntimeAction`, `Act` and `GenerateAsk`.
Removing that would need one action enum per tab and a mapping in the workspace. That changes every view's
signature for no gain. Left out.

What stays as it is: `draw.rs` (1,507 lines) is one file of components and is not split. The frame function
`ui` (718 lines) stays whole in `app/mod.rs`. The view files keep their size.

---

## 4. Migration plan

Ground rules for every step:

- Starts after milestone 15 (`phase-2-package-data`) is archived. No other feature branch is open while the refactor runs.
- No behaviour change, no new test, no changed assertion. Unit tests move with the code they test.
- Files move with `git mv`. A commit that moves a file changes only `mod`, `use` and path lines in it.
- After each commit: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` on macOS (images included), and
  `git status --porcelain crates/mortise-app/tests/snapshots` prints nothing. `UPDATE_SNAPSHOTS` is never set.
- After each core commit also, because CI does not: `cargo check -p mortise-core --no-default-features`, then
  once with each of `--features watch`, `actuator`, `os-secrets`, `run`, `testing`.
- Commit subjects: `refactor(core): ...` and `refactor(app): ...` (the CI pattern allows `refactor`).
- Windows and Linux: checked by CI only. Say so in each pull request.

Three pull requests. Track 1 (core) and track 2 (app) touch disjoint files and can run in parallel in two
worktrees. Inside a track the steps are sequential: each edits `lib.rs` and many `use` lines.

### Track 1: core (PR 1)

| Step | Scope | Accept when | Risk | Model |
|---|---|---|---|---|
| C1 | New `program.rs`. Move `command()` out of `run/process.rs` and `split_words()` out of `editor.rs`. Update `git.rs`, `editor.rs`, `run/mod.rs`. | Checks green. `git.rs` and `editor.rs` no longer name `run`. `run` no longer names `editor`. | Low. `command()` must stay ungated. | Sonnet |
| C2 | `modules/`: move `module.rs`, `layout.rs`, `kind.rs`, `composer.rs`, `clover.rs`, `git.rs`, `code.rs`. Move `Target` from `catalog/actions.rs` to `modules/target.rs`. | Checks green. `lib.rs` exports the same names. | Low | Sonnet |
| C3 | `app/` (`app.rs`, `live.rs`), `boundary/` (`links.rs`, `rules.rs`), `runtime/` (`actuator.rs`, `runtime.rs` as `live.rs`, `environment.rs`, `secrets.rs`). | Checks green, the feature matrix builds. | Medium: the `actuator` and `watch` gates move from `lib.rs` to two `mod.rs` files. | Sonnet |
| C4 | Dissolve `catalog/` and `output.rs` into `actions/` and `generate/` as in 3.2. Split the tests of `output.rs` with the functions they test. `include_str!("v13_3.json")` stays valid because the JSON files move with `catalog.rs`. Keep `pub use generate::cache as catalog_cache` in `lib.rs`. | Checks green, the feature matrix builds. The count of `#[test]` in the crate is unchanged. | Medium: one file is split by judgment, and the `run` gate must cover both halves. | Opus |
| C5 | `palette/`: move `matcher.rs` and `run/recent.rs`. | Checks green. `run/` names neither `Action` nor `Catalog`. | Low | Sonnet |

### Track 2: app (PR 2)

| Step | Scope | Accept when | Risk | Model |
|---|---|---|---|---|
| A1 | Move `alert_box` from `summary.rs` to `draw.rs`. Move `EmptyMark` and `empty_state` from `workspace.rs` to `draw.rs`. | Checks green. `dialog.rs`, `actions_view.rs`, `generate_view.rs` no longer name `summary`. `surface_view.rs` no longer names `workspace`. | Low | Sonnet |
| A2 | Move `Selection` from `app.rs` to `index.rs`. Rename `BoundaryAction` to `WorkAction` and move it to `workspace.rs`. Update the test imports. | Checks green. `index.rs` no longer names `app`. | Low: a rename in 8 files. | Sonnet |
| A3 | `ui/`: move `theme.rs`, `fonts.rs`, `draw.rs`, `dialog.rs`, `toast.rs`. Add one `../` to each `include_bytes!` and `include_str!` in `fonts.rs` and `theme.rs`. | Checks green. The token test in `theme.rs` still reads `docs/spec/tokens.html`. | Low. A wrong include path fails the build, not at run time. | Sonnet |
| A4 | Feature folders: move the 14 view files as in 3.3, add each `mod.rs`. | Checks green. Tests import the new paths. | Low, wide: most `use crate::` lines change. | Sonnet |
| A5 | `app/`: move `app.rs` to `app/mod.rs`, and `chrome.rs`, `rail.rs`, `menu.rs`, `workspace.rs` beside it. Fix the two `include_bytes!` of the icon. Update `main.rs` and the tests (`workspace::Tab`, `chrome::Chrome`). | Checks green. | Low | Sonnet |
| A6 | Split `app/mod.rs`: `keys.rs` first, then `generate.rs`, `actions.rs`, `runs.rs`, then `apps.rs`, `settings.rs`, `palette.rs`, `runtime.rs`. One commit each. Method bodies do not change. A moved method becomes `pub(super)` only where another file calls it. The 10 `cfg` lines move with their methods. | Checks green after each commit. `git diff --color-moved` shows moved blocks, `use` lines and visibility only. `app/mod.rs` is under about 1,500 lines. | Medium: helper types and free functions must land with their one user; a wrong `cfg(target_os = "macos")` breaks only another platform. | Opus |

### Text and documents (PR 3, after PR 2)

| Step | Scope | Accept when | Risk | Model |
|---|---|---|---|---|
| S1 | Split `strings.rs`. A name with one user moves to that feature's `strings.rs`. A name used by `app/` only goes to the feature its wiring file belongs to, or to `app/strings.rs`. A name with users in two features stays in `src/strings.rs`, with the shared formatters. Feature files import `super::strings`, and name shared text as `crate::strings::X`. The 9 tests move with their functions. One commit per feature. | Checks green. The count of `pub const` and `pub fn` over all `strings.rs` files is 561. No name is defined twice. Every "not in the spec" mark is kept. | Medium: about 560 names placed by judgment; a wrong home is harmless but untidy. | Opus |
| D1 | `AGENTS.md`: the Layout table, the strings rule (3.1), the path of `mortise_core::php` if written as a file. `README.md`, `CONTRIBUTING.md` and `docs/deviations.md` (29 mentions of `.rs` files): update each path that moved. `docs/plan/phase-2.md`: add milestone 17 to the table; leave section 3 as history. | `grep -rn '_view\.rs\|catalog/' AGENTS.md README.md CONTRIBUTING.md docs/deviations.md` finds nothing stale. | Low | Sonnet |
| D2 | OpenSpec. `openspec/specs/*` name no source file (checked with grep), so no requirement changes. Propose the change `phase-2-package-by-feature` first (before C1), and archive it last. If `openspec validate` refuses a change with no spec delta, add one small capability, `code-layout`, with the rules of 3.2 and 3.3 that can be checked: no interface dependency in the core, where interface text lives, the direction lists. Archived changes are not edited. | `openspec validate` passes. | Low. Not checked: whether a change without a delta validates. | Sonnet |

### Order

```
propose (D2)  ->  C1 -> C2 -> C3 -> C4 -> C5            (PR 1)
              ->  A1 -> A2 -> A3 -> A4 -> A5 -> A6      (PR 2)
                                                 -> S1 -> D1 -> archive (PR 3)
```

No step needs Fable. The two calls with a real choice (the `WorkAction` exception, wiring stays in `app/`)
are made in section 3.

### Tests

- Unit tests stay beside the code. A split file takes its tests with it (`output.rs`, `strings.rs`).
- Files in `tests/` do not move and are not merged: they are already by feature, and each is a rule of its
  layer. Only their `use` lines change.
- Net test lines: zero. The `AGENTS.md` advisory of 400 lines is not touched.
- Images: the 122 files in `crates/mortise-app/tests/snapshots` stay byte for byte. That is the proof of no
  visual change, on macOS locally and in the macOS job of CI.

---

## 5. Risks, and what to leave out

| Risk | What to do |
|---|---|
| Conflicts with work in flight | Start only after milestone 15 (`phase-2-package-data`) is archived. Keep each PR open for days, not weeks. About 20 stale `step/*` and `build/*` worktrees exist; delete them first so none is rebased over a move by mistake. |
| `git blame` churn | Use `git mv`, keep move commits free of edits, and merge the PRs with a merge commit, as the repo does now (not a squash). `git blame -C` and `git log --follow` then cross the moves. `app.rs` and `strings.rs` are split, so blame needs `-C` there. |
| Public API breaks | The core's public names do not change. `mortise_app` is used by its own tests and `main.rs` only; update about 35 import lines in the same commit as each move. Add no compatibility re-exports. |
| Feature-flag mistakes | CI builds default features only. Run the matrix of section 4 by hand after C1 to C5. Consider adding it to CI as a later, separate change. |
| Include paths | `include_bytes!` and `include_str!` are relative to the file: `ui/fonts.rs`, `ui/theme.rs`, `app/mod.rs` each need one more `../`. The build fails if one is missed. |
| Visibility drift | Do not make `pub mod` private or tighten `pub` to `pub(crate)` in this work. It would raise dead-code warnings under `-D warnings` and is a different change. |
| Test duplication | Add no test. If a move seems to need one, the move changed behaviour: stop and look. |
| Shadowed text | Shared text is named `crate::strings::X`, with no glob re-export, so a feature name cannot hide a shared one. |

Left out, because the cost is higher than the gain:

- More crates (3.1).
- Controllers per feature that own their state in place of `impl MortiseApp`. The methods share `dialog`,
  `toast`, `runs` and `apps[index]`; pulling them apart changes borrow structure and is not mechanical.
- Splitting the frame function `ui` (718 lines) or `draw.rs` (1,507 lines).
- Splitting the big views (`generate/view.rs` 2,401, `settings/view.rs` 2,004). Do it when one is next changed.
- One action enum per tab instead of `WorkAction`.
- Repository traits, ports, events, layer folders (section 2).
- Renames to the interface vocabulary (`Cycle` to circle).
- Removing the 44 `cfg` lines in `run/process.rs` by gating the whole module after C1. It is possible, and
  it is a feature-flag change. Do it as its own small change with the matrix check, not inside a move.
- Merging the 17 test binaries of `mortise-app` into one. It may shorten link time (inferred). It is a
  build change, not an architecture change.

---

## 6. Size estimate

| What | Count |
|---|---|
| Files moved, core | 22 (20 `.rs`, 2 `.json`) |
| Files moved, app | 24 of 28 |
| Files split | 4: `app.rs` into 9, `strings.rs` into 13, `output.rs` into 2, `catalog/mod.rs` into two `mod.rs` |
| New files | about 40: 8 wiring files, 12 `strings.rs`, 11 `mod.rs` in the app, 7 `mod.rs` in the core, `program.rs`, `target.rs` |
| Test files moved | 0. Import lines changed: about 35 |
| Snapshot images changed | 0 of 122 |
| Steps | 14 (5 core, 6 app, 1 text, 2 documents) |
| Commits | about 30 (A6 is 8, S1 is about 12) |
| Pull requests | 3 |
| Models | Sonnet 11 steps, Opus 3 steps (C4, A6, S1), Fable 0 |
| Lines of behaviour changed | 0 |

## Measured again on `main` at `4fa5124`

The research above was read on `95c00dc`, before PR #19 (migration state, `module:test`, translations)
merged. This section is measured on `main` at `4fa5124`, after PR #20. Where it differs from the
research, this section holds.

### Sizes

| What | Research (`95c00dc`) | `main` (`4fa5124`) |
|---|---|---|
| `mortise-core` source | 37 files, 13,314 lines | 37 files, 14,186 lines |
| `mortise-app` source | 28 files, 25,927 lines | 28 files, 27,994 lines |
| Snapshot images | 122 | 140 |
| `#[test]` in `mortise-core/src` | not counted | 123 |
| Names in `mortise-app/src/strings.rs` (`pub const`, `pub fn`, `pub static`) | 561 | 604, and 10 tests |

Files that grew most: `app.rs` (app) 3,396 to 3,912; `generate_view.rs` 2,401 to 3,025; `strings.rs`
3,106 to 3,385; `package.rs` 514 to 759; `output.rs` 452 to 577; `catalog/actions.rs` 879 to 999;
`catalog/generators.rs` 1,017 to 1,228; `findings.rs` 328 to 404; `run/read.rs` 100 to 122.
No file was added or removed in either crate. Line ranges inside `app.rs` (section 1.3) moved; step A6
measures them again.

### Module links that #19 added

- `package.rs` now uses `Catalog` as well as `Target`: `ships()` reads `Catalog::v13_4()`, and
  `generator_gives_data()` calls `ships()`. `catalog/generators.rs` uses `package::{gives_data, ships}`.
  Moving `Target` alone therefore no longer breaks the `catalog` and `package` cycle.
- `findings.rs` uses `package::PackageData`.
- `run/read.rs` uses `package::{DataError, whole_document}`.

### Changes to the plan

1. Step C4 also moves `ships()` and `generator_gives_data()` from `package.rs` to
   `generate/catalog.rs`. Both say what the catalog holds, not what a document says. `lib.rs` still
   exports `generator_gives_data` under the same name. The test `gives_data` in `package.rs` keeps its
   assertions and takes `generator_gives_data` from `crate::generate` in its `use` line, so no assertion
   changes. After this, `package.rs` names neither `Catalog` nor anything in `generate/`.
2. The core dependency direction (section 3.2 and plan section 8.3) becomes, lowest first:
   1. `php`, `program`
   2. `modules` (php, program)
   3. `boundary` (php)
   4. `package` (modules, php)
   5. `findings` (modules, boundary, package)
   6. `run` (program, package)
   7. `actions` (modules, run, package)
   8. `generate` (modules, package, run)
   9. `app` (modules, boundary, findings, package)
   10. `surface`, `runtime`, `palette` (app, modules; palette also actions, generate, run)
   11. `editor` (program), then `settings` (app, modules, editor, findings, run, runtime)
3. Step S1 places 604 names, not 561. The check is the same: the count over all `strings.rs` files equals
   the count before the split, no name is defined twice, every "not in the spec" mark is kept.
4. The images check covers 140 files, not 122.

## What the build did differently from the plan

The sections above are the plan. The build differed from it as follows.

- **No re-exports in a feature `mod.rs`.** Section 3.3 said each `mod.rs` of `mortise-app` re-exports the names
  that `app/` uses. In both crates, a folder's `mod.rs` declares its files and re-exports nothing, so each
  item has one path (`crate::generate::view::GenerateState`, `crate::modules::module::Module`). The core's
  `lib.rs` still re-exports every public name at the crate root. Some imports in `mortise-app` keep the old
  module names as aliases (`use crate::generate::view::{self as generate_view, ...}`), so the method bodies
  did not change.
- **`ships()` and `generator_gives_data()` moved to `generate/catalog.rs`** (see "Measured again on main").
- **The output test helper.** `recording()`, a test helper used by the tests of both halves of `output.rs`,
  stays in one copy in `actions/output.rs`. Its test module is `pub(crate)`, and `generate/output.rs` imports it.
- **The CI feature check builds the library and the binaries only** (`--lib --bins`). The tests depend on
  `mortise-core` itself with its default features, so `--all-targets` turned every feature on in every case.
  The first Sol review of PR 1 found it. The job also sets `RUSTFLAGS=-D warnings`. With it, one unused import
  in `run/process.rs` without the `run` feature showed up and was gated. The commits of PR 1 before that fix
  fail the job under `-D warnings`; nothing else in them does.
- **Gating all of `run/process.rs`** stays out, as section 5 says. The second Sol review of PR 1 asked for it.
- **`app/mod.rs` is 1,542 lines**, a little over "about 1,500". `set_rules` and `save_rules` stay in it,
  because no wiring file covers rules, and so do the helpers that several wiring files use.
- **`AppCatalog`, `Generating` and `SelectNew`** live in `app/generate.rs` and are `pub(super)`. Their fields
  stay private. `catalog_read` and `dry_run_read` live there too, because they write those fields.
- **Strings.** 604 names were placed, not 561 (PR #19 added text). `src/strings.rs` keeps 96 shared names,
  and twelve feature files hold the rest. Five private helpers of `src/strings.rs` became `pub(crate)`
  because feature text calls them: `list_of`, `capitalised`, `generator_noun`, `with_article`,
  `missing_translations`.
- **One test was flaky under load**: `delete_asks_for_the_name_and_selects_all_modules_when_it_ends`
  (`crates/mortise-app/tests/actions.rs`) ran out of its 10 s wait twice in local full runs while other
  builds ran, and passed on each re-run.
