# command-runs Specification

## Purpose
States how a command runs in an app: its output as it comes, how it is stopped, its time limit, one run
at a time per app, the styles of its output and the history of runs.

## Requirements

### Requirement: Output arrives as the command writes it
Mortise SHALL read standard output and standard error of a running command and show each line as it
arrives, with no wait for the end of the command. Standard input MUST be closed.

#### Scenario: Lines before the end
- **WHEN** a command prints a line and then keeps running
- **THEN** the line is shown while the command still runs

#### Scenario: A command asks for input
- **WHEN** a command asks a question and waits for an answer
- **THEN** it reads the end of its input, and the run ends without waiting for the user

### Requirement: Stop ends the whole process tree
Stop SHALL end the command and every process it started. On macOS and Linux Mortise MUST send SIGTERM
to the process group and SIGKILL 3 seconds later if the command still runs. On Windows Mortise MUST end
the process tree.

#### Scenario: A command that ignores SIGTERM
- **WHEN** the user stops a command on macOS or Linux and the command ignores SIGTERM
- **THEN** the command is killed about 3 seconds later and the run ends as stopped

#### Scenario: A launcher with a child
- **WHEN** the runner starts PHP as a child, as `herd.bat` does on Windows, and the user stops the run
- **THEN** the child ends too

### Requirement: A run ends at its time limit
A run SHALL be stopped as Stop does when it runs longer than the time limit of the app. It MUST then
end as stopped, with a line that names the limit.

#### Scenario: A command that hangs
- **WHEN** the time limit is 1 minute and the command runs for longer
- **THEN** the run is stopped at 1 minute and the output names the limit

### Requirement: A run that ended leaves nothing running
When a run has ended, Mortise SHALL hold no thread or pipe for it. A process the command left behind
that still holds its output open MUST be ended on macOS and Linux.

#### Scenario: A child that outlives its command
- **WHEN** a command starts a child that keeps the output open and the command exits
- **THEN** the run ends within a second and the child is ended

### Requirement: One run at a time per app
Mortise SHALL run at most one command at a time in an app. A second start in that app MUST be refused
with the name of the running command. Runs in different apps MUST NOT block each other.

#### Scenario: A second start
- **WHEN** `module:migrate --all` runs in shop-api and another command is started in shop-api
- **THEN** the start is refused and names `module:migrate --all`

### Requirement: A run keeps at most 1 MB of output
A run SHALL keep at most 1 MB of output. Output past that MUST be read and dropped, so the command
never blocks, and the run MUST say that output was cut.

#### Scenario: A command that floods
- **WHEN** a command prints 2 MB of lines
- **THEN** the run ends normally, keeps 1 MB and says the rest was cut

### Requirement: Output colours map to four styles
Mortise SHALL map the SGR codes of the output to four styles: text, dim, success and error. Red maps to
error, green to success, yellow, grey and faint to dim, and every other code to text. Other escape
sequences MUST be removed from the shown text.

#### Scenario: A finished migration line
- **WHEN** a line holds dots in grey and `DONE` in green
- **THEN** the dots are dim and `DONE` is success

### Requirement: The history of runs
Mortise SHALL keep the last 50 runs of each app in its own data folder, with the command, the start
time, how the run ended and its output up to 1 MB. It MUST NOT write the history into the app folder.
A history that is missing or not valid MUST read as empty.

#### Scenario: The 51st run
- **WHEN** an app has 50 runs in its history and one more ends
- **THEN** the oldest run and its output are removed

#### Scenario: A damaged history
- **WHEN** the history file of an app is not valid
- **THEN** the history reads as empty and the next run is kept

### Requirement: Closing Mortise stops every run
When Mortise closes, it SHALL stop the run of each app as Stop does, and wait until each run ended or
the time limit of the stop passed. A run that ends this way MUST be kept in the history as stopped.

#### Scenario: Close during a run
- **WHEN** a command runs in shop-api and the user closes Mortise
- **THEN** the command and its children end before Mortise exits, or are killed when the stop's time limit passes

#### Scenario: Close with no run
- **WHEN** no command runs and the user closes Mortise
- **THEN** Mortise closes at once

### Requirement: A run can take standard input
A run or a read SHALL take optional input. With input, the program's standard input SHALL be a pipe that receives the input and then closes. Without input, standard input SHALL stay closed, as before.

#### Scenario: A question answered by input
- **WHEN** a command that asks for a name runs with the input `Billing` and a line end
- **THEN** it ends with exit code 0, where without input it ends with 1
