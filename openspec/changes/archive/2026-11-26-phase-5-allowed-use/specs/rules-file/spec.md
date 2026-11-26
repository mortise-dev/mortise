## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Each section of mortise.json is an object
`rules`, `rules.public-surface-only`, `rules.allowed-use`, `rules.allowed-use.modules`, `ci` and `coverage` SHALL be read only from JSON objects. A list in their place SHALL NOT be read field by field. Each list of `rules.allowed-use.modules` SHALL be a list of strings. A value of another type SHALL make the file read as invalid, with the defaults and the parser's message.

#### Scenario: A list for ci
- **WHEN** `mortise.json` holds `"ci": [false]`
- **THEN** the file reads as invalid and CI fails on a problem

#### Scenario: A list for allowed-use
- **WHEN** `mortise.json` holds `"rules": { "allowed-use": { "modules": ["Audit"] } }`
- **THEN** the file reads as invalid and the defaults apply
