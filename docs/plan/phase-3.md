# Mortise phase 3: build plan

Status: approved, 23 November 2026. The owner accepts every recommendation in section 2.
The published copy is https://6h4zaivwsd5c.postplan.dev/phase-3.html (version 10).

Phase 3 reads the running app per module. The index and the Summary show the runtime status of each
module. The Metrics tab shows requests, response time, errors, jobs and database time for each module.
All of it is read over HTTP from the actuator of the chosen environment, with the connection that phase 1
built. Mortise writes nothing for it, runs nothing in the app and needs no PHP. The design is in
`docs/spec/`. Screen ids such as S7.3 refer to it.

Phase 3 builds 3 screen ids: S7.1, S7.2, S7.3. The Metrics tab stops showing S7.4. It adds to 3 screens
of phase 1: the runtime mark per row of S3.1 (its sixth drawing), the runtime box of a module on S4.3,
and the Module column of scheduled tasks on S4.2.

Milestones are numbered 18 to 23, because phase 2 used 17 for the package-by-feature refactor.
The phase 4 outline on the published site numbered its milestones 22 to 24. Phase 4 is milestones 24 to 28
(see `docs/plan/phase-4.md`).

## 1. What was checked before writing this

V: read, or asked with `gh api`. I: inferred from the code, not run. Nothing was compiled, run or installed.

| Subject | How | Result |
|---|---|---|
| Mortise | `main` at `da9682e` (v0.1.0). Read `runtime/actuator.rs`, `runtime/live.rs`, `runtime/view.rs`, `index/view.rs`, `summary/view.rs`, `app/workspace.rs`, `testing.rs`. V | The client asks for the base path, `health`, `info`, `mappings` and `scheduledtasks`. It reads JSON only and sends `Accept: application/json`. 200 and 503 are answers. Health every 15 s. `prometheus` and `modules` already show in the list of exposed endpoints. No OpenSpec capability covers the actuator |
| Screen ids | All 44 ids of the spec against `crates/mortise-app/src`, `tests/images.rs` and the stored images. V | 38 built. Open: S7.1, S7.2, S7.3, the sixth drawing of S3.1 and the module box of S4.3 (not built), the Module column of S4.2 (built for tasks whose target is a class name). S5.8, S5.9 and the switch of S5.3 belong to phase 4. Built ids with no stored image: S2.1, S4.1, S5.2, S5.6, S6.5 b to i, S8.1, S8.5 |
| A 13.4 app today | Code read. I | The app runtime box of S4.2 would list the `modules` component with raw JSON, and each `module:x:y` indicator as an app component |
| The Metrics tab | `app/workspace.rs:96-112`. V | Shows S7.4. Its text says the actuator has no metrics endpoint yet, which is now false. The text sits outside `strings.rs`, as does one line in `settings/view.rs:224` |
| `nwidart/laravel-modules` | Releases, tree at `v13.4.0`. V | 13.4.0 is released and carries P9, with P2 to P5, P7 and P8. Module indicators are named `module:{lower case name}:{key}`; a turned-off module registers none. The `modules` component is Down when a module that is on is missing on disk. The `modules` endpoint has no routes and no tasks (P10 in part). No code adds a module to the metrics (A2 in part). Open PR #2290 may add detail keys to the `modules` component |
| `nwidart/laravel-actuator` | Releases, `composer.json`, docs. V | 1.3.1. `prometheus` since 1.2.0: 200 with an empty body when metrics are off, 503 with one line when the store cannot be read, `text/plain; version=0.0.4`. The guide names the module label `module`, with `none` outside a module |
| `nwidart/laravel-metrics` | Own repository now. Releases and docs. V | 1.1.0, store `nwidart/metrics-store`. `http_server_request_duration_seconds` with `http_route` (leading `/`), method and status. `messaging_process_duration_seconds` with `laravel_job_name`. `db_client_operation_duration_seconds` with neither. `laravel_db_queries` counts queries per request or job. Buckets 5 ms to 10 s |
| Route form | Docs of both packages. V | Metrics write `/orders/{order}`; `mappings` writes `orders/{order}`. To confirm on a recorded answer in milestone 18 |
| CI | `.github/workflows/ci.yml`. V | The real-PHP job (not required) installs `laravel-modules ^13.3`, now 13.4.0. It installs no actuator and serves nothing |

