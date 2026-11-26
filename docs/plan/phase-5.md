# Phase 5 plan

Outline, version 1, 26 November 2026.

Status: approved on 26 November 2026. Nothing is built. Section 4 records the owner's answers.

Phase 5 is small. It adds one boundary rule, the list of modules each module may use (cloud request R9). It closes the gap of Windows, which no one has looked at. It makes room for the first feedback after 1.0.
Everything reads the files of one app at one commit, as phase 1 does. Screen ids refer to the specification.

Phase 5 builds no new screen id. It changes 2 of phase 1: the rules of S5.3 and the links that break a rule on S5.7.

## 1. The line with Mortise Cloud

The line of phase 4, section 1, holds: Mortise covers one app, at one commit, on one machine. The list of what Mortise leaves out holds too. Each candidate of phase 5 was checked against it before it went in.

| Candidate | One app, one commit, one machine | In phase 5 |
|---|---|---|
| The allowed use rule (R9) | Yes. A rule is a check of one commit. Phase 4 names it as a free rule | Yes, milestones 29 and 30 |
| A baseline for the new rule, so an old app can adopt it | No. "New problems only" compares with the past | No. A module without a list is not checked, so a team adopts the rule one module at a time |
| Reading the graph export of Larch, with the links of the booted app | Yes for the line. It needs a file that PHP wrote | No. Section 2, option (b) |
| A job that starts the window on Windows in CI | Yes. It concerns Mortise itself, not an app | Yes, milestone 31 |
| Issue forms and a link to them in the window | Yes. Mortise sends nothing by itself | Yes, milestone 32 |
| Usage counts sent from the window (telemetry) | No. It collects across machines and across people | No. Left out, section 10 |

## 2. R9: who owns the allowed use rule

Cloud request R9 asks for a third rule: a list of the modules each module may use. The cloud app needs it for its own layering (cloud spec, section 10), and customers get the same. The owner suggests that Larch can cover it.

**What Larch is.** `nwidart/larch` 0.1.0 is a Pest plugin by the same owner. It checks links between modules in a Pest test and needs PHP 8.3 and Pest 4.3.

- `toOnlyDependOn()` is the rule of R9, for one module, written in PHP.
- `arch()->preset()->mortise()` reads `mortise.json` and runs the two rules of today on imports, as Mortise does. Shared fixture apps in the Larch repository keep the two programs in agreement.
- With `via(Edge::all())` a rule also checks links that only the booted app shows: facades, container keys, listeners, jobs.
- `php artisan larch:graph` exports the links as JSON, format version 1. It holds links, not rule results.
- A baseline file accepts the breaks of today, so a test fails on new ones only.

**The options.**

| | (a) Larch owns R9. Mortise points to it | (b) Mortise reads Larch rules or Larch output | (c) A third rule in `mortise.json` |
|---|---|---|---|
| Where the list is kept | In a Pest test, as PHP code | In a Pest test. Mortise cannot read PHP rules without running PHP, so it reads the graph export, or a new export of results that Larch must add | In `mortise.json`, beside the two rules |
| Who checks it | Pest, in the app's test run | Larch finds the links. Mortise checks them against a list, which it must still get from somewhere | Mortise, on the links it already reads. Larch's preset checks the same key in Pest |
| PHP | Needed in CI. Mortise runs none | `larch:graph` boots the app. In CI the team runs it and passes the file, as with `--coverage`. In the window the user starts it through the runner. Without the file, no rule | None |
| The cloud line | Holds. Mortise builds nothing | Holds | Holds. No baseline: a module without a list is not checked |
| The window | Shows nothing of the rule | Shows the rule only after a run of PHP | Shows the rule and each link that breaks it, as for the two rules |
| `mortise check` | Does not know the rule. A team has two gates | Checks it only when the file is passed | Checks it, in every format |
| How the cloud gets the findings | It does not. A break is a Pest failure. The cloud would need a new ingest and a parser for Pest output | `mortise push` gains a flag for the file and new members in the push document. The format of Larch 0.1.0 becomes part of the frozen contract of R8 | A new `Check` variant in `findings`. The contract allows it (cloud spec, section 2: "Mortise may add a Check variant"). No cloud change and no new schema version |
| Duplication with Larch | None in Mortise. The rules of one app then live in two places: two in `mortise.json`, one in PHP | Mortise depends on a 0.1 format and still holds the check | The check exists in two engines, as the two rules of today do. One list, one set of shared fixtures |
| Size in Mortise | A docs page | A file reader, a flag, a runner command, new document members, a version check | One key, one check, one finding, one row on S5.3 |

