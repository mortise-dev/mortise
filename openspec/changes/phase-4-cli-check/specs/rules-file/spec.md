# Spec Delta

## Purpose

Defines the keys of `mortise.json` beyond the rules: whether a broken rule fails CI, and the coverage settings a team shares.

## ADDED Requirements

### Requirement: ci.fail says whether a broken rule fails CI
`mortise.json` SHALL accept `ci.fail`, a boolean. When it is absent it SHALL read as `true`. The switch "Fail CI when a rule breaks" on S5.3 SHALL show it and write it through the save of the rules: through a temporary file beside the file, asking before the file is created, and keeping the other keys of the file and of `ci`.

#### Scenario: Turn the switch off
- **WHEN** the user turns off "Fail CI when a rule breaks" in an app whose `mortise.json` exists
- **THEN** `mortise.json` holds `"ci": { "fail": false }` and its other keys are unchanged

#### Scenario: No ci key
- **WHEN** `mortise.json` has no `ci` key
- **THEN** CI fails on a problem

### Requirement: The coverage object of mortise.json
`mortise.json` SHALL accept an optional `coverage` object with an optional `limit` (0 to 100) and an optional `report` (a path relative to the app folder, or absolute). A `coverage` that cannot be read, or a limit over 100, SHALL be ignored on its own: the rules and `ci` stay as read, and the window says the coverage block is ignored.

#### Scenario: A limit over 100
- **WHEN** `mortise.json` sets custom rules and `"coverage": { "limit": 101 }`
- **THEN** the custom rules apply, the coverage comes from the settings, and a save keeps the rules and the coverage block

### Requirement: Each section of mortise.json is an object
`rules`, `rules.public-surface-only`, `ci` and `coverage` SHALL be read only from JSON objects. A list in their place SHALL NOT be read field by field.

#### Scenario: A list for ci
- **WHEN** `mortise.json` holds `"ci": [false]`
- **THEN** the file reads as invalid and CI fails on a problem

### Requirement: Which coverage setting wins
For the coverage limit and for the coverage report, a read SHALL use the first that is set of: the value given for the read (a flag of `mortise check`), the `coverage` object of `mortise.json`, the setting of the window, the default. The order SHALL live in the read of `mortise-core` only.

#### Scenario: The file wins over the window
- **WHEN** `mortise.json` sets a limit of 70 and the window's setting is 10
- **THEN** the snapshot's coverage limit is 70

#### Scenario: A flag wins over the file
- **WHEN** `mortise.json` sets a limit of 70 and `--coverage-limit 30` is given
- **THEN** the snapshot's coverage limit is 30

### Requirement: Saving the rules keeps the coverage
Saving the rules from the window SHALL keep the `coverage` object of `mortise.json` as it is.

#### Scenario: Change a rule
- **WHEN** the user changes a rule in an app whose `mortise.json` sets `coverage`
- **THEN** the `coverage` object is unchanged after the save
