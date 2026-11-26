## ADDED Requirements

### Requirement: A use outside the list marks each file
For a finding of the rule allowed use, `mortise check` SHALL give one line per file of the module that uses the other module, at the first line of the file that names a class of it, in the text and github formats. The sentence SHALL name both modules and the list of the module. A name in `allowed-use` that is no module of the app SHALL give a warning line placed on `mortise.json`, without a line number. The github title SHALL be "Allowed use".

#### Scenario: The fixture layered
- **WHEN** `mortise check fixtures/layered` runs
- **THEN** it prints one problem line for each of the two files of Audit that use Billing, one warning line on `mortise.json` for Reports, and the summary "1 problem, 1 warning. 5 modules checked against 3 rules and 3 checks.", and exits 1

#### Scenario: GitHub
- **WHEN** `mortise check --format github fixtures/layered` runs
- **THEN** each file of Audit that uses Billing gets its own `::error file=...,line=...,title=Allowed use::` line, and the warning gets `file=mortise.json` and no `line`

#### Scenario: ci.fail off
- **WHEN** a copy of `fixtures/layered` sets `"ci": { "fail": false }`
- **THEN** the same lines are printed and the exit code is 0
