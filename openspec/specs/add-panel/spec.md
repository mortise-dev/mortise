# add-panel Specification

## Purpose
Keeps every control of the Add panel reachable, whatever the window size and the text size.

## Requirements

### Requirement: The Add panel scrolls when it does not fit
The Add panel SHALL scroll when its content is taller than the room it has. Every control of the panel MUST be reachable.

#### Scenario: Text size 150%
- **WHEN** the text size is 150% and the Add panel is open
- **THEN** the user can scroll to the Add button and the buttons below the fields
