# php-runner Specification

## Purpose
States how PHP runs for each app: the kinds of runner, how one is proposed from the folder, the command
line Mortise builds, the test that checks it, and what Settings keep.

## Requirements

### Requirement: Four kinds of runner
Each app SHALL run PHP through one of four kinds of runner: Herd, Sail, Docker with a service name, or a
custom command the user types. Each kind gives the part of the command line before the artisan
arguments: `herd php artisan`, `./vendor/bin/sail artisan`, `docker compose exec -T <service> php artisan`,
or the custom words.

#### Scenario: Docker with a service
- **WHEN** the runner of an app is Docker with the service `app`
- **THEN** the command line starts with `docker compose exec -T app php artisan`

#### Scenario: Custom command
- **WHEN** the runner is custom with the line `"/opt/my php/bin/php" artisan`
- **THEN** the program is `/opt/my php/bin/php` and the next word is `artisan`

### Requirement: A command line never goes through a shell
Mortise SHALL start a command as one program with a list of arguments. No shell MUST read the command
line, and no part of a module name, an option or a setting MUST be joined into a string that a shell reads.

#### Scenario: An argument with shell characters
- **WHEN** an artisan argument holds `; rm -rf ~` or `$(whoami)`
- **THEN** the program receives that text as one argument and no shell runs it

### Requirement: Every artisan command is non-interactive and keeps its colours
Every artisan command line SHALL hold `--no-interaction` and `--ansi`, once each.

#### Scenario: Version check
- **WHEN** Mortise builds the command line for `--version`
- **THEN** the arguments after the prefix are `--version`, `--no-interaction` and `--ansi`

### Requirement: The command line is shown and copied as one line
Mortise SHALL show and copy a command line as one line. Each word that needs quoting MUST be quoted for
the platform, so the line pasted into a terminal of that platform runs the same program with the same
arguments.

#### Scenario: A path with a space on macOS or Linux
- **WHEN** a word is `/code/shop api/artisan`
- **THEN** the line shows it as `'/code/shop api/artisan'`

#### Scenario: Plain words
- **WHEN** no word holds a space, a quote or a character a shell treats as special
- **THEN** the line shows the words as they are, one space apart

### Requirement: The runner is proposed from the folder
When an app is added and no runner is chosen yet, Mortise SHALL propose one from the folder: Herd when
the folder is a Herd site, else Sail when `vendor/bin/sail` and a compose file exist, else Docker when a
compose file exists. Sail is not proposed on Windows. Detection MUST read files only and run nothing.

#### Scenario: A Sail app
- **WHEN** the folder holds `vendor/bin/sail` and `compose.yaml`, is not a Herd site, and Mortise runs on macOS or Linux
- **THEN** the proposed runner is Sail and the hint names what was found

#### Scenario: Nothing found
- **WHEN** the folder is not a Herd site and holds neither Sail nor a compose file
- **THEN** the hint reads "Not found: Herd, Sail, a compose file." and Herd stays selected

### Requirement: Herd on Windows
On Windows, Mortise SHALL start Herd as `herd.bat` and read Herd sites from the Herd folder of Windows.

#### Scenario: Windows prefix
- **WHEN** the runner is Herd on Windows
- **THEN** the program of the command line is `herd.bat`

### Requirement: Test runs artisan --version
"Test" SHALL run `artisan --version` through the chosen runner in the working folder of the app. Mortise
MUST show the first line of the answer, or what failed and the next step. A test that runs longer than
its limit MUST be ended.

#### Scenario: PHP answers
- **WHEN** the user presses Test and artisan prints `Laravel Framework 13.2.0`
- **THEN** the field shows "Answered" with `Laravel Framework 13.2.0`

#### Scenario: The program is not found
- **WHEN** the user presses Test, the runner is Herd and `herd` is not installed
- **THEN** the field reads: Herd did not answer: "herd: command not found". You can add the app now and read it. Generating and running need PHP: choose another way, or enter the command.

### Requirement: Settings keep how PHP runs for each app
Settings SHALL keep the runner, the working folder and the time limit of each app. The working folder
defaults to the app folder and the time limit to 10 minutes. A `settings.json` written before these
fields existed MUST still load.

#### Scenario: An old settings file
- **WHEN** Mortise loads a `settings.json` whose apps have no runner, working folder or time limit
- **THEN** the apps load with no runner chosen, the app folder as working folder and 10 minutes

### Requirement: Settings show PHP for every app
The PHP section of Settings (S8.2) SHALL list each app with its runner, its command and its last test,
and edit the runner, working folder and time limit of the selected app. The palette SHALL open this
section.

#### Scenario: Open from the palette
- **WHEN** the user picks the PHP entry of Settings in the palette
- **THEN** Settings open on the PHP section
