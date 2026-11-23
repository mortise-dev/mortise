# Design

## Context

See `proposal.md` for why. Milestones 10 to 14 and 16 (`openspec/changes/archive/2026-11-23-phase-2-*`)
built the runner (`Runs::start`, one run per app, time limit), the drawer and its history
(`AppRuns`), the fake artisan that replays `fixtures/output/*.json`, the action catalog
(`catalog/actions.rs`, `plan`), the Actions tab, the generator catalog in P3's shape
(`catalog/generators.rs`, `v13_3.json`), the output reader (`output.rs`) and the Generate tab.
`AppSnapshot::package` holds the package version from `composer.lock` (`composer::Version`: `raw` and the
branch `alias`). An app without `composer.lock` is not read at all (`ReadError::NoLock`).

Package facts, read at the tag `v13.4.0` and checked against the output recorded in `fixtures/output`
(task 15.2). The recorded shapes match the facts below, with these additions:

- JSON rules of every `--json` command: standard output holds one JSON document on one line and nothing
  else; every other message goes to standard error; no ANSI codes, even with `--ansi`; the exit code
  means what it means without `--json`.
- P3 `module:generators --json`: `{"generators":[...]}` in the shape `catalog/generators.rs` reads,
  sorted by name, 43 generators, hidden commands included, an app's own `GeneratorCommand` subclass in
  the group `other`. Each generator now lists the options `dry-run` and `json`.