**Recommendation: (c), with Larch as the second runner of the same list.**

- It is the only option where the cloud gets R9 findings with no cloud change and no PHP. Under (a) the cloud never sees them.
- It keeps one home for the rules of an app. Larch already reads `mortise.json` for two rules. A third key follows the pattern that exists.
- It keeps "reading an app needs no PHP". Under (b) the rule would exist only after a run of PHP.
- The duplication is small and already accepted. Mortise has the links; the check is a lookup in a list. The shared fixtures gain one app.

Larch stays the tool for what Mortise cannot see. A team that wants the rule on runtime links writes `toOnlyDependOn()->via(Edge::all())` in Pest. Mortise's docs point to it.

**Until phase 5 ships,** the cloud app can use option (a) today: replace its `not->toUse` test with one `toOnlyDependOn()` test per module. That needs no change in Mortise.

**The key (proposal).**

```json
{
    "rules": {
        "no-circular-use": true,
        "public-surface-only": { "enabled": true, "folders": ["Contracts", "Events"] },
        "allowed-use": {
            "enabled": true,
            "modules": {
                "Audit": [],
                "History": ["Apps", "Billing", "Audit"],
                "Ingest": ["Apps", "History", "Billing", "Audit"]
            }
        }
    }
}
```

- Without the key, the rule does nothing. The apps of today read the same.
- A module with a list may use the modules of that list only. An empty list means it may use no module.
- A module without a list is not checked.
- A module that is off breaks no rule, and no link to it does, as Larch reads `modules_statuses.json`.
- The rule says which module. The second rule still says which classes of it.

## 3. What was checked before writing this

