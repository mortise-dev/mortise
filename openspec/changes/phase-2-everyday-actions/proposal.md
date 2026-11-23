# Proposal

## Why

Milestone 11 built the runner drawer, but nothing in Mortise starts a package command yet. Milestone 12
of `docs/plan/phase-2.md` adds the everyday actions of a module: turn it off or on, migrate, roll back,
seed, run tests and check translations, with the confirmations of S10.3 and the production rule. It also
brings the sentence of S11.1 c that names the migration that failed, carried over from milestone 11.

## What Changes

- The core holds the action list of a module and of "All modules" as data, the artisan words of each
  action from its options, and the confirmation table of S10.3.
- The production rule: when the app's `APP_ENV` is `production`, migrate and seed ask first and pass
  `--force`. Reset, refresh, fresh and delete do not run there. A command that the package confirms gets
  `--force` only after Mortise asked.
- The Actions tab (S10.1, S10.2 a to f): the list on the left, the panel of the selected action on the
  right with its options, its command line and its run button. Run tests shows the placeholder of
  S10.2 e until P7 (milestone 15).
- The dialogs of S10.3: roll back, turn off a module that others use, migrate or seed in production.
- Migrate and Run tests in the header of S4.3. Actions in the command palette.
- Toasts after an action ends: "Turned off Checkout.", "Migrated Checkout: 1 migration ran." and the
  others of C13. A failure out of view gets a failed toast with "Show output".
- The drawer names the migration that failed (S11.1 c), with "Open migration file" and "Dry run".
- Data that needs P4 (waiting migrations, the last batch) is not shown. The spec's fallback applies.

## Capabilities

### New Capabilities

- `module-actions`: the actions of a module and of "All modules": the list, the options, the command
  line on screen, the run, the toasts, and the entry points in the header and the palette.
- `action-confirmation`: when Mortise asks before an action, when it passes `--force`, and what it
  refuses to run in production.

### Modified Capabilities

- `runner-drawer`: a failed migration names the migration and offers "Open migration file" and "Dry run".

## Impact

- `mortise-core`: new `catalog/` (actions and the confirmation table), new `output.rs` (reading the
  migration lines of the output), `lib.rs`. No new dependency.
- `mortise-app`: new `actions_view.rs`, `app.rs`, `workspace.rs` (header buttons, Actions tab),
  `dialog.rs` (a left button and the destructive style), `runner_view.rs` (the failure sentence),
  `palette.rs`, `toast.rs` (one action), `strings.rs`, tests and images.
- `fixtures/output/`: new recordings of 13.3.0 for the actions.
- Docs: `docs/deviations.md`.
- Not run here: Windows and Linux. CI runs the build and tests there.
