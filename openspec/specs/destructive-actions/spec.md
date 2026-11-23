# destructive-actions Specification

## Purpose
States how Mortise offers the four commands that lose data (reset, refresh, fresh and delete), how it
keeps them apart from the everyday actions, and how it confirms them before they run (S10.4, S10.5).

## Requirements

### Requirement: The closed group
The Actions tab SHALL end its list with a group named "Destructive". For a module it holds Reset
migrations, Refresh migrations and Delete module, and the line "Dropping all tables is under "All
modules"." For "All modules" it holds Drop all tables and migrate. The group MUST be closed each time
the tab or the module changes.

#### Scenario: The group opens
- **WHEN** the Actions tab of Checkout opens
- **THEN** the group "Destructive" is closed and none of its actions shows

#### Scenario: Another module closes it
- **WHEN** the group is open on Checkout and the user selects Billing
- **THEN** the group of Billing is closed

#### Scenario: All modules
- **WHEN** "All modules" is selected and the user opens the group
- **THEN** it holds only Drop all tables and migrate, with the command `module:migrate-fresh --all`

### Requirement: Keys of the group
J and K SHALL stop at the row of the group, and Enter on that row SHALL open or close it. Ctrl Enter
(Cmd Enter on macOS) on a destructive action SHALL only open its dialog. No button in the module header
and no other shortcut runs a destructive action.

#### Scenario: Open with the keys
- **WHEN** the user presses J until the row "Destructive" is selected and presses Enter
- **THEN** the group opens and J moves on to Reset migrations

#### Scenario: Ctrl Enter on Delete module
- **WHEN** Delete module is selected and the user presses Ctrl Enter
- **THEN** the dialog "Delete the module Checkout?" opens and nothing runs

### Requirement: The palette names them only by their word
The command palette SHALL list a destructive action only when a word of the query is the command's own
word: reset, refresh, fresh or delete. Choosing one MUST open the Actions tab on that action, with the
group open, and MUST NOT run it.

#### Scenario: A query without the word
- **WHEN** the query is "checkout"
- **THEN** the palette lists no destructive action

#### Scenario: A query with the word
- **WHEN** the query is "delete checkout" and the user chooses "Delete module Checkout"
- **THEN** the Actions tab of Checkout opens on Delete module and nothing runs

### Requirement: The panel names what is lost
The panel of a destructive action SHALL name what is lost before the dialog does, under "What is lost",
with "cannot be undone". Its run button MUST be the red outline; the only filled red button is in the
dialog. Reset offers a dry run; refresh and fresh say they have none.

#### Scenario: Delete module
- **WHEN** the user selects Delete module on Checkout
- **THEN** the panel names the folder `Modules/Checkout` with its files and lines, its line in the statuses file, and says the tables stay

#### Scenario: Drop all tables and migrate
- **WHEN** the user selects Drop all tables and migrate under "All modules"
- **THEN** the panel offers Seed afterwards, Also drop views, Also drop types and the database connection, and the command line changes with each

### Requirement: Production refuses the four
When the app reports the environment `production`, the run button of each destructive action SHALL be
off, the panel MUST say "<app> reports the environment production. Mortise does not run this command
there. Copy it and run it where you can watch the database.", and "Copy command" MUST still copy the line.

#### Scenario: Reset in production
- **WHEN** `APP_ENV=production` and the user selects Reset migrations
- **THEN** the run button is off, the reason is on the panel, and Copy command copies the line

### Requirement: The strongest dialog
Each destructive action SHALL ask in its own dialog, whose title names the action and the object, and
whose body names what is lost. Focus MUST start on Cancel. Enter MUST NOT confirm. Esc MUST cancel. The
confirming button repeats the action and is the only filled red control.

#### Scenario: Reset asks
- **WHEN** the user runs Reset migrations on Checkout
- **THEN** the dialog "Reset the migrations of Checkout?" opens with focus on Cancel and "Dry run" on the left

#### Scenario: Enter does not confirm
- **WHEN** the dialog of Refresh migrations is open and the user presses Enter
- **THEN** nothing runs

### Requirement: A name typed to confirm
Dropping all tables SHALL ask for the app name to be typed, and deleting a module SHALL ask for the
module name. The red button MUST stay off until the typed text equals the name exactly, with the same
case and no extra space. Reset and refresh ask for no name.

#### Scenario: The name is typed
- **WHEN** the dialog "Delete the module Checkout?" is open and the user types "Checkout"
- **THEN** the button "Delete module" turns on, and pressing it runs `module:delete Checkout --force`

#### Scenario: Another name
- **WHEN** the user types "checkout" or "Checkout " in that dialog
- **THEN** the button "Delete module" stays off

### Requirement: Files that are not committed
The panel and the dialog of Delete module SHALL count the files of the module that git cannot restore:
changed, added and untracked files, read from git. The line MUST be left out when there are none.
When the app is not in a git repository or git is missing, they SHALL say "Mortise cannot tell whether
these files are in git."

#### Scenario: Changes that are not committed
- **WHEN** 12 files of Checkout have changes that are not committed
- **THEN** the dialog reads "12 of the files have changes that are not committed. Git cannot bring those back."

#### Scenario: No git
- **WHEN** the app is not in a git repository
- **THEN** the dialog reads "Mortise cannot tell whether these files are in git."

### Requirement: The app prohibits a command
When a run fails and the package's output says the command is prohibited, the drawer SHALL say "The app
prohibits this command, so the package refused to run it. Its message is quoted above." under the
output. Mortise learns this from the run, not before it.

#### Scenario: A prohibited reset
- **WHEN** the app calls `Module::prohibitDestructiveCommands()` and the user resets Checkout
- **THEN** the drawer quotes the package's warning and Mortise's sentence about the refusal

### Requirement: After a destructive action
When a destructive action ends with code 0, Mortise SHALL show its toast: "Reset the migrations of
Checkout: 9 rolled back.", "Refreshed the migrations of Checkout: 9 ran again.", "Dropped all tables of
shop-api and migrated 7 modules." or "Deleted the module Checkout.". After a delete the index MUST select
"All modules".

#### Scenario: Delete ends
- **WHEN** `module:delete Checkout --force` ends with code 0
- **THEN** a toast reads "Deleted the module Checkout." and the index selects "All modules"
