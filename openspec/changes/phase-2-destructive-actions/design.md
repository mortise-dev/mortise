# Design

## Context

See `proposal.md` for why. Milestone 12 (`openspec/changes/archive/2026-11-23-phase-2-everyday-actions/design.md`)
built `catalog/actions.rs` with `Action::{ResetMigrations, RefreshMigrations, FreshMigrate, DeleteModule}`,
the asks `NamingTables`, `TypeAppName`, `TypeModuleName`, the production refusal (`Plan::Refused`),
`asks_again`, the Actions tab (`actions_view.rs`), `Dialog::{danger, left, start_on_cancel}`, the toasts
and the drawer sentence of S11.1 c. `App::act` returns early for the three destructive asks: nothing on
screen offers them yet.

Package facts, read in `laravel-modules` at the tag `v13.3.0` and at `master` `5b0307af` (source read, not run):

- The four implement `ConfirmableCommand`. `BaseCommand::canProceed` first calls `isProhibited()`, then
  `confirmToProceed`. `module:delete` always confirms (callback `true`); the other three in production.
- `isProhibited()` is Laravel's `Illuminate\Console\Prohibitable`. It prints, through the warn component,
  `This command is prohibited from running in this environment.` and the command exits with 1.
- 13.3.0: `Module::prohibitDestructiveCommands()` prohibits fresh, refresh and reset. The flag lives on
  `BaseCommand`, so `MigrateFreshCommand::prohibit()` alone prohibits every module command that confirms.
  `master`: each of the five (with rollback and delete) has its own flag, and the facade sets all five.
- `module:migrate-reset` options `--pretend --database --subpath --direction`. With `--pretend` it calls
  `Migrator::reset(pretend: true)`, which prints SQL and changes nothing.
- `module:migrate-refresh` options `--database --seed`; it calls reset, migrate and, with `--seed`, seed.
- `module:migrate-fresh` options `--database --seed --drop-views --drop-types`; it refuses `--pretend`.
  It runs `db:wipe` on the whole database.
- `module:delete` deletes the module's folder and its line in the statuses file. The database stays.
- `BaseCommand` adds `--all` to every module command, so `module:delete --all` would delete every module.

## Goals / Non-Goals

**Goals**: the screens S10.4 a to d and S10.5 a to d, the five ways of keeping the four apart, the typed
name, the count of files that are not committed, the refusal when the app prohibits a command, the toasts,
the selection after a delete, Run again of the four.

**Non-Goals**: table names and migration counts (P4, milestone 15). Knowing before a run whether the app
prohibits a command (D4). Rollback stays an everyday action with its own dialog (S10.3).

## Decisions

### D1. Files that are not committed (`git.rs`, task 13.1)

```rust
/// For each folder in `dirs`, the files git cannot restore: changed, staged, added,
/// deleted and untracked. Ignored files are left out. `None` for a folder git
/// cannot tell about.
pub(crate) fn uncommitted(root: &Path, dirs: &[PathBuf]) -> Vec<Option<u32>>;
```

The top of the repository comes from `git rev-parse --show-toplevel`. Each module folder is made
relative to it, after both are resolved (`canonicalize`), so a module outside the app folder is found
too. One call from the top: `git status --porcelain=v1 -z --untracked-files=all --no-renames -- <folders>`,
through the existing `git()` helper (no lock, no prompt, no file system monitor, standard error
dropped). Each NUL-separated entry is `XY <path>` from the top; it counts once for the folder it is
under. A folder with no entry gets 0.

`None` when git cannot tell: a folder outside the repository, a folder in a repository of its own
(a `.git` in the folder or above it below the top, such as a submodule, whose files the outer status
does not list even with `ignore = all`), a folder that git lists as one entry, or when git fails.

`Module` gets `pub uncommitted: Option<u32>`, and the screen says "Mortise cannot tell whether these
files are in git." for `None`. It is read in `app.rs` with the churn, in the background thread that
reads the snapshot. The watcher already reads the app again when a file changes or a commit lands
(`.git/logs/HEAD`), so the count follows.

Alternative: run git when the delete panel opens. Rejected: it would start a process on the interface
thread every frame or need its own cache; the snapshot already has both.

### D2. The destructive rules (`catalog/actions.rs`, task 13.2)

