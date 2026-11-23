# boundary-graph Specification

## Purpose
Defines how the graph of an app draws its module nodes so every node stays readable with many modules.

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
