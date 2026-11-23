# Spec Delta

## ADDED Requirements

### Requirement: Reset and refresh name the migrations and tables
With a migration state of the module, Reset and Refresh SHALL show the count of migrations that ran
("9 ran"), the panel and the dialogs SHALL name the tables those migrations create, and the buttons SHALL
count the migrations ("Reset 9 migrations"). Tables are read from the migration files; a table Mortise
cannot read is not named. Without a state they keep the wording without counts.

#### Scenario: Reset with a state
- **WHEN** 9 migrations of Checkout ran and create 7 tables, and the user runs Reset migrations
- **THEN** the dialog lists the 7 tables, reads "7 tables. Nothing outside Checkout is touched." and its button reads "Reset 9 migrations"

#### Scenario: Reset without a state
- **WHEN** the migration state of Checkout was not read
- **THEN** the dialog has no table list and its button reads "Reset migrations"
