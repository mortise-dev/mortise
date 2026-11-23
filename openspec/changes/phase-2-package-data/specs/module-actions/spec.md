# Spec Delta

## MODIFIED Requirements

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