| Subject | How | Result |
|---|---|---|
| Mortise | `main` at `42764e3` (v1.0.0 and #63). Read `boundary/rules.rs`, `findings.rs`, `boundary/strings.rs`, `ci.yml`, `docs/deviations.md`, the plans of phases 1 to 4 | `Rules` holds two rules. `Rules::save` keeps other keys of the file. `Check` has 7 variants, 2 of them rules. No file names Larch. CI builds and tests on Windows. The image tests run on macOS only |
| Larch | Cloned at `7f62292` (v0.1.0, 24 November 2026). Read the README, `src/Mortise/MortiseConfig.php`, `tests/Fixtures/mortise/README.md`, the changelog and the docs: Mortise guide and reference, graph export, module rules, baseline, how Larch reads an app | As section 2. Its preset stops on a broken `mortise.json`, where Mortise uses the defaults. Its graph guide says "Mortise reads this format"; Mortise does not |
| Mortise Cloud | `docs/spec` at `30b282d`: requests R1 to R9, the snapshot contract, the module layout | R9 is "later, not for launch". The cloud stores a check it does not know by its name. Its own app has 11 modules and a "may use" list, checked by a Pest test until R9 |
| Windows | `docs/deviations.md`, phase 3 "Platforms" | "Nothing here was run on Windows." Phase 3: "Windows: built and tested in CI. Not looked at." |
| Feedback | `gh issue list`, `gh repo view` | No issue, open or closed. No issue form. Discussions are off |
| Work in flight | `git worktree list`, `gh pr list` | Six branches, section 5 |

Not checked: no code was written or run for this outline. Whether a GitHub runner for Windows gives a desktop that a screen shot can read is not known; task 31.1 finds out.

## 4. Questions for the owner

Each has a recommendation; the outline assumes it.

Answered on 26 November 2026. The owner chose the recommendation of questions 1, 5, 7 and 10. The other questions were not asked; their recommendations stand until the owner says otherwise.

1. **Who owns R9.** Recommended: option (c) of section 2. Larch's preset reads the same key.
2. **The shape of the key.** Recommended: `rules.allowed-use` as in section 2. A module without a list is not checked. Open inside it: a list for every module at once (the cloud says "every module may use Audit"). Recommended: none in the first version; each list names `Audit`.
3. **Editing the lists in the window.** Recommended: no. S5.3 gets the switch of the rule and the count of modules with a list. The lists are edited in the file. An editor is a new screen with no drawing in the spec.
4. **The drawing of S5.3.** The spec draws two rules. Recommended: add the third row to `docs/spec` first, then build it. Otherwise it is text "not in the spec" and a deviation row.
5. **Larch's baseline and the line.** Larch has a free "new problems only" mode. With the preset, it also covers the rules of `mortise.json`. Mortise builds no baseline either way. Recommended: accept it. It is local to a test run and keeps no history. The owner decides, since the cloud sells the comparison.
6. **The sentence "Mortise reads this format" in Larch's docs.** It is not true today. Recommended: remove it in Larch. Option (b) stays out until a team asks.
7. **The Windows machine.** Recommended: first the CI job of milestone 31, which costs nothing. For what a runner cannot show, a Windows 11 x64 virtual machine on the Linux box (KVM), reached over the same network. Other choices: a cloud machine by the hour, or a real PC. The owner decides on the licence and the cost.
8. **Where feedback goes.** Recommended: GitHub issues with two forms. Discussions stay off. No telemetry.
9. **The size of the feedback slot.** Recommended: at most 5 small changes, each one pull request and no new screen id. The slot closes when milestones 29 to 31 are merged. A larger request starts the list of phase 6.
10. **A release.** Recommended: 1.1.0 at the end of milestone 33. The in-flight work may ship earlier as 1.0.1.

## 5. Work in flight

This work runs now, each in its own branch. Phase 5 does not plan it again.

| Work | Branch | Phase 5 needs it for |
|---|---|---|
| Cleanup of the docs and the statuses | `docs/fix-phase-4-milestones` (#64) and others | nothing |
| Rust 1.100 with a newer mago | `chore/rust-1-100-mago` | Milestone 29 starts after it merges, so the toolchain does not move under it |
| The focus fix of S7.3 | `fix/s7-3-focus` | nothing |
| Reduced motion on Windows and Linux (D3) | `feat/reduced-motion-windows-linux` | Task 31.4 looks at it on Windows |
| PKG-2, the scheduled tasks of a module | `feat/pkg-2-module-tasks` | nothing |
| A manual sweep of the deviation rows on Linux | `docs/linux-sweep` | Task 31.4 uses its row format |

## 6. Workspace changes

```
crates/mortise-core/src/
  boundary/rules.rs            the allowed-use key: read, defaults, save
  boundary/links.rs            a link from a module to a module outside its list
  findings.rs                  one Check variant, one Detail variant
  push/                        the committed push document gains nothing for shop-api;
                               the schema copy allows the new variant already
crates/mortise-cli/
  src/check.rs                 the new finding in text, json and github
crates/mortise-app/
  src/boundary/                the third row of S5.3; the links on S5.7
fixtures/
  layered/                     NEW: a small app with an allowed-use key and one break
.github/
  workflows/ci.yml             a job, not required: the window on Windows
  ISSUE_TEMPLATE/              NEW: two issue forms
```

`fixtures/shop-api` does not get the key. Its push document must stay equal to the example of the cloud (R4).

## 7. Milestones

Each milestone is one OpenSpec change, built in its own worktree and merged through one pull request.

| # | Milestone | Core and tool | Screens | Needs |
|---|---|---|---|---|
| 29 | Allowed use rule | The key in `mortise.json`. The check, the finding, `mortise check` in three formats, the push document. Cloud R9 | none | Question 1 and 2. The mago branch merged |
| 30 | Allowed use in the window | none | The third row of S5.3. A link that breaks the rule on the graph and on S5.7 | 29. Question 3 and 4 |
| 31 | Windows looked at | A CI job that starts the window on Windows and keeps what it shows | Every screen written on Windows. The deviation rows with a Windows result | Question 7 for the last task |
| 32 | Room for feedback | Issue forms. The rule for a request and the line | A "Report a problem" link | Question 8 and 9 |
| 33 | Hardening and release | The shared fixtures with Larch. The tool on three platforms with the new rule | Images in both themes, accessibility, docs site, deviations, release 1.1.0 | 29 to 32 |

Milestones 29, 31 and 32 share no file and can run at the same time. Milestone 30 follows 29.

## 8. Tasks per milestone

The model column follows `AGENTS.md`: Sonnet for a simple, well-defined change, Opus for standard building and tests, Fable only where the reasoning is hard.

### 29. Allowed use rule

| Id | Task | Done when | Needs | Model |
|---|---|---|---|---|
| 29.1 | Propose the OpenSpec change. It changes the capabilities `rules-file`, `cli-check` and `push-document` | `openspec validate --strict` passes | Questions 1, 2 | Fable: it fixes the meaning of the key for two programs |
| 29.2 | Read and save `rules.allowed-use` | An app without the key reads as today. A key with a wrong value gives the defaults and the parser message, as the other keys do. A save keeps the lists | 29.1 | Opus |
| 29.3 | The check and its finding | On `fixtures/layered` the finding names the module, the module it may not use, and each file with its line. A module without a list gives none. A module that is off gives none | 29.2 | Opus |
| 29.4 | A name in a list that is no module of the app | A warning names the entry, so a typing mistake does not pass in silence | 29.3 | Sonnet |
| 29.5 | `mortise check` | Exit 1 on `fixtures/layered`. `--format github` marks each file on its line. Exit 0 with `ci.fail` off | 29.3 | Sonnet |
| 29.6 | The push document | The document of `fixtures/layered` is valid against the schema copy. `SCHEMA_VERSION` stays 1. The document of `shop-api` does not change | 29.3 | Sonnet |

### 30. Allowed use in the window

| Id | Task | Done when | Needs | Model |
|---|---|---|---|---|
| 30.1 | The third row in `docs/spec`, S5.3, and the text of a break | The spec builds. The text follows section 10 of the spec | Question 4 | Opus |
| 30.2 | The row on S5.3 with its switch | The switch writes `enabled` and keeps the lists. Without the key the row says that no module has a list, and the switch is off. Mortise asks before it creates `mortise.json`, as today | 29, 30.1 | Opus |
| 30.3 | The break on the graph and on S5.7 | A link that breaks the rule has the mark of the other rules. S5.7 names the rule and the files | 30.2 | Opus |
| 30.4 | Images, accessibility | The changed images in both themes, each looked at. The switch has a name and a target of 24 by 24 px | 30.3 | Sonnet |

### 31. Windows looked at

| Id | Task | Done when | Needs | Model |
|---|---|---|---|---|
| 31.1 | A probe job on `windows-latest`, not required: start `mortise-desktop` on `fixtures/shop-api` with `MORTISE_CONFIG_DIR`, `MORTISE_DATA_DIR` and `MORTISE_SECRETS=memory`, take a shot of the desktop, keep it as an artifact | The artifact shows the window with the modules of `shop-api`. Or the task records that a runner gives no desktop to read, and 31.5 takes the whole look | nothing | Fable: the result is unknown and decides the rest |
| 31.2 | Write every screen on Windows: `cargo test -p mortise-app --test screens -- --ignored` in the same job, `target/screens` kept as an artifact | Every screen id has a file. Each one tied to a Windows row is looked at once | nothing | Sonnet |
| 31.3 | The rows that need no window, as tests that run on Windows only where no test covers them yet: a batch file that std refuses (S52), the end of a process tree (S53), the quoted test line (S35) | Each row names its test, or the test that already covers it | nothing | Opus |
| 31.4 | Sweep `docs/deviations.md` for Windows | Each row that can differ on Windows has "run on Windows" with the date and how, or "needs a machine". The sentence "Nothing here was run on Windows" goes | 31.1 to 31.3. The D3 branch merged | Opus |
| 31.5 | The rows that need a real session: snapping (D1), the 1 px line (D2), the setup program and SmartScreen, Narrator through AccessKit | Looked at once on the machine of question 7, and written into the rows. Until then the rows say "needs a machine" and the README says so | Question 7 | The owner, with Opus for the notes |

### 32. Room for feedback

| Id | Task | Done when | Needs | Model |
|---|---|---|---|---|
| 32.1 | Two issue forms: a problem, a request. Fields: Mortise version, platform, laravel-modules version, what happened, what was expected | A new issue offers both forms. The request form says what belongs to Mortise Cloud | Question 8 | Sonnet |
| 32.2 | "Report a problem" in the window, in the README and on the docs site | The link opens the form in the browser with the version and the platform filled in. It carries nothing of the app. Mortise sends nothing by itself | 32.1 | Sonnet |
| 32.3 | The rule for a request, in `CONTRIBUTING.md` | It says: check the request against phase 4, section 1, first. One app, one commit, one machine: label `free`. Across time, commits, apps or people: label `cloud`, answer with the reason, and tell the cloud thread | 32.1 | Sonnet |
| 32.4 | The feedback slot | Each change taken has an issue, the label `free`, one pull request and no new screen id. The slot closes with at most 5 changes, or empty, when 29 to 31 are merged. The rest is listed for phase 6 | Question 9 | Opus per change. Fable for the triage against the line |

### 33. Hardening and release

| Id | Task | Done when | Needs | Model |
|---|---|---|---|---|
| 33.1 | The shared fixtures: an `allowed-use` app in Larch's `tests/Fixtures/mortise` with `expected.json` (LARCH-1, section 9). A Mortise test reads a copy of it | Mortise and Larch give the same result on the same app | 29, LARCH-1 | Opus |
| 33.2 | The CI job of the tool runs `fixtures/layered` on three platforms | Paths in the output use `/` on Windows | 29 | Sonnet |
| 33.3 | The docs site: the rules reference, `mortise.json`, `mortise check`, and a how-to "Check runtime links with Larch" | `npm run build` passes. Each page that names two rules names three | 30 | Sonnet |
| 33.4 | `README.md`, `AGENTS.md`, `docs/deviations.md`, this plan | Each says what was built and what was not run on a platform | 29 to 32 | Sonnet |
| 33.5 | Tell the cloud thread: R9 is built, with the key and the name of the check | The cloud sets the status of R9, adds the key to its own `mortise.json` and drops its Pest test or keeps it for runtime links | 29 | Sonnet |
| 33.6 | Release 1.1.0 | The installers and the four archives are on the release. `cargo install mortise-cli` gives 1.1.0 | 33.1 to 33.5, question 10 | Sonnet, by `CONTRIBUTING.md` |

## 9. External dependencies

| Id | Dependency | What phase 5 needs from it | Exists today | Owner | Blocks |
|---|---|---|---|---|---|
| LARCH-1 | `nwidart/larch` | The preset reads `rules.allowed-use` and runs `toOnlyDependOn()` for each module with a list. One more shared fixture app | `toOnlyDependOn()` exists. The preset reads two rules | nwidart | Task 33.1 only. Milestones 29 and 30 do not wait |
| LARCH-2 | `nwidart/larch` | Its graph guide stops saying that Mortise reads the format | The sentence is there | nwidart | nothing |
| CLOUD-1 | Mortise Cloud | Nothing to build. It shows the new check by its name | The contract allows a new `Check` variant | nwidart | nothing |
| WIN-1 | A Windows machine | A session to look at the rows of task 31.5 | None | The owner | Task 31.5 |
| CI | GitHub runners for Windows | A desktop that a screen shot can read | Not known | GitHub | The form of task 31.1 |

## 10. Tests

- The rule is tested once, in `mortise-core`, on the new fixture `layered`: one break, one module without a list, one empty list.
- The tool is tested once at its own layer: the exit code and the three formats on `layered`. It does not replay the cases of the core.
- The window gets one screen test for the wiring of the third row, and the images of S5.3 and S5.7.
- The read and the save of the key go into the tests that already build a `mortise.json`, not into new ones.
- Milestone 31 adds a test only where a Windows row has none.
- The plan adds one fixture. A pull request over 400 net test lines says why in its body.

## 11. What is uncertain

| Uncertain | Why it matters | What settles it |
|---|---|---|
| Whether a Windows runner shows a window that a shot can read | Milestone 31 is free only if it does | Task 31.1, first |
| A third rule with no drawing in the spec | The spec is the source of truth for screens | Question 4, task 30.1 |
| Mortise and Larch read a broken `mortise.json` differently: defaults against a stop | The same file could pass in one and fail in the other | The shared fixtures hold valid files only. The docs say it |
| A team wants the rule on runtime links in the window | Mortise sees imports only | Larch, in Pest. Option (b) if teams ask |
| A team with an old app wants "new problems only" for the new rule | Mortise has no baseline | Lists for some modules only. Larch's baseline, or the cloud |
| No feedback arrives | The slot stays empty | It closes empty. The phase still ends |
| A request sits on the cloud's side of the line | Building it free would undercut the cloud | Task 32.3, and the owner by phase 4, section 1 |
| The in-flight branches touch the same files | A milestone could build on code that moves | Milestone 29 starts after the mago branch. Task 31.4 after the D3 branch |

## 12. Left out, and why

- A baseline, in a file or in a flag: phase 4, section 1.
- Reading Larch's graph export or its rules (option (b)): it ties the frozen push document to a 0.1 format and needs PHP. No team has asked.
- A list for every module at once, patterns in names, and "may use the public surface only" per pair: the second rule already covers the public surface. Each can come later without a break of the key.
- An editor of the lists in the window: no drawing in the spec. The file is the editor in the first version.
- Telemetry and crash reports sent from the window: across machines and people, and against "Mortise sends nothing by itself".
- GitHub Discussions, a chat server, a survey: issues are enough until there is feedback to sort.
- Signing the Windows setup program: it needs a certificate and a cost. Task 31.5 looks at SmartScreen once and reports.
- A Windows build for arm64: no request.
- The R5 follow-up and the statuses of R1 to R8 in the cloud's table: the cleanup in flight and the cloud thread hold them.
- Everything on the "left out" lists of phases 2 to 4 that this plan does not name.
