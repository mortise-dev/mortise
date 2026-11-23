# Design

## Context

See `proposal.md` for why. Milestone 11 (`openspec/changes/archive/2026-11-23-phase-2-runner-drawer/design.md`)
built `Runs`, `AppRuns::start`, the drawer, the history, the stop dialog and the fake artisan with the
recordings of `fixtures/output/`. Milestone 10 reads `APP_ENV` into `AppSnapshot::app_env`.

What exists in the app: the Actions tab draws a placeholder (`workspace.rs`), the header of a module has
"Open in editor" only, `Dialog` (C14) has a title, a body, a confirm and a cancel button, and `Toast`
(C13) has no action. `AppSnapshot::used_by(name)` gives the links into a module.

Package facts, read in `laravel-modules` at `5b0307af` and at the tag `v13.3.0` (source read, not run):

- `BaseCommand` adds `--all` and the `module` argument to every module command, so `--all` works for
  `module:migrate`, `module:seed`, `module:lang`, `module:disable` and `module:enable`.
- `module:migrate-rollback`, `-reset`, `-refresh`, `-fresh` and `module:delete` implement
  `ConfirmableCommand`. They call `confirmToProceed`, which asks in production only, except
  `module:delete`, whose callback is `true`: it always asks. Under `--no-interaction` the question
  answers no and the command fails, unless `--force` is given.
- `module:migrate` and `module:seed` pass `--force` on to Laravel's `migrate` and `db:seed`, which ask
  in production.
- Options: `module:migrate` `--pretend --seed --database --subpath --direction --force`;
  `module:migrate-rollback` `--pretend --database --subpath --direction --force`; `module:seed`
  `--class --database --force`. `module:enable`, `module:disable`, `module:lang` have none that the
  spec draws.

Constraints: `unsafe_code = "forbid"`; `mortise-core` has no interface dependency; macOS, Windows, Linux.

## Goals / Non-Goals

**Goals**: the actions of S10.1 and S10.2 a to f, the confirmation table and the production rule in the
core, the dialogs of S10.3, the header buttons of S4.3, the palette rows, the toasts, the sentence of
S11.1 c.

**Non-Goals**: the four destructive actions on screen, the closed group, the typed name (milestone 13).
The core already knows their rules, so milestone 13 adds screens only. P4 and P7 data (milestone 15).
Recent actions in the empty palette (S12.2 a): left out until a later milestone.

## Decisions

### D1. Actions as data (`catalog/actions.rs`)

The plan puts actions and generators in `catalog/`. This milestone adds `catalog/mod.rs` and
`catalog/actions.rs`; milestone 14 adds the generators beside them. No feature gate: it is plain data
and builds no process. Public surface:

```rust
pub enum Action {
    TurnOn, TurnOff, Migrate, RollBack, Seed, RunTests, CheckTranslations,
    ResetMigrations, RefreshMigrations, FreshMigrate, DeleteModule,   // screens in milestone 13
}
pub enum Target { Module(String), All }
#[derive(Default)]
pub struct Options { pub pretend: bool, pub seed: bool, pub database: String, pub class: String }
pub struct Context<'a> {
    pub production: bool,          // AppSnapshot::app_env == Some("production"), exact
    pub users_on: &'a [String],    // modules that are on and use the target module
}
pub enum Ask { TurnOff, RollBack, Production, NamingTables, TypeAppName, TypeModuleName }
pub enum Plan {
    Run { args: Vec<String> },                       // runs at once
    Ask { ask: Ask, args: Vec<String> },             // the dialog first; args already hold --force
    Refused { args: Vec<String> },                   // production: copy only
    NoCommand,                                       // Run tests until P7
}
impl Action {
    pub fn for_module(on: bool) -> [Action; 6];      // S10.1 order; TurnOn when off
    pub fn for_all() -> [Action; 4];                 // Migrate, Seed, RunTests, CheckTranslations
    pub fn artisan(self) -> Option<&'static str>;    // "module:migrate"; None for RunTests
    pub fn options(self, target: &Target) -> OptionSet; // which of pretend, seed, database, class apply
}
pub fn plan(action: Action, target: &Target, options: &Options, context: &Context) -> Plan;
```

`plan` builds the words in a fixed order: the command, the module name or `--all`, then the options
in the order of `OptionSet` (`--pretend`, `--seed`, `--database=<value>`, `--class=<value>`), then
`--force`. A text option is trimmed; an empty one is left out; each is one word, `--database=<value>`,
so nothing is split or read by a shell. An option that the action does not take is ignored.

`OptionSet`, from the drawings of S10.2: Migrate pretend, seed, database; Roll back pretend (database is
in the command but not drawn); Seed class, database; the "All modules" Migrate pretend, seed; Reset
pretend, database; Refresh seed, database; Fresh seed, database and the two drop flags of milestone 13.
The drop flags are added with their screens.

Alternative: one enum per command with its own option struct. Rejected: the panels share four options,
and one `Options` keeps the form state in the app simple.