- `Action::destructive(target: &Target) -> &'static [Action]`: `[ResetMigrations, RefreshMigrations,
  DeleteModule]` for a module, `[FreshMigrate]` for all. `Action::is_destructive(self) -> bool`.
- `plan` returns `Plan::NoCommand` for reset, refresh or delete on `Target::All`, and for fresh on a
  module. `module:delete --all` would delete every module, and a fresh of one module leaves the other
  modules without tables (S10.4 "Decision"). The lists are the rule on screen; this makes the core hold it too.
- `Options` gets `drop_views: bool` and `drop_types: bool`; `OptionSet` gets the same two, true for fresh
  only. Word order: the command, the module or `--all`, `--pretend`, `--seed`, `--drop-views`,
  `--drop-types`, `--database=`, `--class=`, then `--force`, then `-- <name>` for a name that starts with `-`.
- A reset with `--pretend` outside production is `Plan::Run`: it changes nothing, as for roll back.
  In production it stays `Refused`, as S10.4 d draws it. `asks_again` gives false for
  `module:migrate-reset … --pretend`.
- The typed name:

  ```rust
  impl Ask {
      /// The name the user types to confirm, if this question asks for one:
      /// the app for `TypeAppName`, the module for `TypeModuleName`.
      pub fn name_to_type<'a>(self, app: &'a str, target: &'a Target) -> Option<&'a str>;
  }
  /// True when `typed` is `name` exactly: same case, no space trimmed.
  pub fn typed_name_matches(typed: &str, name: &str) -> bool;
  ```

  The app name is the display name of Settings (the one in the dialog title). Exact match is the spec
  ("The name must match exactly"). An empty name never matches.
- `Action::from_artisan(word) -> Option<Action>`, the inverse of `artisan`, and
  `Options::from_args(args) -> Options`, the inverse of the option words, for Run again (D7).
- `refuses(action, context) -> bool`: the four in production, and delete when `Context::
  modules_folder_assumed` is true. Mortise then could not read `paths.modules` from
  `config/modules.php` and shows the package default, while the package may delete another folder
  of the same name. `plan` gives `Refused` for both. The layout reads a key as assumed also when a
  part of its path only PHP knows (`'paths' => $paths`, or an array with a spread), not only when the
  key's own value does (`ConfigValue::lookup`).

Tests, once each, in `catalog/actions.rs`: the destructive lists; the four rows of the plan table for
targets they do not take; the reset dry run row; the word order with both drop flags; `asks_again` of a
reset dry run; `name_to_type` and `typed_name_matches` (exact, case, trailing space, empty).
The `--force` invariant test of milestone 12 already covers every action.

### D3. Reading the output (`output.rs`, task 13.2)

- `pub fn prohibited(lines: &[String]) -> bool`: a line, through `ansi::plain` and trimmed, ends with
  `This command is prohibited from running in this environment.` Laravel's warn component puts ` WARN `
  before it. The words are Laravel's; the real-PHP job of milestone 16 guards them.
- `migrations_done` is split so a refresh can count what ran again and a reset what rolled back:

  ```rust
  pub struct MigrationCount { pub ran: usize, pub rolled_back: usize }
  pub fn migration_count(lines: &[String]) -> MigrationCount;
  ```

  `ran` counts migration lines that end with `DONE` without `Rollback:`; `rolled_back` counts the
  migrations rolled back, each name once (a reset can print a task line and a `Rollback: <name>` line for
  the same migration: read the recordings and count what they show). Migrate and roll back toasts move to
  the two fields. The recordings decide the exact shapes; the tests run on them.

### D4. Knowing that the app prohibits a command

Mortise learns it from the run the user started, and does not read the app's files for it. The call is
`Module::prohibitDestructiveCommands($this->app->isProduction())` or a command's `prohibit(...)`, in any
service provider, often behind a condition. Its argument is evaluated when the app boots: from the files
Mortise could only find the call, not its value, and a wrong "prohibited" or "allowed" on screen is worse
than none. Running PHP to ask would run app code that the user did not start, which `AGENTS.md` forbids.
The spec says the same: "Mortise cannot know before running; it shows the refusal".

When a run fails and `prohibited(lines)` is true, the drawer replaces its sentence with "The app prohibits
this command, so the package refused to run it. Its message is quoted above." and offers "Copy output".
It wins over the failed-migration sentence. It applies to any command, since rollback can be prohibited too.

