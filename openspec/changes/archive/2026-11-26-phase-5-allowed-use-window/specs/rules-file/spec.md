## ADDED Requirements

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