### D2. The confirmation table and the production rule (`plan`)

| Action | Asks | Package confirms | Production |
|---|---|---|---|
| Turn on | never | no | runs |
| Turn off | when `users_on` is not empty | no | runs |
| Migrate, Seed | in production | in production | asks, `--force` |
| Roll back | always; a dry run asks only in production | in production | asks, `--force` |
| Run tests | never (`NoCommand` until P7) | no | — |
| Check translations | never | no | runs |
| Reset, Refresh | always, `NamingTables` | in production | `Refused` |
| Fresh | always, `TypeAppName` | in production | `Refused` |
| Delete | always, `TypeModuleName` | always | `Refused` |

`--force` is added when the package confirms and `plan` returns `Ask`. Since every row that the package
confirms also asks in the same case, the rule "`--force` only after Mortise asked" holds by
construction, and one test checks it over every action, both values of `production` and both values of
`pretend`. A migrate with `--pretend` in production still asks, because Laravel's `migrate` asks before
it reads `--pretend` (Laravel source as remembered, not read; the safe side either way).

Roll back with `--pretend` outside production does not ask: the dry run changes nothing, and S10.3 offers
the dry run as the way out of the dialog.

`stop_asks_first` stays in `run/process.rs`: it is about a running process, not an action.

### D3. Reading the output (`output.rs`)

```rust
pub struct FailedMigration { pub module: Option<String>, pub migration: String }
pub fn failed_migration(lines: &[String]) -> Option<FailedMigration>;
pub fn migrations_done(lines: &[String]) -> usize;
pub fn migration_file(module_dir: &Path, migration: &str) -> Option<PathBuf>;
```

