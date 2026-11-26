# public-surface Specification

## Purpose
Defines the accepted public surface of a module: where it is read from, how the second boundary rule uses it, and how the window accepts and compares one.

## Requirements

### Requirement: The core reads the surface key of module.json
`mortise-core` SHALL read the `surface` key of each module's `module.json` without PHP, as three lists of strings: `classes`, `routes` and `commands`. A missing list SHALL be empty. A module without the key SHALL have no surface. A key that is not a JSON object of these lists (an array included), that holds another key, or whose lists hold an entry the package would not store as it is, SHALL count as no surface, as the package reads it.

#### Scenario: The fixture module with a surface
- **WHEN** shop-api is read
- **THEN** Users has a surface whose classes are `Modules\Users\Contracts\UserLookup`, and Billing has none

#### Scenario: A surface the package would refuse
- **WHEN** the `classes` of a surface is a string
- **THEN** the module has no surface, and the folder list applies

#### Scenario: A surface as an array
- **WHEN** the `surface` key is an array of three lists
- **THEN** the module has no surface, and the folder list applies

### Requirement: The second rule checks an accepted surface
A reference to a class of a module with a surface SHALL be internal unless the surface lists the class. Class names SHALL compare without case and without a leading backslash. A module without a surface SHALL keep the folder list of the rules. The rule SHALL be checked in one place in `mortise-core`, so the window and `mortise check` give the same findings.

#### Scenario: A surface allows a model
- **WHEN** Billing's surface lists only `Modules\Billing\Models\Invoice`
- **THEN** Checkout's reads of the model are allowed, and its reads of the contract and the event are internal

#### Scenario: Empty lists
- **WHEN** Billing's surface has three empty lists
- **THEN** every read of Billing by Checkout is internal

### Requirement: Accept runs the package command after showing it
Accept SHALL run `module:surface <Module> --set --stdin --json` through the PHP the user chose for the app, with the new surface as one JSON object on its standard input. A dialog SHALL show the exact line before it runs. Mortise SHALL NOT write `module.json` itself. When the answer holds the stored surface, a toast SHALL say so and the app SHALL be read again; Otherwise a toast SHALL say why nothing was stored.

#### Scenario: Accepting a proposal
- **WHEN** the user presses Accept on a proposal of Billing in an app with laravel-modules 13.5.0 and confirms the dialog
- **THEN** the command runs with the checked entries on standard input, and the toast reads "Accepted the public surface of Billing: 7 entries."

#### Scenario: A package without the command
- **WHEN** artisan answers that `module:surface` is not defined
- **THEN** the toast says that the surface was not stored and to update nwidart/laravel-modules to 13.5 or later

### Requirement: A cut answer is checked against module.json
The package writes `module.json` before it prints its answer. An answer cut at the line limit of a command that ended with 0 SHALL count as stored when `module.json` holds the surface that was sent.

#### Scenario: A long answer
- **WHEN** the command ends with 0 after a line longer than Mortise keeps, and `module.json` holds the surface that was sent
- **THEN** the toast says that the surface was accepted

### Requirement: Accept is off when it cannot run
Accept SHALL be off, with its reason on hover, when the app's package is not a release of 13.5.0 or later, when no way to run PHP is chosen, or while the app runs a command.

#### Scenario: A 13.3.0 app
- **WHEN** the proposal of a 13.3.0 app is shown
- **THEN** Accept is off and its reason says that nwidart/laravel-modules 13.3.0 has no module:surface

### Requirement: The accepted surface is shown read only
A module with a surface SHALL show it read only, with Change and Scan again (S5.8). Change SHALL scan and show the proposal with what the surface lists checked. An accepted route SHALL stay checked when the routes were not read. Scan again in a change SHALL keep what the developer checked or left out.

#### Scenario: Change
- **WHEN** the user presses Change on the surface of Users
- **THEN** the proposal of Users shows `Modules\Users\Contracts\UserLookup` checked and a new interface unchecked

### Requirement: A later scan lists the differences
Scan again SHALL list only the differences (S5.9): what the scan proposes that the surface lacks, and what the surface lists that the module no longer has. A route pattern SHALL be new only when it stands for a route that no accepted route covers. A route SHALL count as gone only when the routes were read. Each difference SHALL start checked. The differences SHALL be dropped when `module.json` changes the accepted surface they were found against.

#### Scenario: An interface came and a class went
- **WHEN** Users gains an interface and its surface lists a class it no longer declares
- **THEN** a later scan lists both, each checked

#### Scenario: Nothing differs
- **WHEN** the scan proposes what the surface lists
- **THEN** the accepted surface stays on screen and a toast reads "The scan matches the accepted surface."

### Requirement: Accepting the differences
Accepting SHALL store the accepted surface with the checked differences applied. A difference left out SHALL keep its accepted state.

#### Scenario: One difference left out
- **WHEN** the class that went is left out and the user accepts
- **THEN** the stored surface adds the interface and still lists the class