Not checked: no package was installed in an app, and no `prometheus` answer of a running app was read.
`cargo doc` was not re-run after `e6c2262`. The default body limit of `ureq` is from memory.
Nothing was run on Windows or Linux.

## 2. Decisions of the owner

The owner accepted each recommendation on 23 November 2026.

1. **The actuator rule.** "The actuator client asks for the actuator base path, `health`, `info`,
   `mappings`, `scheduledtasks` and `prometheus` only. It never asks for `env` and follows no redirect."
   `prometheus` is asked for only while the Metrics tab is open and the window is visible. `modules`
   stays out until a milestone reads it.
2. **Figures per module.** A label named `module` wins; its value `none` counts as "Not in a module".
   Without it, Mortise places a request by its route template through `mappings`, and a job by the
   namespace of its class. A series that matches nothing counts as "Not in a module". Recorded as a deviation.
3. **Database time.** The column reads "unknown" with the tooltip "Needs a newer nwidart/laravel-modules"
   (not in the spec) until the package adds the label (PKG-1).
4. **The status of a module.** The worst of the module's own indicators, in the order Down, Out of
   service, Up, Unknown. Down when the `modules` component reports the module missing. No mark and no box
   for a module with no indicator of its own.
5. **The 95% figure.** The smallest bucket bound under which 95% of requests fall, such as "100 ms".
   Recorded as a deviation.
6. **Errors.** The share of requests answered with a 5xx status.
7. **Environments with different versions.** Decided per environment from what its actuator answers.
   The version in `composer.lock` is not used for this.
8. **The spec text.** S7.2 reads "Add prometheus to exposure.include in config/actuator.php, next to
   health." Section 12.2 is brought up to laravel-modules 13.4.0 and laravel-actuator 1.3.1.
9. **Tasks per module without P10.** Milestone 22 matches a task `php artisan <command>` to the module
   that declares that command. Other tasks stay "none" until PKG-2.
   PKG-2 does not read the `modules` endpoint, which the actuator rule leaves out. The module comes as a
   `module` tag on each task of `scheduledtasks` instead. The tag wins; the match of milestone 22 is the
   fallback for older packages and for tasks the app schedules.
10. **Phase 4.** M1, P6, S5.8 and S5.9 are phase 4. Spec 2.5 and 12.2 change in milestone 18.
11. **Release.** No release during phase 3. At its end the owner decides.
12. **The Linux look.** A live look on the thinkcentre in milestone 23 only, with the screen unlocked by
    the owner at that time. Windows is looked at through CI or recorded as not run.

## 3. Workspace changes

No new crate and no new dependency. The Prometheus text format is parsed in the core: counters, gauges
and histograms only. The chart is drawn with the painter. All metrics code sits behind the feature
`actuator`. Paths follow the `code-layout` capability.

```
crates/mortise-core/src/
  runtime/actuator.rs   + a text read beside fetch(), a body limit set on purpose, asks prometheus
  runtime/live.rs       + the metrics read: every 15 s, only while asked for and visible
  runtime/modules.rs    NEW  status per module from module:{lower}:{key} and the modules component;
                             takes them out of the app's component list
  metrics/              NEW  behind feature actuator, after runtime in the feature order
    mod.rs                   declares the files
    prometheus.rs            the text format: counter, gauge, histogram, labels, escapes
    sample.rs                reads kept in memory, resets, rates, the window, an injected clock
    place.rs                 a series to a module: label module, route template, job class
    figures.rs               figures per module and per route, the 95% bound, the error share
  modules/ or surface.rs     command names per module for milestone 22 (placed by its design)
  testing.rs            StubActuator grows the recorded answers; no second stub
crates/mortise-app/src/
  metrics/              NEW  mod.rs, view.rs (S7.1 to S7.3), chart.rs, strings.rs
  app/metrics.rs        NEW  wiring: range, selected row, when to read
  app/workspace.rs      Tab::Metrics stops returning the S7.4 block
  index/view.rs         the runtime mark per row
  summary/view.rs       the runtime box of a module
  runtime/view.rs       module components leave the app box; the Module column of tasks
fixtures/actuator/      NEW  answers recorded from one real app, each with the command that made it
```

