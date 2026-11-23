# Spec Delta

## ADDED Requirements

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