### D5. The closed group and the panels (`actions_view.rs`, task 13.3)

`ActionsState` gets `destructive_open: bool`. `follow` closes it when the target changes or the tab is
not shown; a destructive action that was selected then goes back to Migrate. `ActionsState::on(target,
action)` opens it when `action` is destructive (palette, Run again in production).

The list: the everyday rows, then the group row "Destructive" (detail "open" or "closed", the second not
in the spec), then, when open, the rows of `Action::destructive(target)`, then for a module the dim line
"Dropping all tables is under "All modules"." under the list. The group row is one stop of `ListNav`, so
J and K stop at it. Enter or a click opens or closes it. Row details: Delete module the module's file
count; Reset and Refresh none (the "9 ran" of S10.4 needs P4).

The panel of each, from S10.4: title, sentence, the box "What is lost" with "cannot be undone" on the
right, then the options, the line under "Command", then the run button (`ButtonKind::Danger`, the red
outline), "Dry run" for reset (checks `pretend`, runs nothing: deviation S38), "Copy command".

| Action | Rows of "What is lost" without P4 |
|---|---|
| Reset | "Every table and column created by the migrations of Checkout that ran" / "with all rows" |
| Refresh | "Every row in the tables of Checkout" / "come back empty" |
| Fresh | "Every table of the database, also those that no module owns" / "with all rows"; "Views, if chosen below"; "Types, if chosen below" / "Postgres only" |
| Delete | "The folder Modules/Checkout" / "84 files, 6,412 lines"; "Changes that are not committed" / "12 files" (left out at 0; without git "Mortise cannot tell whether these files are in git."); "Its line in modules_statuses.json" (the statuses file of the layout) |

Delete adds under the box: "Stays: the tables of Checkout in the database and the record of its
migrations." and, when modules that are on use it, "Shipping and Notifications use Checkout and will break."

Refused (`Plan::Refused`): the alert box with the reason under the panel's sentence, the run button
off with the same sentence as tooltip, Copy command on. The reason is the production sentence, or for
delete with the modules folder assumed "Mortise cannot read where shop-api keeps its modules, so the
package could delete another folder. Copy the command and run it where you can check the folder."
(not in the spec).

Ctrl Enter takes the line only when its whole text was inside the visible part of the tab in the last
frame: a line cut off by the scroll area does not count as on screen.

Ctrl Enter on a destructive panel does what the run button does: it opens the dialog. The only line that
runs at once is a reset dry run, which changes nothing.

### D6. The strongest dialog (`dialog.rs`, `app.rs`, task 13.3)

`Dialog` gets three optional parts, drawn between the body and the footer in this order:

- `lines: Vec<String>`: extra paragraphs, such as the uncommitted line of delete, "Nothing outside
  Checkout is touched." of reset, "This command has no dry run." of refresh.
- `code: Option<String>`: the command line in the mono code style (fresh, S10.5 c). It is the line the
  dialog runs (`line_now`, deviation S37).
- `typed: Option<Typed>` with `name: String` and `text: String`: the label "Type <name> to confirm" and
  a single-line field. The confirm button is off until `typed_name_matches(text, name)`.

