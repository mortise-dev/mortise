## MODIFIED Requirements

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

## ADDED Requirements

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
