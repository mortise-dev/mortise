# Proposal

## Why

Milestone 12 built the everyday actions and already knows the rules of the four destructive commands
(`module:migrate-reset`, `module:migrate-refresh`, `module:migrate-fresh`, `module:delete`), but
nothing on screen offers them. Milestone 13 of `docs/plan/phase-2.md` adds them with the screens
S10.4 a to d and S10.5 a to d. These commands lose data that the app cannot bring back, so the spec
keeps them apart from the everyday actions and gives them the strongest confirmation.

## What Changes

- The Actions tab gets a group named "Destructive" at the end of the list, closed each time the tab or
  the module changes. A module has Reset migrations, Refresh migrations and Delete module. "All
  modules" has Drop all tables and migrate. J and K stop at the group's row; Enter opens it.
- Each of the four has a panel that names what is lost before the dialog does (S10.4 a to c), a red
  outline run button, and "Copy command". In production the run button is off, the reason is on the
  panel, and the command can still be copied (S10.4 d).
- The strongest dialogs (S10.5 a to d). Reset and refresh say what is dropped. Dropping all tables
  asks for the app name to be typed; deleting a module asks for the module name. Until the name matches
  exactly, the red button is off. Focus starts on Cancel, Enter never confirms, Esc cancels.
- The delete panel and dialog count the files of the module that are not committed in git. The line
  is left out when there are none. Without git they say Mortise cannot tell.
- When the app prohibits a command (`Module::prohibitDestructiveCommands()` or a command's own
  `prohibit()`), the package refuses it. The drawer says so in Mortise's sentence and quotes the
  package's message.
- The palette lists the four only when the query holds the command's own word (reset, refresh,
  fresh, delete). Choosing one opens the Actions tab on it. No header button and no shortcut runs them.
- Toasts after each of the four. After a delete the index selects "All modules".
- Run again of one of the four opens its strongest dialog, not the plain "Run again" question.
- A dry run of reset outside production runs without a question, as a dry run of roll back does.
- The table names and migration counts need P4 (milestone 15). The spec's fallback applies.

## Capabilities

### New Capabilities

- `destructive-actions`: the four destructive actions of a module and of "All modules": the closed
  group, the five ways they are kept apart, the panels, the strongest dialogs with the typed name, the
  count of files that are not committed, the refusal when the app prohibits a command, the toasts and
  the selection after a delete.

### Modified Capabilities

- `action-confirmation`: a dry run of reset outside production runs without a question; reset,
  refresh and delete are not planned for "All modules", and fresh only for "All modules".
- `runner-drawer`: Run again of a destructive command opens its strongest dialog. A run that the app
  prohibits gets Mortise's sentence for the refusal.

## Impact

- `mortise-core`: `git.rs` (count of files that are not committed), `module.rs` and `app.rs` (a field
  on `Module`), `catalog/actions.rs` (the destructive lists, the drop options, the typed-name check,
  the targets each action takes), `output.rs` (the refusal line, the rollback count). No new dependency.
- `mortise-app`: `actions_view.rs`, `dialog.rs` (typed name, code line, extra lines), `app.rs`,
  `palette.rs`, `runner_view.rs`, `strings.rs`, tests and images.
- `fixtures/output/`: new recordings of 13.3.0 for reset, refresh, fresh, delete and a prohibited command.
- Docs: `docs/deviations.md`.
- Not run here: Windows and Linux. CI runs the build and tests there.
