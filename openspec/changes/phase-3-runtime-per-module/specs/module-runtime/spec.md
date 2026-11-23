# Spec Delta

## Purpose

Defines how Mortise works out the runtime status of each module from the health components of an environment, and where the interface shows it.

## ADDED Requirements

### Requirement: A module indicator belongs to the module of its lower case name
A health component named `module:{name}:{key}` SHALL belong to the module of the app whose name, in ASCII lower case, equals `{name}`. Its key SHALL be the rest of the component name after `module:{name}:`.

#### Scenario: The recorded health of a 13.4 app
- **WHEN** `health` lists `module:blog:storage` and the app has a module named `Blog`
- **THEN** the component belongs to `Blog`, with the key `storage`

#### Scenario: No module has that name
- **WHEN** `health` lists `module:legacy:x` and the app has no module whose lower case name is `legacy`
- **THEN** the component belongs to no module and stays a component of the app

### Requirement: The status of a module is the worst of its own indicators
The runtime status of a module SHALL be the worst status of the indicators that belong to it, in the order Down, Out of service, Up, Unknown. A module with no indicator of its own and no error SHALL have no runtime status.

#### Scenario: One indicator up
- **WHEN** `module:billing:gateway` is Up and no other indicator belongs to `Billing`
- **THEN** `Billing` reads Up

#### Scenario: One indicator down
- **WHEN** `Checkout` has one indicator Up and one Down
- **THEN** `Checkout` reads Down

#### Scenario: Unknown ranks below Up
- **WHEN** `Checkout` has one indicator Up and one Unknown
- **THEN** `Checkout` reads Up

#### Scenario: A module with no indicator
- **WHEN** the `modules` component lists `Legacy` as turned off and no indicator belongs to it
- **THEN** `Legacy` has no runtime status

### Requirement: A module the modules component reports in error is Down
When the `modules` component lists a module with an `error`, that module SHALL read Down, whatever its own indicators say, and SHALL carry the error as the app wrote it. The `modules` component SHALL be matched to a module by lower case name.

#### Scenario: A module missing on disk
- **WHEN** the `modules` component lists `Billing` with `"error":"Enabled in the status store, not found on disk."`
- **THEN** `Billing` reads Down with the sentence "Enabled in the status store, not found on disk."

### Requirement: Module components leave the components of the app
The components of the app SHALL NOT include an indicator that belongs to a module or the `modules` component. An error the `modules` component gives for a name that is no module of the app SHALL stay in the app box, apart from the components. A `modules` component that does not carry the list of modules SHALL stay a component of the app.

#### Scenario: The app box of a 13.4 app
- **WHEN** `health` lists `db`, `cache`, `module:billing:gateway` and `modules`, and the app has a module `Billing`
- **THEN** the runtime box of the app lists `db` and `cache` only

#### Scenario: A module that is not Up is named in the app box
- **WHEN** `Billing` reads Down from its own indicator
- **THEN** the runtime box of the app has a line for `Billing` with the Down mark

#### Scenario: An error for a module the files do not have
- **WHEN** the `modules` component lists `Ghost` with an error and the app has no module `Ghost`
- **THEN** the runtime box of the app has a line for `Ghost` with the Down mark and the error

#### Scenario: The modules check failed
- **WHEN** the `modules` component is Down with an `error` detail and no list of modules
- **THEN** the runtime box of the app keeps the `modules` component with its error

### Requirement: The status follows what each environment answers
The runtime status of the modules SHALL come from the last answer of the chosen environment only. It SHALL NOT depend on the package version in `composer.lock`. Without an answer, with hidden components, or with no module component, no module SHALL have a runtime status.

#### Scenario: An app before 13.4
- **WHEN** the environment answers `health` with components and none of them is a module component
- **THEN** no index row has a runtime mark and no module Summary has a runtime box

#### Scenario: Hidden components
- **WHEN** `health` answers without components
- **THEN** no index row has a runtime mark and no module Summary has a runtime box

### Requirement: The index row shows the runtime mark beside the finding mark
An index row of a module that has a runtime status SHALL show a runtime stroke mark to the left of its finding mark, on two-line rows, on one-line rows and on the strip. The runtime mark SHALL NOT replace the finding mark. Each runtime mark SHALL have an accessible name that says the module, its status and the environment.

#### Scenario: A module down with no finding
- **WHEN** `Billing` reads Down and has no finding
- **THEN** its row shows the Down stroke mark and the good finding mark

#### Scenario: A module up with problems
- **WHEN** `Checkout` reads Up and has 2 problems
- **THEN** its row shows the Up stroke mark and the problem mark with 2

#### Scenario: The strip
- **WHEN** the index is collapsed to its strip and a module has a runtime status
- **THEN** the strip shows both marks of that row side by side, the runtime mark first

### Requirement: A Down module raises no pip on the rail
The pip on a rail tile SHALL count findings of the kind problem only. A module that reads Down SHALL NOT raise it.

#### Scenario: A module down, no problem
- **WHEN** `Billing` reads Down and the app has no problem
- **THEN** the tile of the app on the rail has no pip

### Requirement: The Summary of a module shows its runtime box
The Summary of a module that has a runtime status SHALL show a runtime box: the status of the module with when it was checked, the error of the `modules` component when there is one, then each of the module's own indicators on its own line, by key, with its status and its details as the app wrote them. A module with no runtime status SHALL have no runtime box.

#### Scenario: A module with an indicator
- **WHEN** `Billing` has `module:billing:gateway` Up with the details `provider fake` and `latencyMs 12`
- **THEN** the Summary of `Billing` has a runtime box with the line "Status of Billing" Up, and a line `gateway` Up

#### Scenario: Indicators with mixed statuses
- **WHEN** `module:billing:gateway` is Down and `module:billing:db` is Up with the detail `replicaLag 12`
- **THEN** the runtime box of `Billing` has a line for `gateway` and a line for `db` with `replicaLag 12`

#### Scenario: A module with no indicator
- **WHEN** `Legacy` has no runtime status
- **THEN** the Summary of `Legacy` has no runtime box
