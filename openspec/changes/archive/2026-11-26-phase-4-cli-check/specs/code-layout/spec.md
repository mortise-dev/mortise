# Spec Delta

## MODIFIED Requirements

### Requirement: Interface text lives in strings files
Each feature folder of `mortise-app/src` SHALL hold its own `strings.rs`. `mortise-app/src/strings.rs` SHALL hold text that more than one feature shows. The command line tool SHALL hold its text in `mortise-cli/src/strings.rs`. No other file holds interface text.

#### Scenario: Text of one feature
- **WHEN** a text is shown by one feature only
- **THEN** it is defined in the `strings.rs` of that feature folder

#### Scenario: Text of two features
- **WHEN** a text is shown by two or more features
- **THEN** it is defined in `mortise-app/src/strings.rs` and no name is defined twice

#### Scenario: Text of the command line tool
- **WHEN** the command line tool prints a text
- **THEN** it is defined in `mortise-cli/src/strings.rs`
