# module-actions Specification

## Purpose
States what the Actions tab of S10.1 and S10.2 offers for a module and for "All modules", how an action
shows its command before it runs, and how the header, the palette and the toasts reach the actions.

## Requirements

### Requirement: The actions of a module
The Actions tab of a module SHALL list, in this order: Turn off (or Turn on when the module is off),
Migrate, Roll back, Seed, Run tests and Check translations. "All modules" SHALL list Migrate all
modules, Seed all modules, Run all module tests and Check translations, with `--all`. Turning every module
on or off at once is not offered.

#### Scenario: A module that is on
- **WHEN** the user opens the Actions tab of Checkout, which is on
- **THEN** the first action reads "Turn off" and Migrate is selected

#### Scenario: All modules
- **WHEN** "All modules" is selected and the user opens the Actions tab
- **THEN** the list holds the four actions of "All modules" and Migrate all modules runs `module:migrate --all`

### Requirement: The command is on screen before it runs
The panel of the selected action SHALL show its options, the exact command line that its run button
starts, and "Copy command". The line MUST change with every option. A run MUST start only the line on
screen, through the PHP chosen for the app, never through a shell.

#### Scenario: An option changes the line
- **WHEN** the user checks "Dry run: print the SQL, run none of it" on Migrate Checkout
- **THEN** the command line ends with `--pretend` before anything runs

#### Scenario: Copy instead of run
- **WHEN** the user presses "Copy command"
- **THEN** the line is on the clipboard, nothing runs, and a toast reads "Copied the command."

### Requirement: One run per app
While a command runs in the app, every run button of the Actions tab SHALL be off, with the tooltip
"<label> is running".

#### Scenario: A run goes on
- **WHEN** `module:migrate Checkout` runs and the user looks at Seed
- **THEN** the Seed button is off and its tooltip reads "module:migrate Checkout is running"

### Requirement: Run tests waits for the package
For an app with 13.4 or later, Run tests SHALL run `module:test <module> --json`, or `module:test --all
--json` for "All modules", with the options `--filter` and `--parallel`, without asking first. Its toast
SHALL count the tests, such as "31 tests passed in Checkout.". For an older app Run tests SHALL show the
placeholder line of S10.2 e and SHALL NOT run or copy a command.

#### Scenario: Run tests of Checkout
- **WHEN** the user selects Run tests in an app with 13.3.0
- **THEN** the panel shows the placeholder line and the run and copy buttons are off

#### Scenario: Run tests with the package command
- **WHEN** the user runs Run tests of Checkout in a 13.4 app and 31 tests pass
- **THEN** `module:test Checkout --json` runs in the drawer and the toast reads "31 tests passed in Checkout."

### Requirement: Keys of the Actions tab
J and K SHALL move through the actions, Tab SHALL go into the options, and Ctrl Enter (Cmd Enter on
macOS) SHALL run the selected action, asking first where the rules say so.

#### Scenario: Run from the keyboard
- **WHEN** Migrate is selected and the user presses Ctrl Enter
- **THEN** the command on the panel starts and the drawer opens

### Requirement: Header and palette lead to the panel
Migrate and Run tests in the header of a module (S4.3) and the actions in the command palette SHALL open
the Actions tab with that action selected, so its line is on screen. They do not run it.

#### Scenario: Migrate in the header
- **WHEN** the user presses Migrate in the header of Checkout
- **THEN** the Actions tab opens with Migrate selected and nothing runs

#### Scenario: An action in the palette
- **WHEN** the query "check" matches Checkout and the user chooses "Migrate Checkout"
- **THEN** the Actions tab of Checkout opens with Migrate selected

### Requirement: Toasts after an action
When an action ends, Mortise SHALL show the done toast of C13 for it, such as "Turned off Checkout." or
"Migrated Checkout: 1 migration ran.". A dry run gets no toast. A failure gets a failed toast with
"Show output" only when the drawer is not open on that app.

#### Scenario: Migrate ends
- **WHEN** `module:migrate Checkout` ends with code 0 and one migration done
- **THEN** a toast reads "Migrated Checkout: 1 migration ran." and leaves after 4 s

#### Scenario: A failure out of view
- **WHEN** a run fails while the drawer is hidden
- **THEN** a failed toast names the command and its exit code and offers "Show output", which opens the drawer

### Requirement: Data that needs the migration state
Without a migration state read from the package (P4), the panels SHALL leave out "Waiting" and "Last
batch", and the buttons and dialogs SHALL carry no migration count. The roll back dialog offers the dry
run. With a state, Migrate SHALL show "Waiting" with the migrations and "Migrate (1)", Roll back "Last
batch" with "Roll back 1 migration" and the migrations in its dialog, and the header of S4.3 "Migrate
(1)" with "9 ran" and "1 waiting".

#### Scenario: Migrate without P4
- **WHEN** the app's package has no migration state
- **THEN** the Migrate button reads "Migrate" and the panel has no "Waiting" box

#### Scenario: Migrate with a state
- **WHEN** the state of Checkout has one migration waiting
- **THEN** the panel shows "Waiting" with "1 migration", the row reads "1 waiting", and the button reads "Migrate (1)"

#### Scenario: Roll back with a state
- **WHEN** the last batch of Checkout holds one migration and the user runs Roll back
- **THEN** the dialog lists that migration and its button reads "Roll back 1 migration"

### Requirement: Recent actions in the empty palette
With an empty query, the "Recent" group of the palette SHALL list, after the recent module, the most
recent distinct actions and generators of the current app, newest first, read from its run history.
An action row MUST show its line, such as "Migrate Checkout module:migrate Checkout", with the time
since its last run. A generator row MUST read "Generate a <label> in <module>" with "Form". Choosing a
row MUST lead to the panel or the form, as the palette does elsewhere. It MUST NOT run anything.

#### Scenario: A migrate that ran
- **WHEN** Migrate Checkout ran 2 minutes ago and the user opens the palette with no query
- **THEN** "Recent" holds "Migrate Checkout" with "ran 2 minutes ago", and Enter on it opens the Actions tab of Checkout with Migrate selected

#### Scenario: A generator that ran
- **WHEN** a model was generated in Billing and the user opens the palette with no query
- **THEN** "Recent" holds "Generate a model in Billing" with "Form", and Enter on it opens that form

#### Scenario: No history
- **WHEN** the app has no run in its history
- **THEN** "Recent" lists the recent module only, and no group is empty
