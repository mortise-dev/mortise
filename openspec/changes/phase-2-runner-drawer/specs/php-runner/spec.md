# Spec Delta

## MODIFIED Requirements

### Requirement: Test runs artisan --version
"Test" SHALL run `artisan --version` through the chosen runner in the working folder of the app. Mortise
MUST show the first line of the answer, or what failed and the next step. A test that runs longer than
its limit MUST be ended, with every process it started, as Stop ends a run.

#### Scenario: PHP answers
- **WHEN** the user presses Test and artisan prints `Laravel Framework 13.2.0`
- **THEN** the field shows "Answered" with `Laravel Framework 13.2.0`

#### Scenario: The program is not found
- **WHEN** the user presses Test, the runner is Herd and `herd` is not installed
- **THEN** the field reads: Herd did not answer: "herd: command not found". You can add the app now and read it. Generating and running need PHP: choose another way, or enter the command.

#### Scenario: A launcher that hangs
- **WHEN** the runner starts PHP as a child and the test passes its limit
- **THEN** the launcher and the child both end, and the test reads as timed out
