## MODIFIED Requirements

### Requirement: A command that did not start
When a command cannot start, the drawer SHALL say "The command did not start" with the reason, and
offer "Open settings", which opens how PHP runs for that app.

#### Scenario: Herd is missing
- **WHEN** the runner is Herd and `herd` is not installed
- **THEN** the drawer reads: The command did not start: "herd: command not found". Choose how PHP runs for shop-api in Settings.

#### Scenario: Windows refuses a batch argument
- **WHEN** the runner is Herd on Windows and the standard library refuses to pass an argument to `herd.bat`
- **THEN** the drawer reads "The command did not start" with the reason the standard library gave, and nothing runs
