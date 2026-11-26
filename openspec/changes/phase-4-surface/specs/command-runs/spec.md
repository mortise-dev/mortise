# Spec Delta

## Purpose

States how a command runs in an app: its output as it comes, how it is stopped, its time limit, one run
at a time per app, the styles of its output and the history of runs.

## ADDED Requirements

### Requirement: A run can take standard input
A run or a read SHALL take optional input. With input, the program's standard input SHALL be a pipe that receives the input and then closes. Without input, standard input SHALL stay closed, as before.

#### Scenario: A question answered by input
- **WHEN** a command that asks for a name runs with the input `Billing` and a line end
- **THEN** it ends with exit code 0, where without input it ends with 1