Each line is read through `ansi::plain`. A migration line starts, after spaces and an optional
`Rollback: ` (the rollback's task line, `MigrateRollbackCommand.php:59`), with a name of the form
`dddd_dd_dd_dddddd_` and ends with `DONE` or `FAIL` (Laravel's task component). `failed_migration`
takes the first `FAIL` line; its module is the name in the last `Running Migration <Name> Module` line
before it. `migrations_done` counts the `DONE` lines; it serves the toasts of migrate and roll back.
`migration_file` returns `<module_dir>/database/migrations/<migration>.php` when that file exists, the
package's default path (`kind.rs` uses the same folder). A module with another path gets no "Open
migration file". Without P5 this reading breaks when Laravel changes the words; the real-PHP job of
milestone 16 is the guard (plan, section 7).

### D4. The Actions tab (`actions_view.rs`)

State, per open app, in memory: the selected action, `Options`, and the module they belong to. A change
of module or of app resets the options and selects Migrate (S10.1). The left list is 250 px; each row has
the action's name and a detail on the right:

| Row | Detail (without P4) |
|---|---|
| Turn off / Turn on | good mark "On" / "Off" |
| Migrate | the count of files of the kind Migrations, "9 files" (the S4.3 fallback) |
| Roll back | none |
| Seed | the count of the kind Seeders, "1 seeder" |
| Run tests | `module.tests`, "31 tests" |
| Check translations | none |

For "All modules" the details are sums over the modules that are on. Deviation S41 records these
fallbacks and the missing "Waiting" and "Last batch" boxes.

The panel: title and sentence of S10.2, the options as checkboxes and text fields, the line under
"Command" in the mono code style, then the run button (primary, or red outline for Roll back) with its
keycap, and "Copy command". The line is `line_now(runner, args)` of the drawer, the same text as the
drawer's `$` line: the runner's prefix and `--no-interaction --ansi`. Deviation S37. With no runner
chosen, the line reads `php artisan <words>` and the run button is off with the tooltip "Choose how PHP
runs for <app> in Settings." (not in the spec).

Run: `plan` with the current options. `Run` starts `args` through `AppRuns::start`. `Ask` opens the
dialog of D5; Confirm starts the same `args`. `Refused` cannot happen for the six everyday actions.
While `Runs::running(app)` is `Some(label)` every run button is off with "<label> is running".

Roll back's "Dry run" button and the dialog's "Dry run" check `pretend` on the panel and run nothing;
the line on screen then ends with `--pretend` and Ctrl Enter runs it. Deviation S38: S10.2 c draws a
button that runs. The same holds for "Dry run" in the drawer (D7).

Run tests: S10.2 e with the placeholder line in dim, the run and copy buttons off with the tooltip "The
package has no command for the tests of one module yet." (not in the spec). Deviation S39.

Keys, where nothing is typed: J and K move through the actions, Ctrl Enter (Cmd Enter) runs. Tab
follows the egui order: list, options, buttons.

### D5. Dialogs

`Dialog` gets two fields: `danger: bool` (the confirm button is the filled red one of C14) and
`left: Option<String>` (a quiet button on the left that answers `Answer::Left`). The dialogs of the
actions start on Cancel.

| Ask | Title | Body | Confirm | Left |
|---|---|---|---|---|
| RollBack | "Roll back Checkout?" | `c.rb.p`; in production also the production sentence | "Roll back" (no count without P4) | "Dry run" |
| TurnOff | "Turn off Checkout?" | "Shipping and Notifications use it. They stay on and can fail where they call Checkout." | "Turn off Checkout" | — |
| Production, migrate | "Migrate Checkout in production?" | "shop-api reports the environment production. The command runs with --force on that database." | "Migrate in production" | — |
| Production, seed | "Seed Checkout in production?" (not in the spec) | the same sentence | "Seed in production" (not in the spec) | — |

For "All modules" the object is "all modules". Roll back and the two production dialogs are `danger`.
The panel of Turn off shows the alert of S10.2 a when `users_on` is not empty.

### D6. Toasts

`Toast` gets `action: Option<(String, ToastAction)>`; the failed toast of a run uses
`ToastAction::ShowOutput(app index)`, which selects that app and opens the drawer.

The app keeps, beside the run it started from the Actions tab, the `Action` and `Target`. When `poll`
sees the end:

- Code 0, not a dry run: the done toast of C13: "Turned off Checkout.", "Turned on Checkout.",
  "Migrated Checkout: 1 migration ran." (`migrations_done`; 0 reads "Migrated Checkout: nothing to
  migrate.", not in the spec), "Rolled back Checkout: 1 migration.", "Seeded Checkout.". Check
  translations gets none: its output is the result. For "All modules" the object is "all modules".
- Failed or stopped, while the drawer is not `Open` or another app is on screen: the failed toast
  "<label> stopped with exit code <n>." with "Show output". On screen, the drawer already says it (S9.6).

### D7. The failure sentence of S11.1 c

When a run whose first word is `module:migrate` fails and `failed_migration` finds a line, the drawer
replaces the sentence of milestone 11 with `x.run` for that label, code and migration, and the steps
"Open migration file" (keycap Enter, only when `migration_file` finds it), "Dry run" and "Copy output".
The module is `failed.module`, else the module name in the run's `args`. "Open migration file" opens
the file through `open_in_editor` at line 1. "Dry run" selects that module (or "All modules" for
`--all`), opens the Actions tab on Migrate and checks `pretend`. Both are new `DrawerAction`s handled
in `app.rs`.

### D8. Header and palette

The header of a module gets "Run tests" and "Migrate" (primary) before "Open in editor", as S4.3 draws
them. Both open the Actions tab with that action selected. Deviation S40 covers them and the palette: S10.1 says the
header Migrate runs the command, but a command runs only with its exact line on screen first.

The palette gets the group "Actions for <module>" after the modules of the app, as S12.1 orders it.
The module is the best module match of the query; with no module match, the module selected in the
index; with "All modules" selected, the actions of "All modules". Each row reads "Migrate **Checkout**"
with the artisan words in dim mono (`module:migrate Checkout`), the hint "Action", and at most four rows,
the best matches of the query against the label, the module and the artisan name. A row opens the
Actions tab on its action (D4), as the header does. New `Command::Action { module: Option<String>,
action: Action }`.

### D9. Recordings and tests

New recordings in `fixtures/output/`, made as in milestone 11 from a scratch app with 13.3.0 and SQLite,
with modules named like the fixture app: `module:disable Checkout`, `module:enable Checkout`,
`module:migrate Checkout` (one migration done), `module:migrate Shipping` failing at a migration
(a duplicate column), `module:migrate-rollback Checkout`, `module:lang Checkout`.

Core tests, each rule once: `plan` over the table of D2 in one table test, the `--force` invariant over
every action, the word order and the trimmed options, `for_module` with on and off; `failed_migration`,
`migrations_done` and `migration_file` on the new recordings.

Interface, `tests/actions.rs` against the fake: Migrate Checkout runs and the toast counts one migration;
a dry run line; Turn off with a user asks and names it; production asks and the line holds `--force`;
Run tests is off; the header Migrate opens the panel; the failing migration names the migration in the
drawer. The palette row is tested in `tests/palette.rs` through `entries`. Images: S10.1, S10.2 a to f,
S10.3, light and dark; S11.1 c again with the sentence.

## Risks / Trade-offs

- [Laravel changes the words of its task lines] → `output.rs` reads them without P5. The real-PHP job of
  milestone 16 catches it.
- [A custom migration path] → No "Open migration file"; the sentence still names the migration.
- [The header and the palette do not run] → One more key, Ctrl Enter, on a panel that shows the line.
  Recorded as S40 so the owner can overturn it.
- [`APP_ENV` from `.env` only] → A cached config or a real environment variable can differ. The package
  then still asks under `--no-interaction` and the command fails with its own message, which the drawer
  quotes.

## Migration Plan

No change to `settings.json` or to the history. Rollback is a revert.

Order: D1 to D3 in the core with the recordings (task 12.1), then the Actions tab with the dialogs,
toasts and drawer sentence (12.2) and the palette (12.3) in parallel.
