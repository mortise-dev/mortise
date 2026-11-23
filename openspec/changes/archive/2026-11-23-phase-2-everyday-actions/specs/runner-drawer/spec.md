# Spec Delta

## ADDED Requirements

### Requirement: A failed migration names the migration
When a `module:migrate` run fails and its output marks a migration as failed, the sentence of the drawer
SHALL name that migration, as S11.1 c says, and offer "Open migration file" and "Dry run" before "Copy
output". "Open migration file" opens the file in the editor. "Dry run" opens the Migrate panel with the
dry run checked; it does not run.

#### Scenario: A migration fails
- **WHEN** `module:migrate Checkout` exits with code 1 and the output reads `2026_11_20_091200_add_tracking_code_to_checkouts_table ... FAIL`
- **THEN** the drawer reads "module:migrate Checkout stopped with exit code 1 at 2026_11_20_091200_add_tracking_code_to_checkouts_table. The error from the database is quoted above. Fix the migration, or try a dry run to read the SQL."

#### Scenario: Open the migration
- **WHEN** the file of that migration exists in the module and the user presses "Open migration file"
- **THEN** the file opens in the editor

#### Scenario: Another failure
- **WHEN** a run fails and no migration is marked as failed
- **THEN** the drawer keeps the sentence "<label> stopped with exit code <n>." of milestone 11