The `metrics` folder needs a delta to the `code-layout` capability: it goes after `runtime` in the order
of features. The CI job `core-features` proves the gate.

## 4. The model

```
ModuleRuntime   per module: status, the failing indicator and its detail as the app wrote it.
                From health components module:{lower}:{key} and the modules component (P9).
Window          15 min | 1 hour | 24 hours. Kept in memory, never written.
Read            one answer of prometheus: time, and per module and per route the values kept.
Place           Label(module) | Route(template, found in mappings) | Job(class, by namespace) | NotInAModule
ModuleFigures   per module, and per route of one module:
                requests per minute, 95% under, error share, jobs per minute, database time share
```

| Figure on S7.3 | Read from | Placed by |
|---|---|---|
| Requests/min | The count of `http_server_request_duration_seconds`, as a rate | The `module` label, else the route template |
| 95% under | The buckets of the same histogram | The same |
| Errors | The share of the same count with a 5xx status | The same |
| Jobs/min | The count of `messaging_process_duration_seconds`, as a rate | The `module` label, else the job class |
| Database time | The sum of `db_client_operation_duration_seconds` against the sum of request time | The `module` label only |

Rules the core holds, each tested once there:

- `prometheus` is read every 15 s, only while the Metrics tab is open and the window is visible.
- Nothing from metrics is written: not to disk, not to `mortise.json`, not to `settings.json`.
- A 24 hour window at 15 s is 5,760 reads. A read keeps one value per figure, per module and per route,
  and is dropped once it leaves the longest window. Reads older than the last hour are thinned to one per
  5 minutes: the store keeps every 15 s read of the last hour and the first read of each 5 minutes before
  that, at most 519 reads in all. Thinning loses no request, because each read holds running totals.
  Memory stays flat after 24 hours.
- A counter that goes down is a reset. The read after it starts a new rate. No negative rate is drawn.
- Switching the environment empties the window. Switching the range keeps it.
- No figure has a colour and no limit (Q16).
- A 503 is an answer that Mortise quotes. An empty answer means metrics are off; the table says so
  (not in the spec).
- The runtime mark never replaces the finding mark. A Down module raises no pip on the rail (Q11).

## 5. Milestones

Each milestone is one OpenSpec change, led in its own T3 thread on a branch `change/phase-3-<slug>` from
`origin/main`. Each is proposed, built, checked with `openspec validate --strict`, merged through a pull
request with a Sol review, then archived through its own pull request with no reviewer. A milestone starts
only when the milestones it depends on are archived on `main`.

| # | Milestone | Change | Core | Screens | Depends on |
|---|---|---|---|---|---|
| 18 | Groundwork | `phase-3-groundwork` | The actuator rule. Capability `actuator-client`. Recorded answers | Spec 12.2 and S7.2 text. The two strings outside `strings.rs` | The decisions of section 2 |
| 19 | Runtime per module | `phase-3-runtime-per-module` | `runtime/modules.rs` | The row mark of S3.1, the module box of S4.3. Module components leave the app box of S4.2 | 18 |
| 20 | Reading metrics | `phase-3-reading-metrics` | `metrics/`: parser, reads, resets, rates, placing. The text read and its pace | S7.1, S7.2, the 503 and empty answers, a plain table. The Metrics tab stops showing S7.4 | 18 |
| 21 | Metrics tab | `phase-3-metrics-tab` | The 95% bound, the error share, database share, figures per route, ranges | S7.3 in full: chart, table, one module with its routes, J and K, range and environment switch | 20 |
| 22 | Tasks per module | `phase-3-module-tasks` | Command names per module; a task matched to the module that declares its command | The Module column of S4.2 and the tasks of the module Summary | 18 |
| 23 | Hardening | `phase-3-hardening` | A 24 hour run with a fake clock | Images, a real-actuator CI job (not required), Linux and Windows looked at, README, deviations, this plan | 19, 21, 22 |

