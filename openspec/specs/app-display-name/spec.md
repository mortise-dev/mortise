# app-display-name Specification

## Purpose
Gives an app one display name, the name set in Settings when there is one, on every screen.

## Requirements

### Requirement: Every screen shows the display name of an app
The display name of an app SHALL be the name set in Settings when the user set one, else the name read from the app. The rail, the workspace header, Findings, Boundary and the command palette MUST show the display name.

#### Scenario: Renamed app in Findings, Boundary and the palette
- **WHEN** the user renames an app in Settings
- **THEN** Findings, Boundary and the palette show the new name

#### Scenario: App without a name in Settings
- **WHEN** no name is set in Settings for an app
- **THEN** each screen shows the name read from the app
