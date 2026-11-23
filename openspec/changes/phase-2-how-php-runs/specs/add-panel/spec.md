# Spec Delta

## ADDED Requirements

### Requirement: The Add panel chooses how PHP runs
After a folder is chosen, the Add panel SHALL show "Run PHP with" with Herd, Sail, Docker and Custom,
the command it gives, and a Test button. The proposal from the folder is selected first, and the hint
says what was found.

#### Scenario: A Herd site
- **WHEN** the chosen folder is a Herd site
- **THEN** Herd is selected, the command reads `herd php artisan` and the hint reads "Found: this folder is a Herd site."

### Requirement: An app without PHP can still be added
When the test of the Add panel fails, the panel SHALL show the failure at the "Run PHP with" field and
the add button SHALL read "Add <name>, read only". The app is added and read without PHP.

#### Scenario: PHP not found
- **WHEN** the test fails because the program is not found
- **THEN** the failure is shown under the field and pressing "Add booking-engine, read only" adds the app