Order: 18 first. Then 19, 20 and 22 in parallel. 21 after 20. 23 after all.

- 19 and 20 both touch `runtime/actuator.rs` and `runtime/live.rs`. If both are ready together, 19 merges
  first and 20 rebases.
- 19 and 22 both touch `runtime/view.rs`, in different functions. The second to merge rebases.

## 6. Tasks per milestone

Sonnet for simple, well defined work. Opus for standard implementation, tests and specs. Fable 5.2 once,
for the one design with system-wide effect. Each lead runs on Opus. A task escalates to Fable only after it
fails twice on Opus. Totals: 30 tasks; Fable 1, Opus 20, Sonnet 9.

### 18. Groundwork

Acceptance:

- `AGENTS.md`, `README.md`, `openspec/config.yaml` and the doc comment of `runtime/actuator.rs` state the
  rule of decision 1.
- A new capability `actuator-client` describes today's behaviour: the paths asked, no `env`, no redirect,
  503 is an answer, the 15 s and 5 minute paces. `openspec validate --strict` passes.
- `fixtures/actuator/` holds answers recorded from one real app with laravel-modules 13.4.0,
  laravel-actuator 1.3.1 and laravel-metrics 1.x, each with the command that made it: discovery, `health`
  with the `modules` component and two module indicators, `mappings`, `scheduledtasks`, `prometheus`
  quiet, busy, empty and 503.
- The change's `design.md` says whether every recorded `http_route` equals `/` followed by the `uri` of
  `mappings`, and whether the `Accept` header suits the text endpoint.
- Spec 2.5, 12.2 and S7.2 are rebuilt from `docs/spec-build`. Deviation S6 is updated. No interface text
  is left outside a `strings.rs`.

| Task | Model | Why |
|---|---|---|
| 18.1 Propose: proposal, design, delta specs (`actuator-client` new) | Opus | Spec writing from code that exists |
| 18.2 Create a Laravel app with the three packages and two modules with an indicator each, expose the endpoints, make traffic, record the answers with a README | Opus | Judgement on what to record; PHP and Composer on the Mac |
| 18.3 Compare routes and job names between the answers; write the findings into `design.md` | Opus | Settles the top risk of the phase |
| 18.4 The rule in four places; the two strings into `strings.rs` | Sonnet | Fixed text |
| 18.5 Update `docs/spec-build` for 2.5, 12.2 and S7.2, rebuild `docs/spec`, update deviation S6 | Sonnet | Mechanical |

### 19. Runtime per module

Acceptance:

- A core test on the recorded `health`: `module:blog:payments` belongs to module `Blog` by its lower case
  name. The status follows decision 4. A module with no indicator has none.
- A module that the `modules` component marks with an error reads Down with the app's own sentence.
- With hidden components, or an app before 13.4, each row keeps one mark and the module box is not drawn.
- The runtime mark never replaces the finding mark. A Down module raises no pip on the rail.
- Images of the marks on two-line and one-line rows (above 20 modules) and of the module box, light and
  dark, looked at. Each mark has an accessible name.

| Task | Model | Why |
|---|---|---|
| 19.1 Propose: capability `module-runtime`; design of the two marks on a one-line row and on the strip | Opus | One layout decision, otherwise standard |
| 19.2 Core: `runtime/modules.rs`, the nested `modules` detail, tests on the fixtures | Opus | Parsing with rules |
| 19.3 App: the mark in `index/view.rs` | Opus | Layout in a dense, shared row |
| 19.4 App: the module box in `summary/view.rs`; module components leave `runtime/view.rs` | Opus | Reuses the component list |
| 19.5 Strings, images, accessible names, deviation rows | Sonnet | Well defined once 19.3 and 19.4 exist |

### 20. Reading metrics

Acceptance:

- Parser tests on the recorded answers: labels with escapes, `le="+Inf"`, `_sum` and `_count`. Unknown
  families are skipped. A bad line does not fail the read.
