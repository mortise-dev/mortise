# rules-file Specification

## Purpose
Defines the keys of `mortise.json` beyond the rules: whether a broken rule fails CI, and the coverage settings a team shares.

## Requirements

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
`rules`, `rules.public-surface-only`, `rules.allowed-use`, `rules.allowed-use.modules`, `ci` and `coverage` SHALL be read only from JSON objects. A list in their place SHALL NOT be read field by field. Each list of `rules.allowed-use.modules` SHALL be a list of strings. A value of another type SHALL make the file read as invalid, with the defaults and the parser's message.

#### Scenario: A list for ci
- **WHEN** `mortise.json` holds `"ci": [false]`
- **THEN** the file reads as invalid and CI fails on a problem

#### Scenario: A list for allowed-use
- **WHEN** `mortise.json` holds `"rules": { "allowed-use": { "modules": ["Audit"] } }`
- **THEN** the file reads as invalid and the defaults apply

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

### Requirement: rules.allowed-use lists the modules each module may use
`mortise.json` SHALL accept an optional `rules.allowed-use` object with `enabled` (a boolean, `true` when absent) and `modules` (an object from a module name to a list of module names, empty when absent). nwidart/larch reads the same key. Without the key the rule SHALL check nothing, and a save SHALL NOT add it. A save SHALL keep `enabled` and every list.

#### Scenario: No key
- **WHEN** `mortise.json` has no `allowed-use` key
- **THEN** the app reads as before: no allowed use result and no finding of it

#### Scenario: A save keeps the lists
- **WHEN** the rules of an app whose key lists Audit and History are saved
- **THEN** `mortise.json` holds the same lists for Audit and History

### Requirement: What the allowed use rule checks
When `rules.allowed-use.enabled` is `true`, each link from a module with a list to a module that its list does not name SHALL be a problem of the check `AllowedUse`, counted on the module that uses. An empty list SHALL mean that the module may use no module. A module without a list SHALL NOT be checked. A link from or to a module that is off SHALL break nothing. The check SHALL count as a rule, and its result SHALL count the modules of the app that have a list.

#### Scenario: The fixture layered
- **WHEN** Mortise reads `fixtures/layered`, where Audit has an empty list and uses Billing, History uses only Audit, Billing and Legacy, which is off, Ingest has no list, and Legacy is off with an empty list
- **THEN** the one problem is on Audit for Billing, with 2 files

#### Scenario: The rule off
- **WHEN** `rules.allowed-use.enabled` is `false`
- **THEN** the rule gives no result and no finding, and the lists are kept

### Requirement: A name in allowed-use that is no module is a warning
When the rule allowed use runs, each name in the key that is no module of the app, as a module with a list or in a list, SHALL be one warning of the check `AllowedUse`, with no module. A module that is off SHALL count as a module of the app.

#### Scenario: A typing mistake
- **WHEN** the list of History in `fixtures/layered` names Reports, which is no module
- **THEN** one warning names Reports

### Requirement: The window has the switch of the rule allowed use
The rules box of S5.3 SHALL have the switch "A module uses only the modules of its list", with a line
under it that counts the modules of the app with a list. The switch SHALL write
`rules.allowed-use.enabled` and keep every list. Without the key the switch SHALL be off and the line
SHALL read "No module has a list yet. Write the lists in mortise.json, under allowed-use." The window
SHALL NOT edit the lists. The first change SHALL ask before `mortise.json` is created.

#### Scenario: No key
- **WHEN** the user opens the Boundary of all modules of an app without `rules.allowed-use`
- **THEN** the switch is off and the line says that no module has a list

#### Scenario: Turn the rule off
- **WHEN** the user turns the switch off in `fixtures/layered`
- **THEN** `mortise.json` holds `"enabled": false` and the 3 lists, and no link breaks a rule
