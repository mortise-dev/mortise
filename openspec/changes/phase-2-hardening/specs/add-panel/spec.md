## MODIFIED Requirements

### Requirement: The Add panel scrolls when it does not fit
The Add panel SHALL scroll when its content is taller than the room it has. Every control of the panel
MUST be reachable. A long app name MUST NOT push a control out of the panel.

#### Scenario: Text size 150%
- **WHEN** the text size is 150% and the Add panel is open
- **THEN** the user can scroll to the Add button and the buttons below the fields

#### Scenario: A long app name at 150%
- **WHEN** the text size is 150% and the app name is longer than the panel is wide
- **THEN** Cancel stays inside the panel