- `prometheus` is asked only while the Metrics tab is open and the window is visible: a stub test counts
  the requests. `env` is never asked.
- A counter that goes down starts a new rate. No negative rate.
- Placing follows decision 2, tested once in the core.
- A generated answer for 60 modules is parsed within a time and a memory the change states. The body
  limit of the client is set on purpose and tested.
- S7.1 and S7.2 match the spec text; "Try again" and "Open settings" work. The 503 and empty answers have
  their sentences. Images in both themes.
- After a read, the config and data folders are unchanged.

| Task | Model | Why |
|---|---|---|
| 20.1 Propose: capability `metrics`; design of the read store (what a read keeps, the 24 hour bound, the reset rule, the clock, threading with the live runtime worker); delta to `code-layout` | Fable 5.2 | The one design with system-wide effect: memory over 24 hours, the shared worker, the feature order |
| 20.2 `metrics/prometheus.rs` and its tests | Opus | Standard parser |
| 20.3 `metrics/sample.rs`: window, resets, rates, injected clock | Opus | Rules with edge cases |
| 20.4 `metrics/place.rs` and `figures.rs`: requests and jobs, the label path | Opus | Reuses the routes and `module_of` |
| 20.5 Client: the text read, the body limit, the pace in `runtime/live.rs` | Opus | Touches the shared worker loop |
| 20.6 App: the `metrics/` folder, S7.1, S7.2, 503 and empty, a plain table; remove the S7.4 use | Opus | Standard screen work |
| 20.7 Images, accessible names, the 60-module measurement | Sonnet | Mechanical |

### 21. Metrics tab

Acceptance:

- The table has the seven columns of S7.3. No figure has a colour or a limit.
- "95% under" follows decision 5 and "Errors" follows decision 6, each with one core test.
- Ranges 15 min, 1 hour and 24 hours. The range switch keeps the reads; the environment switch empties them.
- "read every 15 s, last read 8 s ago" counts up between reads.
- With a module selected, the chart shows that module and the table lists its routes. J and K move
  through the table and the chart follows; kittest drives it.
- Database time reads "unknown" with its tooltip when no series has the `module` label.
- The chart has an accessible name and a text summary. Images in both themes with a fixed clock.

| Task | Model | Why |
|---|---|---|
| 21.1 Propose: delta to `metrics`; design of the chart and the table | Opus | Follows the drawing |
| 21.2 Core: the 95% bound, error share, database share, figures per route, ranges | Opus | Small rules on top of 20 |
| 21.3 App: `metrics/chart.rs` with the painter: straight segments, 1.5 px ink, hairline grid | Opus | New drawing code, bounded |
| 21.4 App: the table, one module, range and environment switch, J and K | Opus | Standard |
| 21.5 Strings, images, accessible names, deviations | Sonnet | Mechanical |

### 22. Tasks per module

Acceptance:

- A task `php artisan billing:send-invoices` belongs to Billing when Billing declares that command. A task
  with only a description stays "none". A task list from the package, when present, wins.
- The module Summary lists the same tasks.
- Reading command names does not slow the first read of an app: measured on `fixtures/shop-api` and a
  generated 60-module app, written in the report.

| Task | Model | Why |
|---|---|---|
| 22.1 Propose, with the cost of reading command names when an app is read | Opus | One trade-off to state |
| 22.2 Core: command names per module in the snapshot, the match of a task, using the reader of the surface scan | Opus | Reuse across features |
| 22.3 App: the column and the module Summary; assertions in the test that already builds that render | Sonnet | Small, well defined |

### 23. Hardening

Acceptance:

- A 24 hour run with the fake clock: memory stays under the bound the test states once the window is full.
- Images for S7.1 to S7.3 and the drawings per module, both themes. The built ids without a stored image
  (section 1) get one, or a line in the change that says why not.
- A CI job, not required, serves a real app with the actuator and reads `health` and `prometheus` through
  the client, in `#[ignore]` tests as `real_php.rs` does.
- Linux looked at on the thinkcentre with the screen unlocked by the owner. Windows looked at through CI,
  or recorded as not run.