`show` takes `&mut self` for the field. Focus starts on Cancel in every one of the four (S10.5 c: "as it
opens, focus on Cancel, the red button disabled"). Enter in the field does not confirm; Esc cancels, also
while the field has focus. `danger` is set on all four.

`App::act` builds the four from `Ask` and the snapshot (titles, bodies and buttons of S10.5 with the P4
fallbacks: no table list, "Reset migrations" and "Refresh migrations" without a count). Reset's
"Dry run" on the left checks `pretend` on the panel and closes, as roll back does. The app name is
`display_name()`. The module count of fresh is the modules that are on.

### D7. Run again, the toasts and the delete

- Run again (`App::run_again`): when the first word is one of the four, `Action::from_artisan` and the
  next word (`--all`, or the module, or the word after `--`) give the action and target, and the app
  opens the strongest dialog of D6 for those exact words. When Mortise now refuses the action (D2), it
  opens the Actions tab on that action with the options of those words (`Options::from_args`) and runs
  nothing. Other lines keep the plain dialog of milestone 12.
- Confirm checks the refusal again, from the app's files: the snapshot can be a moment old when the
  app turns to production while a dialog is open. `read_app_env` and, for a delete, `read_layout` are
  read at that moment; a modules folder that differs from the snapshot's counts as assumed. Without a
  snapshot nothing runs. A refused action then opens its panel as above instead of running. "Dry run" of a dialog sets the panel's
  options from the words the dialog asked about, then checks `pretend`.
- A danger dialog drops every Enter key event at the start of its frame, with any modifier and
  wherever focus is: egui presses a focused button on Enter whatever the modifiers are, and Tab can
  move focus to the red button in the same frame. Space and a click still press it; Esc cancels.
- Toasts in `action_ended`, code 0, not a dry run: "Reset the migrations of Checkout: 9 rolled back.",
  "Refreshed the migrations of Checkout: 9 ran again.", "Dropped all tables of shop-api and migrated 7
  modules.", "Deleted the module Checkout.". With a count of 0: "…: nothing to roll back." and "…:
  nothing ran again." (not in the spec). The fresh count is the modules that were on when it started.
- After a delete ends with code 0, the app's selection becomes "All modules" (`Selection::All`); the tab
  stays. The watcher reads the app again and the module leaves the index.

### D8. The palette (`palette.rs`, task 13.3)

The group "Actions for <module>" gets the destructive rows only when a word of the query, lower case, is
`reset`, `refresh`, `fresh` or `delete`. Words, not substrings: "refresh" does not list fresh. Reset,
refresh and delete come with the module of the group; fresh comes with "All modules", in that group when
"All modules" is the target, else as one more row under the module's group with the object "all
modules". A row opens the Actions tab on its action with the group open (D5). Rows are labelled like the
list rows ("Delete module **Checkout**").

### D9. Recordings and tests

New recordings in `fixtures/output/`, made as in milestones 11 and 12 from a scratch app with 13.3.0 and
SQLite, modules named like the fixture app, no agent variables in the environment:
`module:migrate-reset Checkout`, `module:migrate-reset Checkout --pretend`, `module:migrate-refresh
Checkout`, `module:migrate-fresh --all`, `module:delete Checkout --force` (last), and
`module:migrate-reset Shipping` with `Module::prohibitDestructiveCommands()` in the scratch app's
`AppServiceProvider`. `args` stay unique across the folder.

Core: D2 and D3 tests; `uncommitted` on a temporary repository with a changed, a staged, an untracked and
an ignored file, in a subfolder app too (the prefix), and `None` outside a repository.

Interface, `tests/actions.rs` against the fake artisan: the group is closed and opens with J and Enter;
Ctrl Enter on Delete opens the dialog; the typed name keeps the button off until "Checkout" is typed,
then the delete runs and the index selects "All modules" with the toast; production turns the run button
off and keeps Copy; reset dry run runs without a dialog; Run again of a delete opens the typed dialog.
The prohibited sentence in `tests/runner.rs`. The palette word rule once in `tests/palette.rs`. Images
S10.4 a to d and S10.5 a to d (c and d before and after typing), light and dark.

## Risks / Trade-offs

- [The uncommitted count is read with the snapshot] → The watcher reads the app again on a change of a
  file or of `.git/logs/HEAD`; between a change and that read, the count can be one change behind.
  The dialog says what git said at the last read.
- [Ignored files are not counted] → They are lost by a delete too, but git cannot restore them either
  and the module's file count leaves them out the same way.
- [Laravel changes the prohibition sentence] → The refusal still shows the package's own message and
  the plain failure sentence; the real-PHP job of milestone 16 catches it.
- [The refusal shows only after a run] → The run changed nothing: the package refuses before it acts.
- [A dry run runs migration code] → `--pretend` calls each migration's `down()` with the migration's
  connection in pretend mode, as Laravel does. Code in a migration that writes through another
  connection, or does more than SQL, still runs. The spec offers the dry run as "print the SQL, change
  nothing", and the dry run of roll back has run without a question since milestone 12. Kept: a
  migration that writes outside its connection in `down()` breaks Laravel's own `--pretend` too.

## Migration Plan

`Module` gets one field; a `settings.json` or history from milestone 12 reads as before. Rollback is a revert.

Order: 13.1 (D1) and 13.2 (D2, D3, recordings) in parallel, then 13.3 (D5 to D8, tests, images, deviations).
