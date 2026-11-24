## ADDED Requirements

### Requirement: The environment switch of a runtime box stays inside the box
The environment switch in the heading of a runtime box SHALL stay inside the box and SHALL NOT cover its title, whatever the number and the length of the environment names. Every environment SHALL stay reachable by pointer and by keyboard, and SHALL keep its full name as its accessible name.

#### Scenario: Many long environment names
- **WHEN** an app has eight environments with names of 30 characters
- **THEN** the switch of the runtime box on the Summary is drawn inside the box, and each environment can be chosen
