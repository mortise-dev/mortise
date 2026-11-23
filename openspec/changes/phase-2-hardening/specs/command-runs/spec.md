## ADDED Requirements

### Requirement: Closing Mortise stops every run
When Mortise closes, it SHALL stop the run of each app as Stop does, and wait until each run ended or
the time limit of the stop passed. A run that ends this way MUST be kept in the history as stopped.

#### Scenario: Close during a run
- **WHEN** a command runs in shop-api and the user closes Mortise
- **THEN** the command and its children end before Mortise exits, or are killed when the stop's time limit passes

#### Scenario: Close with no run
- **WHEN** no command runs and the user closes Mortise
- **THEN** Mortise closes at once
