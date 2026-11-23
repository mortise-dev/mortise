# Spec Delta

## Purpose

States when Mortise uses the data commands of `nwidart/laravel-modules` 13.4 (P2 to P8), how it starts
and reads them, and what it shows when an answer is missing or cannot be read.

## ADDED Requirements

### Requirement: The version gate
Mortise SHALL use P2 to P8 only when the app's `composer.lock` gives `nwidart/laravel-modules` as a
released version 13.4.0 or later, written as three numbers. For any other version it SHALL keep the
behaviour of milestones 10 to 14. It MUST NOT decide from the failure of a command whether a feature exists.

#### Scenario: A released 13.4
- **WHEN** the app's lock gives `v13.4.0`
- **THEN** the Generate and Actions tabs offer the package data

#### Scenario: A dev version
- **WHEN** the app's lock gives `dev-master` with the branch alias `13.x-dev`
- **THEN** the screens behave as for 13.3 and no data command is offered

#### Scenario: A pre-release
- **WHEN** the app's lock gives `13.4.0-RC1`
- **THEN** the screens behave as for 13.3

### Requirement: A read is started by the user
A command that only gives data (the generator list, a dry run, the migration state) SHALL run only when
the user starts it, with its exact line on screen beside the control that starts it. It SHALL run through
the app's runner, count as the app's one run, and show its result where the data goes, not in the drawer
or the history.

#### Scenario: Reading the migration state
- **WHEN** the user presses "Read migration state" on the Migrate panel of Checkout
- **THEN** `module:migrate-status Checkout --json` runs, the box says "Reading" while it runs, and the drawer does not change

#### Scenario: A read while a command runs
- **WHEN** `module:migrate Checkout` runs
- **THEN** the button "Read migration state" is off with the tooltip "module:migrate Checkout is running"

### Requirement: A missing or malformed answer
When a data command gives no JSON document, or one that cannot be read, Mortise SHALL say so where the
data goes, with the reason in quotes, and SHALL show what it showed without the data. A cut document
MUST NOT be read in part.

#### Scenario: The generator list cannot be read
- **WHEN** `module:generators --json` ends with no document and the error "Class not found" on standard error
- **THEN** the picker keeps the shipped list and says that Mortise could not read the generators, quoting "Class not found"

### Requirement: The generator list is cached
The generators read from an app SHALL be kept in Mortise's data folder for that app and its package
version, and used again in later sessions while the version is the same. A new package version MUST make
the cache stale.

#### Scenario: The package is updated
- **WHEN** the generators were read with 13.4.0 and the lock now gives 13.4.1
- **THEN** the picker shows the shipped list until the user reads the generators again

### Requirement: The migration state after a migration command
When a migrate, roll back, reset, refresh or fresh that is not a dry run ends, from anywhere and whatever
its exit code, Mortise SHALL forget the migration state it may have changed: every module for `--all` or a fresh, else its module,
whatever connection the state was read for. Mortise MUST NOT start a read of the state after it.

#### Scenario: Migrate after a read
- **WHEN** the state of Checkout was read and the user runs Migrate
- **THEN** after the migrate ends the state box reads "Not read yet." with "Read migration state", and no state command runs

#### Scenario: A dry run keeps the state
- **WHEN** the state of Checkout was read and the user runs Migrate with `--pretend`
- **THEN** after the run the state box still shows the state that was read

#### Scenario: A fresh forgets every module
- **WHEN** the state of every module was read and the user runs Fresh
- **THEN** after the run every state box reads "Not read yet."

### Requirement: Translations in Findings
After Check translations ran with its JSON answer, Findings SHALL list the check "Translations are
complete" and a warning for each language of a module that lacks keys or files, such as "Checkout: 4 keys
in en are missing in fr.". A check of one module MUST keep the report of the others.

#### Scenario: A missing key
- **WHEN** `module:lang Checkout --json` reports 4 keys missing in fr
- **THEN** Findings show the warning "Checkout: 4 keys in en are missing in fr." and Checkout's mark is a warning

### Requirement: To do in Findings
With a migration state, Findings SHALL list under "To do, not counted as findings" each module with
waiting migrations, such as "Checkout has 1 migration waiting.". A row MUST NOT change a mark or a count.

#### Scenario: A waiting migration
- **WHEN** the state of Checkout has one migration waiting
- **THEN** the To do group lists "Checkout has 1 migration waiting." and the problem and warning counts do not change
