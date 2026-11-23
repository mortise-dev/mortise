# runner-drawer Specification

## Purpose
States what the runner drawer of S11.1 shows and how it is used: the run, its status and output, the
history, its height and its keys.

## Requirements

### Requirement: The drawer opens when a command starts
The runner drawer SHALL open at the bottom of the workspace when a command starts and stay until it is
hidden. Ctrl ` (and Cmd J or Ctrl J) MUST show and hide it. Hidden, it is one line of 28 px. Until it is
first shown, it takes no room.

#### Scenario: Hide and show
- **WHEN** the drawer is open and the user presses Ctrl `
- **THEN** the drawer shrinks to its 28 px line with "Show", and Ctrl ` opens it again

#### Scenario: Nothing has run
- **WHEN** no command has run in the app and the user shows the drawer
- **THEN** it reads "No command has run yet. Output of generators and actions appears here."

### Requirement: The drawer shows the run and its status
The drawer SHALL show the command, its output as written with the four styles, and its status: the
elapsed time while it runs, "Done in" and the time, or the exit code and the time. A failure MUST end
with one sentence from Mortise under a rule.

#### Scenario: Running
- **WHEN** a command runs for 2.4 s
- **THEN** the status reads "Running, 2.4 s" and Stop is offered

#### Scenario: Failed
- **WHEN** the command exits with code 1 after 412 ms
- **THEN** the status reads "Exit code 1" and "412 ms", and the sentence from Mortise follows the output

### Requirement: Stop from the drawer
Stop and Cmd . or Ctrl . SHALL stop the running command. For a migration, a rollback or a seed Mortise
MUST ask first. Through Sail or Docker the drawer MUST say that PHP in the container may go on.

#### Scenario: Stop a migration
- **WHEN** `module:migrate Checkout` runs and the user presses Ctrl .
- **THEN** the dialog "Stop module:migrate Checkout?" asks first, with "Keep running" and "Stop command"

### Requirement: The drawer height
The top edge of the drawer SHALL be a handle that sets its height between 120 px and half the workspace
height, 200 px by default. With focus, Up and Down MUST change it by 24 px, and Home and End jump to the
highest and the lowest. A double click collapses the drawer.

#### Scenario: Keys on the handle
- **WHEN** the handle has focus and the user presses Up
- **THEN** the drawer is 24 px higher, up to half the workspace height

### Requirement: History in the drawer
History SHALL list the runs of the app, the latest first, each with its clock time, command and result,
and show the output of the selected run. Up and Down MUST move the selection, and Enter MUST run the
selected command again through the current runner of the app.

#### Scenario: Run again from history
- **WHEN** the user selects the second run in History and presses Enter
- **THEN** that command starts again and the drawer shows its output

### Requirement: A command that did not start
When a command cannot start, the drawer SHALL say "The command did not start" with the reason, and
offer "Open settings", which opens how PHP runs for that app.

#### Scenario: Herd is missing
- **WHEN** the runner is Herd and `herd` is not installed
- **THEN** the drawer reads: The command did not start: "herd: command not found". Choose how PHP runs for shop-api in Settings.

### Requirement: Run buttons while a command runs
While a command runs in an app, every control that starts a command in that app SHALL be disabled with
the tooltip "<command> is running".

#### Scenario: Run again while running
- **WHEN** `module:migrate --all` runs in shop-api
- **THEN** Run again in History is disabled with the tooltip "module:migrate --all is running"

### Requirement: A failed migration names the migration
When a `module:migrate` run fails and its output marks a migration as failed, the sentence of the drawer
SHALL name that migration, as S11.1 c says, and offer "Open migration file" and "Dry run" before "Copy
output". "Open migration file" opens the file in the editor. "Dry run" opens the Migrate panel with the
dry run checked; it does not run.

#### Scenario: A migration fails
- **WHEN** `module:migrate Checkout` exits with code 1 and the output reads `2026_11_20_091200_add_tracking_code_to_checkouts_table ... FAIL`
- **THEN** the drawer reads "module:migrate Checkout stopped with exit code 1 at 2026_11_20_091200_add_tracking_code_to_checkouts_table. The error from the database is quoted above. Fix the migration, or try a dry run to read the SQL."

#### Scenario: Open the migration
- **WHEN** the file of that migration exists in the module and the user presses "Open migration file"
- **THEN** the file opens in the editor

#### Scenario: Another failure
- **WHEN** a run fails and no migration is marked as failed
- **THEN** the drawer keeps the sentence "<label> stopped with exit code <n>." of milestone 11

### Requirement: Run again of a destructive command
Run again of a reset, refresh, fresh or delete, from the drawer, its keys or History, SHALL open the
strongest dialog of that action, with its typed name where it has one. When the app now reports
production, Run again MUST NOT run it: it opens the Actions tab on that action, where the line can be
copied.

#### Scenario: Run a delete again
- **WHEN** the last run is `module:delete Checkout --force` and the user presses Run again
- **THEN** the dialog "Delete the module Checkout?" opens and asks for "Checkout" to be typed

#### Scenario: Run again in production
- **WHEN** the app reports production and the user runs a reset again from History
- **THEN** nothing runs and the Actions tab opens on Reset migrations

### Requirement: A command the app prohibits
When a run fails and its output holds the package's line "This command is prohibited from running in
this environment.", the drawer SHALL put "The app prohibits this command, so the package refused to run
it. Its message is quoted above." in place of the sentence of milestone 11.

#### Scenario: A prohibited command
- **WHEN** `module:migrate-reset Shipping` exits with code 1 and its output holds the prohibition warning
- **THEN** the drawer shows the output as written and then Mortise's sentence about the refusal
