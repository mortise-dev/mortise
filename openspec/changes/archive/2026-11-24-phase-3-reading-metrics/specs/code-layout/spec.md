# Spec Delta

## MODIFIED Requirements

### Requirement: Core modules have no cycle
The modules of `mortise-core` SHALL depend on each other in one direction, lowest first: `php` and `program`; `modules`; `boundary`; `package`; `findings`; `run`; `actions`; `generate`; `app`; `surface`, `runtime` and `palette`; `metrics`; `editor`, then `settings`. A module SHALL NOT name a module that comes after it in this list.

#### Scenario: Low modules name no high module
- **WHEN** `package.rs` and `run/` are searched for the names of `generate`, `actions` and `palette`
- **THEN** nothing is found

#### Scenario: The runtime does not name metrics
- **WHEN** `runtime/` is searched for `crate::metrics`
- **THEN** nothing is found

#### Scenario: Metrics exist only with the actuator
- **WHEN** `mortise-core` is built with the feature `actuator` alone, and then with no feature
- **THEN** `metrics/` compiles in the first build and is absent from the second
