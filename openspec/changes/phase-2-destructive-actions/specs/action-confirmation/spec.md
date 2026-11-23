# Spec Delta

## MODIFIED Requirements

### Requirement: Mortise asks before an action as S10.3 says
Mortise SHALL ask before an action as the table of S10.3 says. Turn on, run tests and check
translations never ask. Turn off asks only when a module that is on uses the module. Migrate and seed ask
only in production. Roll back always asks, except a dry run outside production. Reset always asks, except
a dry run outside production. Refresh, fresh and delete always ask.

#### Scenario: Turn off a module that others use
- **WHEN** Shipping is on and uses Checkout, and the user turns off Checkout
- **THEN** Mortise asks "Turn off Checkout?" and names Shipping before it runs `module:disable Checkout`

#### Scenario: Turn off a module that no module on uses
- **WHEN** only modules that are off use Billing, and the user turns off Billing
- **THEN** `module:disable Billing` runs without a question

#### Scenario: Roll back
- **WHEN** the user rolls back Checkout outside production
- **THEN** Mortise asks "Roll back Checkout?" with focus on Cancel, and a dry run of the rollback runs without a question

#### Scenario: A dry run of reset
- **WHEN** the user resets the migrations of Checkout with the dry run checked, outside production
- **THEN** `module:migrate-reset Checkout --pretend` runs without a question

## ADDED Requirements

### Requirement: Each destructive action has one kind of target
Mortise SHALL plan reset, refresh and delete only for one module, and fresh only for all modules. For any
other target it MUST plan no command, so `module:delete --all` and a fresh of one module never run.

#### Scenario: Delete for all modules
- **WHEN** delete is planned for "All modules"
- **THEN** there is no command to run or copy

#### Scenario: Fresh for one module
- **WHEN** fresh is planned for Checkout
- **THEN** there is no command to run or copy
