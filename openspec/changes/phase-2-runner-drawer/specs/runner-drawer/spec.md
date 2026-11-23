# Spec Delta

## Purpose

States what the runner drawer of S11.1 shows and how it is used: the run, its status and output, the
history, its height and its keys.

## ADDED Requirements

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
