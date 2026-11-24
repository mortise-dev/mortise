# code-layout Specification

## Purpose
How the code of Mortise is laid out: the core has no interface code, the PHP parser stays in one file, the core's modules depend on each other in one direction, and the interface text of each feature lives in its own `strings.rs`.

## Requirements

### Requirement: The core has no interface dependency
`mortise-core` SHALL NOT depend on an interface crate. Drawing belongs in `mortise-app` only.

#### Scenario: Dependencies of the core
- **WHEN** the dependencies of `mortise-core` are listed
- **THEN** none of them is egui, eframe or another interface crate

### Requirement: The PHP parser stays in one file
All use of the PHP parser (`mago-*`) SHALL stay in `mortise_core::php`.

#### Scenario: Search for the parser
- **WHEN** the source of `mortise-core` is searched for `mago_`
- **THEN** every match is in `php.rs`

### Requirement: Interface text lives in strings files
Each feature folder of `mortise-app/src` SHALL hold its own `strings.rs`. `mortise-app/src/strings.rs` SHALL hold text that more than one feature shows. No other file holds interface text.

#### Scenario: Text of one feature
- **WHEN** a text is shown by one feature only
- **THEN** it is defined in the `strings.rs` of that feature folder

#### Scenario: Text of two features
- **WHEN** a text is shown by two or more features
- **THEN** it is defined in `mortise-app/src/strings.rs` and no name is defined twice

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

### Requirement: The core re-exports its public names at the crate root
`lib.rs` of `mortise-core` SHALL re-export the public names at the crate root, so `mortise-app` names them without a module path. A restructure SHALL NOT change these names.

#### Scenario: A name before and after a move
- **WHEN** a file of the core moves to another folder
- **THEN** `mortise_core::<Name>` still resolves and `mortise-app` needs no change for that move