- `README.md`, `docs/deviations.md` and this plan updated.
  `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passes.

| Task | Model | Why |
|---|---|---|
| 23.1 The 24 hour memory test | Opus | Care with the clock and the bound |
| 23.2 The real-actuator CI job and its ignored tests | Opus | Composer, a served app, timing |
| 23.3 Images and the accessibility sweep | Sonnet | Mechanical |
| 23.4 Linux and Windows look, deviation rows | Sonnet | A checklist |
| 23.5 README, plan, deviations | Sonnet | Docs |

## 7. Package work, kept separable

Nothing below blocks a milestone. Each is a brief for the owner's other repositories.

| Id | Repository | What | Mortise without it |
|---|---|---|---|
| PKG-1 (A2, rest) | laravel-modules | Add the `module` attribute to the built-in metrics through the attribute callbacks of laravel-metrics. For a query, the module of the request or job that runs it | Mortise places requests, the 95% figure, errors and jobs itself. Database time reads "unknown" |
| PKG-2 (P10, rest) | laravel-actuator, laravel-modules | Each task of `scheduledtasks` has `"tags": {"module": "Blog"}` when a module scheduled it in `configureSchedules()`. laravel-actuator PR #40 adds `Actuator::tagScheduledTasksUsing()`; laravel-actuator 1.5.0 ships it (26 November 2026). laravel-modules PR #2311 adds the tag; laravel-modules 13.6.0 ships it (26 November 2026). Mortise reads the tag | Milestone 22 matches artisan tasks. Other tasks stay "none" |
| PKG-3 | laravel-modules | Open PR #2290: an invalid `module.json` on health | Nothing. New detail keys are read as text |
| P6 | laravel-modules | Where a public surface is stored | Phase 4 |

The work itself needs PHP 8.4 and Composer on the Mac to record the answers in milestone 18, and a
metrics store for that app (a Laravel cache store is enough; Redis only to check two instances).
No new account or secret.

## 8. Tests and continuous integration

- Recorded answers in `fixtures/actuator/`, each with the command that made it. The local test server of
  phase 1 serves them. `StubActuator` grows them; no second stub.
- The parser, rates, resets, placing, the 95% bound and the error share are each tested once in the core.
  A screen test proves the wiring once.
- The read store takes a clock, so a 24 hour window runs in milliseconds.
- Image tests for every new drawing, light and dark, on macOS, with a fixed clock.
- `core-features` builds the core with `actuator` alone and without it; `metrics/` compiles in the first
  and is absent in the second.
- One new CI job, not required, in milestone 23. No new required job.
- Known flaky locally under load: the image tests `actions` and `destructive_actions`, waiting for the
  file watcher. Both pass alone.

## 9. Risks

| Risk | Why it matters | What settles it |
|---|---|---|
| `http_route` and the `uri` of `mappings` differ beyond the slash: optional parameters, domains, a prefix | Placing fails for those routes | Task 18.3 on real answers |
| The size of one `prometheus` answer for 60 modules | The client reads the whole body; `ureq` has a default limit | The measurement of 20.7; the limit set in 20.5 |
| The client sends `Accept: application/json` to a text endpoint | The endpoint may refuse it | The recorded answer of 18.2 |
| Values reach the store after the response; a cache store keeps one server's numbers | A 15 s rate may jump; one read may show one server | A run in 18.2. The table says what one read covers |
| Two marks on a one-line row above 20 modules, and on the strip | S3.1 draws them on a two-line row only | Design in 19.1, images in 19.5 |
| A module name whose lower case form differs, or an app indicator named `module:...` | The wrong module gets a status | Match on the lower case name; test in 19.2 |
| Reads every 15 s add requests to production | The app's team may not want it | Reads stop when the tab closes; the actuator leaves its own routes out of the metric |
| The packages moved three times in two days | Fixtures go stale | The job of 23.2 |

## 10. Left out, and why

- S5.8, S5.9, Accept on S5.7, M1 and P6: phase 4. They read files, not the running app.
- Limits, colours and alerts on metrics: Q16 puts them in the later cloud service.
- Storing metrics on disk or across restarts: the spec keeps them in memory for the chosen range.
- Other metrics of the package (cache, exceptions, scheduler, transactions): S7.3 draws five figures.
- Reading the `modules` endpoint: it adds nothing the files do not give. PKG-2 puts the module of a task on
  `scheduledtasks`, so the rule keeps `modules` out.
- Deviations D1 to D3 and D5: no tie to live data. Looked at in milestone 23 only if the look on Windows
  or Linux shows them.
- The phase 2 "left out on purpose" list and its open restart questions.
- A release: only if the owner asks.

## 11. What the build did

Milestone 18 to 23 as built. Each milestone is one OpenSpec change, with a pull request for the build and
one for its archive.

| PR | Milestone | What |
|---|---|---|
| #30 | 18 Groundwork | Phase 3 groundwork: actuator rule, recorded answers, spec text |
| #31 | none (fix) | fix(app): keep one title across tabs and indent json in the runner |
| #32 | none (agent rule) | docs: add the worktree rule for agents |
| #33 | none (fix) | fix(app): keep a long component detail beside its name in the runtime box |
| #34 | 18 | Archive the phase 3 groundwork change |
| #35 | 22 Tasks per module | feat: place scheduled tasks in the module that declares their command |
| #36 | 19 Runtime per module | feat: runtime status per module (milestone 19) |
| #37 | 22 | Archive module tasks change |
| #38 | 20 Reading metrics | feat: read metrics per module (milestone 20) |
| #39 | 19 | Archive runtime per module |
| #40 | 20 | Archive reading metrics |
| #41 | 21 Metrics tab | feat: build the metrics tab of S7.3 (milestone 21) |
| #42 | 21 | Archive metrics tab |
| #43 | 23 Hardening | ci: real actuator job (milestone 23) |
| #44 | 23 | Milestone 23 hardening: 24 h memory test, environment switch wrap, images and accessibility sweep, strings sweep, spec-build paths |
| this PR | 23 | docs: README, this plan and the deviations |
| to come | 23 | Archive of `phase-3-hardening` |

### Differences from the plan

- Reads past the last hour are thinned to one per 5 minutes. Section 4 first said to keep every read for
  24 hours. That takes 517 MB for 60 modules. The owner confirmed the change on 23 November 2026
  (`phase-3-reading-metrics`, D1).
- Measured memory of the 24 hour run, with a fake clock. 88 subjects hold 6.34 MB; 660 subjects (60 modules)
  hold 47.18 MB against a bound of 47.25 MB. The window is full and flat from read 5,742. The test runs
  88 subjects, because 660 take 13.6 s in a debug build.
- Measured cost of the figures. The slowest call of `Figures::of` took 3.06 ms (60 modules, 24 hours,
  the total selected). That is under the 8 ms of the rule, so the figures stay on the interface thread.
- The real-actuator job runs `php artisan serve --no-reload`. Without it, `serve` removes the process
  variables the three servers use to switch metrics off or break the store.
- The environment switch wraps (deviation S70). The plan had no rule for names wider than the tab.
- S6.5 e has no stored image. Its sentence quotes the port of the stub, which changes with every run.
- S7.3 after a choice of environment showed the waiting state, which had no switch, so keyboard focus
  left the switch and Tab had to reach it again. Fixed: the waiting state keeps the environment switch,
  and the focus stays on the chosen environment (deviation S77).
- Tasks per module (milestone 22) match a task to the module that declares its command. The plan asked for
  nothing more: PKG-2 was still open.
- After phase 3, Mortise reads the `module` tag of PKG-2 on each task of `scheduledtasks`. The tag wins;
  the match of milestone 22 stays the fallback. The package changes are open pull requests.

### Platforms

- macOS: looked at, in both themes, through the stored images.
- Linux: built and tested in CI. Looked at on the owner's Linux box (Hyprland 0.56.2 on Omarchy) with the
  recorded answers served there: the index with runtime marks, a module Summary with its runtime box, S7.1
  and S7.3, at 1920 by 1080 and 960 by 600. One difference, from a compositor plugin: deviation D8.
- Windows: built and tested in CI. Not looked at.
