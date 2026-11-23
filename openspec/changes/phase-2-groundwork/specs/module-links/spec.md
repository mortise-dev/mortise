# Spec Delta

## Purpose

Defines which files of a module make links and circles between modules, and which links count as rule findings.

## ADDED Requirements

### Requirement: Test files make no links
Files in the `tests/` folder of a module SHALL NOT make links. They MUST NOT count toward circles or rule findings.

#### Scenario: A test uses the factory of another module
- **WHEN** a file in `Modules/Billing/tests/` uses a model of Checkout
- **THEN** no link from Billing to Checkout comes from that file, and no rule finding comes from it

### Requirement: A rule finding counts only when its modules are on
A link SHALL make a rule finding only when both of its modules are on. A circle SHALL make a finding only when every module in it is on. A module counts as on unless the statuses file turns it off, so a module whose state is unknown counts as on. Boundary MUST still draw a link to or from a module that is off, in the off style.

#### Scenario: A link to a module that is off
- **WHEN** Billing is on, Checkout is off, and Billing uses an internal class of Checkout
- **THEN** no rule finding is counted, and Boundary draws the link in the off style

#### Scenario: An app whose module states are unknown
- **WHEN** the app does not use the file activator and Billing uses an internal class of Checkout
- **THEN** a rule finding is counted

#### Scenario: Both modules on
- **WHEN** Billing and Checkout are on, and Billing uses an internal class of Checkout
- **THEN** a rule finding is counted

### Requirement: Other members of a circle name where it is counted
A circle SHALL be counted once, on the module that carries it (Q4). The Summary of each other member MUST show one line "Part of a circle counted on <module>" (not in the spec), with the name of that module.

#### Scenario: Summary of a member that does not carry the circle
- **WHEN** Checkout carries a circle with Billing and the user opens the Summary of Billing
- **THEN** the Summary reads "Part of a circle counted on Checkout", and the finding count of Billing does not change