- P2 and P5, every generator and `module:make`: `{"command","dry_run","files":[{"path","exists","action",
  "result"?}],"errors":[...]}`. `path` is absolute with forward slashes. `action` is `create`, `replace`,
  `keep` or `delete` (the folder `module:make --force` deletes first). `result` (real runs only) is
  `created`, `replaced`, `exists`, `deleted` or `failed`. A dry run writes nothing and exits with the code
  the real run would give: a `keep` file gives exit 1 and its error, with the absolute path of the file in the
  text of the error. Files are in write order and
  include the files of called generators (`module:make-model --all`, `module:make-component`,
  `module:make`). `module:make-model` passes `--force` to the generators it calls (PR #2302).
- P4 `module:migrate-status [<module>...|--all] --json`: `{"modules":[{"name","migrations":[{"name",
  "ran","batch"}]}],"migration_table":bool}`. Migrations sorted by name. Without the migrations table every
  migration is `ran: false`, `migration_table: false`, exit 0. An unknown module gives `{"modules":[],...}`
  and exit 1. `--all` means the modules that are on. The package's `Migrator::rollback` rolls back the
  module's ran migrations whose batch is the highest batch among them.
- P7 `module:test [<module>...] [--all] [--filter=] [--parallel] [--json]`: the runner output goes to
  standard error; standard output ends with `{"modules":[{"name","ran","passed","failed","skipped",
  "exit_code"}],"passed","failed","skipped","exit_code"}`. `ran: false` for a module without `tests`;
  counts are `null` then, or when the runner wrote no report. A `tests` folder with no test gives `ran: true`
  and counts of 0. `--all` takes every module, on or off. The runner output goes to standard error.
- P8 `module:lang [<module>...|--all] --json`: `{"modules":[{"name","checked","locales":{"<locale>":
  {"missing_files":[...],"missing_keys":{"<file>":["key",...]}}}}]}`. `checked: false` with fewer than
  two locales. Exit 1 when a key or file is missing. `{"modules":[]}` and a non-zero exit when no module.

The rule of `AGENTS.md` holds for every new command: Mortise runs a command in the app only when the user
starts it, with the exact command on screen first. Nothing below runs PHP on its own.

## Goals / Non-Goals

**Goals**: the version gate; reads with their own place on screen; P3 with a cache; P2 with the file list,
S9.3 and counted buttons; P5 results; P4 in S10.1 to S10.5, S4.3 and Findings; P7 Run tests; P8 in
Findings; recordings and real-PHP tests; images, deviations, README.

**Non-Goals**: P1, P6 (S5.8, S5.9), P9, P10. A dry run on every keystroke (it would run PHP that the user
did not start, see D3). Reading the migration state when an app opens. A release of Mortise.

## Decisions

### D1. The version gate (`mortise_core::package`, task 15.3a)

```rust
/// True when the app's package gives P2 to P8: a released 13.4.0 or later.
pub fn gives_data(version: &Version) -> bool;
```

`Version::full()` must be `MAJOR.MINOR.PATCH`, ASCII digits only, and `(MAJOR, MINOR) >= (13, 4)`.
Anything else is false: `dev-master`, `13.x-dev`, a branch alias, `13.4.0-RC1`, `13.4.0-beta1`.
A dev build or a pre-release may lack a feature, and Mortise never probes a feature by running a
command and reading its failure. An app without `composer.lock` is not read, so the gate never sees it.
The gate is computed from the snapshot each time; nothing is stored. Tested once in core with
`v13.4.0`, `13.4.0`, `13.3.9`, `14.0.0`, `13.4.0-RC1`, `dev-master` with alias `13.x-dev`.

Where it applies: every place below that says "a 13.4 app" means `gives_data(&app.package)`. Elsewhere the
behaviour of milestones 10 to 14 stays unchanged, with its deviations (S39, S41, S42, S44).

### D2. Reading a JSON document from a run (`mortise_core::package`, task 15.3a)

```rust
pub enum DataError {
    /// Standard output held no line, or only empty lines.
    Missing,
    /// The last line is not the document: serde's message.
    Malformed(String),
}
/// The last non-empty line of standard output, read as `T`.
pub fn document<T: DeserializeOwned>(stdout: &[String]) -> Result<T, DataError>;
```

Only standard output is read, and only its last non-empty line: the package prints the document last,
and a line of a broken PHP setup before it does not matter. The exit code does not decide whether the
document is read: a dry run with a `keep` file exits 1 with a whole document. Each command's types live in
`package.rs` (one `Deserialize` struct per document of the Context section, unknown keys ignored, so a
later package can add keys).

`AppRuns` keeps the standard output lines of the run on screen apart, as well as in the mixed output it
shows (`RunEvent::Line` already carries the stream). `ended()` gives them.

**Line length.** `process.rs` cuts a line at `KEEP` (64 KiB). If the recording of P3 (15.2) is longer
than 64 KiB, a line of standard output is kept up to `OUTPUT_LIMIT` (1 MB) instead; standard error keeps
`KEEP`. A cut document is `Malformed`, never read in part. 15.2 records the size and 15.3a applies this
only when it is needed.

**Fallback with a visible reason.** A missing or malformed document never fails silently: the place that
shows the data says what happened, with the reason in quotes, and shows what milestones 10 to 14 showed:

| Command | Fallback |
|---|---|
| `module:generators --json` (P3) | The shipped catalog stays. The picker says "Mortise could not read the generators of shop-api: "<reason>". The list is the one shipped for 13.4." (not in the spec) |
| dry run (P2) | No file list. The form says "The dry run gave no file list: "<reason>"." and keeps "The files are listed after the run." |
| generator run (P5) | The result is read from the output lines (`output.rs`), with the line "Read from the output: the package's answer was "<reason>"." (not in the spec) |
| `module:migrate-status --json` (P4) | The state stays as it was (or absent). The state box says "Mortise could not read the migration state: "<reason>"." |
| `module:test --json` (P7) | The toast falls back to the toast of a failed or done action without counts |
| `module:lang --json` (P8) | The report stays as it was. Findings keep the earlier report, or none |

A read that exits non-zero with no document gives as reason the last non-empty line of standard error
(plain text, without ANSI), else "exit code N". A read that did not start gives the reason of S11.1.

### D3. Reads (`mortise_core::run::read`, task 15.3a)

A read is a package command whose only purpose is data: `module:generators --json`, a generator with
`--dry-run --json`, `module:migrate-status ... --json`. It differs from a run in three ways:

- It shows its result where the data goes (the picker, the file list, the state box), not in the drawer.
  It is not kept in the history. A read in the drawer would push out the output of the command the user
  is looking at, such as the migrate that a state read follows.
- Its line is on screen before it runs, beside the control that starts it, in the code style, quoted as
  the drawer quotes it (`line_now`).
- While it runs, its place says "Reading" (not in the spec) and the app counts as busy.

It shares the rest with a run: `Runs::start` (one command at a time per app, the same lock; a read is
refused while a run runs, and the other way round, with the tooltip "<label> is running"), the runner of
the app, `--no-interaction --ansi`, the time limit of the app, and Stop on close (`on_exit`).

```rust
pub struct Read { run: Run, stdout: Vec<String>, stderr: Vec<String>, cut: bool }
impl Read {
    pub fn start(runs: &Runs, app: &Path, php: &PhpSettings, args: &[String]) -> Result<Read, StartError>;
    /// Takes what arrived. `Some` once the read ended.
    pub fn poll(&mut self) -> Option<ReadEnd>;
}
pub struct ReadEnd { pub end: RunEnd, pub stdout: Vec<String>, pub stderr: Vec<String> }
```

The app keeps at most one `Read` per open app, with what it is for:
`enum Reading { Catalog, DryRun { words }, Migrations { target } }`, polled each frame like `AppRuns`.

Why a dry run is not run on every keystroke, as S9.2 implies ("The command is built from the form on
every keystroke. The file list comes from a dry run of the command."): a dry run boots the app and runs
its PHP. The rule of `AGENTS.md` lets Mortise run a command only when the user starts it. The form offers
"List files" with the line it runs (deviation, design D5).

### D4. P3: the app's generators (`catalog`, `generate_view.rs`, task 15.3)

- **Shipped**: `catalog/v13_4.json` is the recording of `module:generators --json` on a fresh 13.4.0 app
  (15.2), unchanged. `Catalog::v13_4()` like `v13_3()`. `Catalog::shipped(version)` gives `v13_4` for a
  13.4 app and `v13_3` otherwise. The mark "for nwidart/laravel-modules 13.x" shows when the shipped
  catalog is used for another minor version (`13.4` beside `13.5.0`, `13.3` beside `13.2.0`).
- **Read**: on a 13.4 app the picker shows, after the count line, a quiet button "Read the generators
  of shop-api" with the line `herd php artisan module:generators --json --no-interaction --ansi` beside it
  (not in the spec). The read's document replaces the shipped catalog for that app.
- **Cache**: in Mortise's data folder, `catalogs/<key>.json` with `{"package": "<raw version>",
  "read_ms": <unix ms>, "generators": [...]}`; `<key>` is the key `History` uses for the app folder.
  It is used while the app's `package.raw` equals `package`; a new version makes it stale and the shipped
  catalog of that version applies until the user reads again. Without a data folder the catalog is kept
  in memory for the session. A damaged file reads as no cache. The cache is written through a temporary
  file beside it, as the history is.
- **Picker**: the count line counts the catalog in use. With a read catalog, the line after it reads
  "Read from shop-api at 14:05." with "Read again" (not in the spec). An app's own generator (any name the
  order table of S9.1 does not place) shows under "Other", as `grouped` already does.
- **Options Mortise does not offer**: `force` (Replace is the way), `dry-run` and `json` (Mortise adds
  them, D5 and D6). `Generator::words` ignores them as it ignores unknown options.
- Every generator in a read catalog is offered, with its form built as deviation S46 says. A generator
  with no `module` argument (an app's own) gets no module word.

### D5. P2: the file list (`generate_view.rs`, task 15.4)

On a 13.4 app, under "Command", "Files this creates" shows (S9.2, S9.4):

- Before a dry run: the button "List files" (not in the spec) with the line of the dry run under it:
  the form's words with `--dry-run --json`. Enter in the form does not start it; Ctrl Enter still runs
  the command itself.
- After it: the tree of the files of the document, grouped by folder. Paths are made relative to the
  module folder of the snapshot for a file generator (`Modules/Billing/` as the root row) and to the app
  folder for `module:make` (`Modules/Loyalty/`). A path outside both is shown as printed. The head reads
  "3 files, none exist yet", "3 files, 1 exists already" or, for `module:make`, "Modules/Loyalty does not
  exist yet" or "Modules/Loyalty exists with 23 files" (from the snapshot). A `delete` entry is the head
  of a Replace, not a row.
- The list belongs to the words it ran with. Any change of the words drops it and brings back "List
  files". The primary button then reads "Create files" again.
- Buttons: with a list and no file kept, "Create 3 files"; for `module:make`, "Create module" stays.
- **S9.3**: when the list has a `keep` file that a model's flag writes (the same name mapping as
  `replace_words`: `<Model>Controller`, `<Model>Request`, `<Model>Resource`, `<Model>Seeder`,
  `<Model>Policy`, the factory), Mortise clears that flag in the form (and `all`, setting the other six
  flags), marks the checkbox "exists" (S9.3), keeps the list with the row "exists", and shows "app/
  Policies/PaymentIntentPolicy.php exists. A generator does not write over a file, so the policy was taken
  out of the command. Create it afterwards with "Replace" if you want the generated one." The list stays
  valid for the new words. The button reads "Create the other 2 files". A migration is never kept (its
  file name carries the time).
- **The generator's own file exists** (a `keep` that no flag writes, such as the model itself or a
  single-file generator): the list shows the row "exists", the sentence "<path> exists. The generator does
  not write over a file. Choose another name, or replace the file." (not in the spec), the primary button
  is off, and "Replace <file>" opens the Replace dialog of S9.6 with the words and `--force`.
- Errors of the document other than `File : ... already exists.` show under the list, quoted.
- A dry run changes nothing in the app. The test that runs it against the fake artisan asserts the app
  folder is unchanged, and the real-PHP test asserts it once on a real app.

### D6. P5: the result of a run (`app.rs`, `output.rs`, task 15.4)

On a 13.4 app a generator run carries `--json` (`Generator::words` stays as it is; the app appends
`--json` through one core function `package::with_json(words)`, which puts it before a `--` if there is
one). The line on screen and the history show it. At the end, `GeneratorRun` document: each file maps to
the existing `GeneratedFile`: `created` to `Created`, `replaced` to `Replaced`, `exists` to `Left`,
`failed` to `Failed`; `deleted` (the folder of `module:make --force`) is not a row. `autoload_failed` is
true when `errors` holds "composer dump-autoload failed.". Paths go through the same relative-path step as
today (the absolute path against the app folder, with `/` as separator on every platform). Without a
document, D2's fallback reads the lines as milestone 14 does; with `--json` the task lines are on
standard error and still in the mixed output.

The rest of S9.5 and S9.6 (toasts, Replace, selection after `module:make`) is unchanged.

### D7. P4: the migration state (`package.rs`, `app.rs`, `actions_view.rs`, `summary.rs`, task 15.5)

```rust
pub struct ModuleMigrations {
    pub ran: Vec<Ran>,          // name and batch, by name
    pub waiting: Vec<String>,   // by name
    pub tables: Vec<String>,    // created by the ran migrations, read from their files (below)
    pub read_ms: i64,
}
pub struct PackageData {
    pub migrations: BTreeMap<String, ModuleMigrations>,  // by module name
    pub migration_table: Option<bool>,
    pub translations: Option<LangReport>,               // D9
}
```

- **Where it lives**: `ReadOptions` gains `package: PackageData`, and the snapshot keeps it
  (`AppSnapshot::package_data`). The app sets it with `LiveApp::set_options`, which reads the app again
  (milliseconds, cached). So every screen and `findings::find` read the same data from the snapshot, and
  no screen keeps its own copy. It is kept in memory only: the database changes outside Mortise, and an
  old state on disk would read as true.
- **Last batch**: the ran migrations of the module whose batch is the highest among them, as the
  package's `Migrator::rollback` takes them.
- **Tables**: `mortise_core::php::created_tables(source)` gives the first argument of each
  `Schema::create('<literal>', ...)` (also `Schema::connection(...)->create`) inside the `up` method,
  only when it is a string literal. The files are found with `output::migration_file`. A migration
  whose file is missing or whose table is not a literal adds nothing. With no table found, the table row
  is left out as today (S42). The list is in migration order, without duplicates.
- **When it is read**: the user starts each read.
  - The state box of Migrate, Roll back, Reset and Refresh (S10.1 to S10.4), and of Migrate all modules,
    shows before a read: "Not read yet." with the button "Read migration state" and its line (not in the
    spec). For a module: `module:migrate-status <Module> --json`; for "All modules": `module:migrate-status
    --all --json`. The result merges into `migrations` by module (a module of the document replaces its
    entry; under `--all`, a module that is on and absent from the document is removed).
  - After a read in this session, a migrate, roll back, reset, refresh or fresh started from Mortise reads
    the state again when it ends, whatever its exit code, with the same target (fresh: `--all`). The state
    box of those panels says so before the run: "After the run Mortise reads the state again:" and the
    line (not in the spec). A dry run (`--pretend`) is not followed by a read. Seed and delete are not:
    delete removes the module's entry.
  - "Read at 14:05" stands in the state box after a read (not in the spec).
- **On screen** with a state for the module:
  - S10.1: "Waiting" with "1 migration" (the names in the box), the button "Migrate (1)", and the row
    detail "1 waiting" ("Nothing waiting" when none, not in the spec).
  - S10.2 c and S10.3: "Last batch" with "1 migration", row detail "last batch, 1", the dialog lists the
    migrations of the last batch and its button reads "Roll back 1 migration".
  - S10.2 f (All modules): "Waiting 1 migration in 1 module", "Migrate (1)", row detail "1 waiting".
  - S10.4 and S10.5: "9 ran" beside Reset and Refresh, "Every table and column created by the 9
    migrations of Checkout that ran", the tables row ("checkouts, carts, cart_items, orders, order_items
    and 2 more", "7 tables"), "7 tables come back empty", the code box of tables in the dialogs, "7
    tables. Nothing outside Checkout is touched.", "Reset 9 migrations", "Refresh 9 migrations".
  - S4.3: the Migrations line of "Module state" reads "9 ran" and "1 waiting", and the header button
    "Migrate (1)". Without a state: "9 files" and "Migrate" as today.
  - Without a migrations table (`migration_table: false`): "The database has no migrations table yet."
    (not in the spec) in the state box; every migration counts as waiting.
- The toasts of milestone 12 and 13 keep reading their counts from the output.

### D8. P7: Run tests (`catalog/actions.rs`, `actions_view.rs`, task 15.6)

- `Context` gains `package_data: bool` (the gate). With it, `Action::RunTests` plans
  `module:test <Module> --json` or `module:test --all --json`, with the options `--filter=<pattern>`
  (text, "Filter, a test name or pattern", not in the spec) and `--parallel` ("Run in parallel",
  not in the spec), the command's own options. No question: it reads and reports (S10.3). Without the
  gate the placeholder of S10.2 e and deviation S39 stay.
- The panel's text names the folder: "31 tests in Modules/Checkout/tests." (S10.2 e, without the
  sentence about the placeholder). The row detail keeps the count of test files.
- It runs in the drawer like any action. At the end the toast reads "31 tests passed in Checkout." from
  the document (S10.2, C13), "3 of 31 tests failed in Checkout." on a failure (not in the spec), and for
  "All modules" "307 tests passed in 7 modules." (not in the spec). A module with `ran: false` gives
  "Checkout has no tests folder." (not in the spec). Without counts the toasts of milestone 12 apply.
- The header button Run tests of S4.3 opens the panel, as today (S40).

### D9. P8: translations in Findings (`findings.rs`, `findings_view.rs`, task 15.7)

- On a 13.4 app Check translations runs `module:lang <Module> --json` or `module:lang --all --json`
  (`plan` with the gate). It runs in the drawer. At its end the document becomes `LangReport` and goes
  into `PackageData::translations`: a module of the document replaces its entry, so a check of one module
  keeps the others.
- `findings::find` gains a seventh check, `Check::TranslationsComplete` ("Translations are complete",
  with `module:lang` as its source, S6.4), present only when a report is there. For each module and
  locale with missing keys or files: a warning, `Detail::MissingTranslations { locale, keys, files,
  from }`. The sentence is the spec's "Checkout: 4 keys in en are missing in fr." when the module has two
  locales (`from` is the other one); with more locales "Checkout: 4 keys are missing in fr." (not in the
  spec); missing files add "and 1 file" (not in the spec). The quiet row reads "5 modules with lang
  files" from the modules whose `checked` is true. A warning changes the module's mark and counts like
  any warning; it opens the Actions tab on Check translations.
- **To do** (S6.4): a group "To do, not counted as findings" after Warnings, one row per module with
  waiting migrations from the P4 state: "Checkout has 1 migration waiting." with "Migrations" and
  "Actions" (opens Migrate). It never changes a mark or a count. `Findings` gets `todo: Vec<Todo>`,
  computed in `find` from `PackageData`.
- The report and the to do list are as old as their read; Findings shows no age (not in the spec).

### D10. Recordings and the fake artisan (task 15.2)

Recorded once on a scratch app with `nwidart/laravel-modules` 13.4.0 (PHP 8.4, Composer, SQLite), as the
README of `fixtures/output/` says, with `made_with` naming 13.4.0, `args` unique across the folder:

- `module:generators --json`;
- `module:make-model PaymentIntent Billing --migration --policy --fillable=order_id,amount,currency,status
  --dry-run --json` with nothing there, and the same with `PaymentIntentPolicy.php` there (S9.3);
- a P5 run that creates (the same model words with `--json`, different `args`), one that leaves a file,
  and `module:make Loyalty --api --dry-run --json` and `--json`;
- `module:migrate-status Checkout --json` with one migration waiting, `--all --json`, and one without a
  migrations table;
- `module:test Checkout --json` passing and one failing, `module:test --all --json`;
- `module:lang Checkout --json` with keys missing, and `--all --json`.

Each recording keeps a new key `root`: the scratch app folder as the package printed it. The fake artisan
replaces `root` in standard output and standard error with its working folder, written with `/` (JSON
paths use `/`), so a test app's paths match. The size of the P3 line is noted in the README (D2).
The fixture app `fixtures/shop-api` stays at 13.3.0; a test that needs 13.4 copies it and changes the
version in its `composer.lock` (a helper in the tests that already copy it).

`tests/real_php.rs` (ignored) adds: one dry run that lists files and leaves the app unchanged (a hash of
the module folder before and after), one P5 run read from its document, `module:migrate-status --json`
before and after a migrate, `module:test` on a module with one test, `module:lang --json` with one key
missing. The CI job's constraint `^13.3` already resolves to 13.4.0.

### D11. Interface text

Every sentence above is in `strings.rs`, marked "not in the spec" where it is. Each new control has an
accessible name and a 24 px target (`tests/accessibility.rs`).

### Changed while building 15.3a

- The ran migration type is `RanMigration`: `Ran` is already the name of a recent run (`run::Ran`).
- `migration_file` moves from `output` to `package`. The read of an app needs it, and that read
  does not depend on the `run` feature.
- `package::with_dry_run(words)` adds `--dry-run --json`, beside `with_json`.
- `Read` also has `wait(deadline)` and `stop()`, so `on_exit` can stop a read as it stops a run.
  It keeps no `cut` flag: a cut document does not parse, so it reads as `Malformed`.
- `ReadEnd::error_line()` gives the last non-empty line of standard error as plain text. The app
  writes "exit code N" when there is none, since that text belongs in `strings.rs`.
- `LangReport` is both the document of P8 and the report in `PackageData::translations`. A merge
  keeps its modules sorted by name.
- `AppRuns::ended()` gives the end, the mixed output and the standard output lines.
- The P3 line is 48552 bytes, under `KEEP`, so the line limits stay as they are.
- `OpenApp` keeps the app's `PackageData` and passes it to every read (`set_package`), so a change of
  settings does not drop it.
- The app's reads: `OpenApp::reading` holds at most one `PackageRead { read, label, purpose: ReadFor }`,
  polled each frame; `start_read` starts one and `read_ended` takes its end (a start error ends it at
  once). `OpenApp::busy()` gives the label of a run or a read; every run button uses it.

### Changed while building 15.5

- `package::migrate_status_args(target)` gives the words of a state read, with a module name that
  starts with `-` after `--`. The panel shows them and the app runs them.
- The button that reads ("Read migration state", then "Read again" with "Read at 14:05") and its line
  stand under the state box. After a read in this session the line has "After the run Mortise reads the
  state again:" above it, so the one line serves both. Fresh has no box: after a read its panel shows
  that sentence and the line under "What is lost".
- "A read in this session" is `PackageData::migration_table` being set. The app looks at the words of
  every run that ends (`AppRuns::args`), so Run again from the drawer is followed by a read too. A delete
  forgets the module's state only when it exits with 0.
- The box of Reset and Refresh is titled "Ran" and lists the migrations that ran (not in the spec).
  Without a waiting migration the button reads "Migrate", and the box "Nothing waiting"; without a ran
  one the box reads "Nothing ran" (not in the spec).
- Under "All modules" the state is that of the modules that are on and were read. The box shows
  "Reading" while a read of its target or of "All modules" runs.
- The roll back dialog keeps its dry run on the left and lists the last batch in its code box. The
  tables row names five tables and then "and 2 more"; the dialogs list them all.
- S4.3 reads "9 ran" alone when nothing waits.

### Changed while building 15.6

- `Action::options(target, package_data)` takes the gate: Run tests has no options to draw
  before 13.4. `Action::artisan()` still gives none for Run tests; `Action::from_artisan`
  knows `module:test`, so a test run in the history is found again.
- "All modules" on 13.4 counts the tests of every module and reads "307 tests in every module,
  on or off." (not in the spec): `module:test --all` takes every module, on or off. The row
  detail keeps the count of the modules that are on.
- The kind of the Run tests toast follows the exit code, as for every action: a failed run gets
  its toast, "3 of 31 tests failed in Checkout.", only when the drawer does not show it. A run
  with exit 0 and no counts gets no toast, as Run tests had none in milestone 12.
- The field "Filter, a test name or pattern" shows `--filter` beside its label in dim mono,
  as a checkbox shows its option.

### Changed while building 15.7

- `findings::Subject` takes `package: &PackageData`. The translation check and the to do list skip
  modules of the report or the state that are not in the app; the check counts only those that are.
- A row of Findings opens a module's action through `BoundaryAction::OpenModuleAction { module,
  action }`: `OpenAction` takes the selected module, and Findings has "All modules" selected.
- In the palette a translation warning opens Check translations of its module, with "Actions".
- In the alert of a module's Summary the translation clause drops the module in front:
  "... , and 2 keys in en are missing in fr." (not in the spec).
- The quiet row draws `module:lang` dim after "Translations are complete", as S6.4 does.
- With no finding and some work to do, the To do group shows below "No findings. ...".

## Risks / Trade-offs

- [The user must press "List files" and "Read migration state"] → One more step than the spec draws.
  It follows the rule of `AGENTS.md`; the owner can change the rule later. Deviation rows record it.
- [The migration state goes stale when the database changes outside Mortise] → It is read again after
  each migration command Mortise runs, and "Read at" shows its age.
- [Tables come from the migration files] → A table built from a variable is not named. The panel then
  names fewer tables or none; the dry run of reset still prints the SQL.
- [A long P3 line] → Measured in 15.2; the line limit of standard output grows only if needed.
- [A dev version of the package does not get the data] → It has the features or not; Mortise cannot
  tell without running a command. It keeps the 13.3 behaviour.

## Migration Plan

No change to `settings.json` or the history. A new folder `catalogs/` in Mortise's data folder. Rollback
is a revert; the folder is then ignored.

Order: 15.1 (this design), 15.2 (recordings), 15.3a (core groundwork: gate, documents, reads, stdout,
`PackageData` in the snapshot). Then 15.3, 15.5, 15.6 and 15.7 in parallel; then 15.4; then 15.8.
Two pull requests to `main`: the first with 15.1 to 15.4 (generate data: P2, P3, P5) and their images and
deviations, the second with 15.5 to 15.8 (action data: P4, P7, P8). The change is archived after the
second.
