# boundary-graph Specification

## Purpose
Defines how the graph of an app draws its module nodes so every node stays readable with many modules, and how it marks a link that breaks the rule allowed use.

## Requirements

### Requirement: A node shows its code stamp and its name when the name fits
Each node of the graph SHALL draw the code stamp of its module. A node MUST be at least as wide as its
stamp. It MUST draw the module name only when the whole name fits in the node. The full name MUST be
the tooltip and the accessible name of the node.

#### Scenario: Graph of 12 modules
- **WHEN** the user opens the graph of an app with 12 modules
- **THEN** each node shows its code stamp, no name is cut, and the tooltip of a node gives the full name

#### Scenario: A chain of 12 modules
- **WHEN** the graph draws 12 modules that use each other in a chain
- **THEN** each node is at least as wide as its code stamp

#### Scenario: Name that fits
- **WHEN** a node is wide enough for the name of its module
- **THEN** the node draws the code stamp and the full name

#### Scenario: The graph of one module
- **WHEN** the graph shows one module and its neighbours
- **THEN** the name of each neighbour is drawn whole or left out, never cut

### Requirement: A link outside the list of its module breaks a rule
While the rule allowed use is on, a link from a module with a list to a module that its list does not
name SHALL be drawn as a link that breaks a rule, on the graph of the app and on the graph of one module,
unless one of the two modules is off. "Links that break a rule" SHALL list it.

#### Scenario: The fixture layered
- **WHEN** the user opens the Boundary of all modules of `fixtures/layered`
- **THEN** "Links that break a rule" holds one row, "Audit uses Billing outside its list"

### Requirement: The files of a link outside its list name the rule
The files of a link outside the list of its module SHALL each carry the problem mark. The head SHALL
read "Outside its list" with the number of files, and the footer SHALL name the rule and the list.

#### Scenario: The files of the break
- **WHEN** the user opens the row "Audit uses Billing outside its list"
- **THEN** the Boundary of Audit shows "Audit uses Billing", "Outside its list, 2 files" and "Rule: a module uses only the modules of its list. mortise.json lets Audit use no module."
