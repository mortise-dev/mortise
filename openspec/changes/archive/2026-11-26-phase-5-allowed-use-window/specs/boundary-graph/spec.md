## ADDED Requirements

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
