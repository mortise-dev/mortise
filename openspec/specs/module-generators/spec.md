# module-generators Specification

## Purpose
States how Mortise offers the generators of `nwidart/laravel-modules` (S9.1 to S9.6): which generators
it knows, how a form builds the exact command, how names are checked, what a run's result says, and how
an existing file or module is replaced.

## Requirements

### Requirement: The generator catalog
Mortise SHALL describe each generator by its name, group, description, arguments and options, with
the keys of the package's generator description (P3). For an app with 13.4 or later it SHALL use the
generators read from the app when the user read them, else the description it ships of 13.4.0. For an
older app it SHALL use the description it ships of the 42 generators of 13.3.0. The picker MUST say
"for nwidart/laravel-modules 13.x" when the shipped description is of another minor version.

#### Scenario: The shipped catalog
- **WHEN** the Generate tab of Billing opens in an app with package 13.3.0
- **THEN** the picker reads "41 generators for Billing, and New module." and is not marked for 13.3

#### Scenario: Another version
- **WHEN** the app's `composer.lock` names `nwidart/laravel-modules` 13.2.0
- **THEN** the picker is marked "for nwidart/laravel-modules 13.3"

#### Scenario: The app's own generators
- **WHEN** a 13.4 app has a generator of its own and the user reads the generators of the app
- **THEN** the picker lists it under "Other" with its form

### Requirement: Groups of the picker
The picker SHALL show the generators in Mortise's groups and order of S9.1: In the app, Data, HTTP,
Events and queues, Views, Classes, Console and setup, Tests. A generator that Mortise does not place
MUST appear in a last group, "Other".

#### Scenario: A generator Mortise does not place
- **WHEN** the catalog holds a generator that the order of S9.1 does not name
- **THEN** it shows in the group "Other"

### Requirement: Keys of the picker
3 SHALL open the Generate tab and G SHALL open it with the filter focused. Typing SHALL filter by label
and by artisan name. Up, Down, Left and Right SHALL move, Enter SHALL open the form, and Esc SHALL clear
the filter.

#### Scenario: Filter
- **WHEN** the user presses G and types "polic"
- **THEN** Policy is the only file generator listed and Enter opens its form

### Requirement: The form builds the exact command
A form SHALL offer the arguments and options of its generator, with the module filled from the
selection. The command line under "Command" MUST be the line the run button starts, rebuilt on every
change, and "Copy command" (Ctrl Shift C, Cmd Shift C on macOS) MUST copy it. Each value MUST be one
word of the command; no part of a name or value is read by a shell. Checking "All of these" on the
model form MUST give `--all` in place of the seven flags.

#### Scenario: A checkbox changes the line
- **WHEN** the model form of Billing has the name PaymentIntent and the user checks Migration and Policy
- **THEN** the line ends with `module:make-model PaymentIntent Billing --migration --policy`

#### Scenario: All of these
- **WHEN** the user checks "All of these"
- **THEN** the seven boxes are checked and the line shows `--all` and none of the seven flags

### Requirement: The file list waits for the dry run
For an app with 13.4 or later, a form SHALL offer "List files" with the line of the dry run on screen.
After the dry run it SHALL list the files the command would write, by folder, and count them in the
primary button. Any change of the command MUST drop the list. For an older app a form SHALL NOT list
files: it shows the sentence "The files are listed after the run." and the button "Create files".

#### Scenario: Preview without P2
- **WHEN** the model form is on screen in an app with 13.3.0
- **THEN** no file is listed and the sentence "The files are listed after the run." shows under the command

#### Scenario: List files
- **WHEN** the user presses "List files" on the model form of Billing in a 13.4 app
- **THEN** the dry run lists three files under `Modules/Billing/`, the head reads "3 files, none exist yet", and the button reads "Create 3 files"

#### Scenario: A file exists (S9.3)
- **WHEN** the dry run reports that `PaymentIntentPolicy.php` exists
- **THEN** Policy is unchecked and marked "exists", `--policy` leaves the line, the row reads "exists", and the button reads "Create the other 2 files"

#### Scenario: The dry run changes nothing
- **WHEN** the dry run of a generator runs
- **THEN** no file or folder of the app is created or changed

### Requirement: Name checks
A class name SHALL start with a letter and have no spaces. A module name SHALL start with a letter and
hold only letters, digits, "-" and "_". While a name fails, the error SHALL show under its field and the
run button MUST be off. An empty name turns the button off without an error.

#### Scenario: A class name with a space
- **WHEN** the user types "payment intent" in the model name
- **THEN** the field reads "A model name starts with a letter and has no spaces. Write PaymentIntent, not "payment intent"." and the button is off

#### Scenario: A module name
- **WHEN** the user types "2fa" in the module names
- **THEN** the field reads "The module name [2fa] is not valid. Use letters, digits, "-", "_" or spaces, starting with a letter." and the button is off

### Requirement: The new module form
"New module" in the index footer, the picker, Ctrl N (Cmd N on macOS) and the palette SHALL select "All
modules" and open the form of `module:make` with its name field focused. The form SHALL offer the
names, the kind (Web, API, Plain; Web adds no flag), `--inertia`, "Turn the module on after creation"
(unchecked adds `--disabled`), "Replace the module if it exists" (`--force`) and the author. With Replace
checked, the run MUST ask first, and the dialog MUST show the line it runs.

#### Scenario: Ctrl N
- **WHEN** Checkout is selected and the user presses Ctrl N
- **THEN** "All modules" is selected and the Generate tab shows the new module form with focus in the names

#### Scenario: Replace the module
- **WHEN** Loyalty exists, Replace is checked and the user presses Ctrl Enter
- **THEN** the dialog "Replace the module Loyalty?" opens with the line, and nothing runs until "Replace module"

#### Scenario: After creation
- **WHEN** `module:make Loyalty --api` ends with code 0 and the app is read again with Loyalty
- **THEN** Loyalty is selected in the index and its Summary opens

### Requirement: Run only what is on screen
Ctrl Enter (Cmd Enter on macOS) and the primary button SHALL run the words of the line the form showed,
through the runner of the app. Without a runner, while a name fails, or while a command runs in the app,
the button MUST be off and Ctrl Enter MUST run nothing.

#### Scenario: A command is running
- **WHEN** `module:migrate Checkout` runs in the app
- **THEN** the primary button of the form is off with the tooltip "module:migrate Checkout is running"

### Requirement: The result of a run
When a generator run ends, the form SHALL keep its values and show the files it created, replaced, or
left because they exist, with "Open" for each that exists. For an app with 13.4 or later the files MUST
come from the run's JSON answer; when it is missing or malformed they come from the output lines and the
result says so. A run that created all its files SHALL show the done toast "Created 3 files in Billing.".
A run that left a file SHALL show the alert of S9.6, with "Open the existing file", and no toast while the
result is on screen. The result MUST NOT come from the exit code.

#### Scenario: Created
- **WHEN** `module:make-model PaymentIntent Billing --migration --policy --fillable=order_id,amount,currency,status` ends having created three files
- **THEN** the result reads "Created in Billing" with three rows and "Open", and the toast reads "Created 3 files in Billing."

#### Scenario: A file existed
- **WHEN** a model run prints that `InvoicePolicy.php` already exists
- **THEN** its row reads "exists, left as it is" and the alert says it was left as it is

#### Scenario: The JSON answer
- **WHEN** a generator run of a 13.4 app ends with a JSON answer that lists a file as `exists`
- **THEN** its row reads "exists, left as it is", whatever the task lines say

### Requirement: Replace a file
When Mortise knows the generator that writes a left file, the result SHALL offer "Replace <file>". It
MUST open the dialog "Replace <file>?" with "The file exists. The generator writes a new one over it,
and Mortise cannot bring the old one back." and the line it runs, with focus on Cancel. Only "Replace
file" runs that generator with `--force`.

#### Scenario: Replace the policy
- **WHEN** the user presses "Replace InvoicePolicy.php" and then "Replace file"
- **THEN** `module:make-policy InvoicePolicy Billing --force` runs and its result lists the file as replaced

### Requirement: Generators in the palette
The palette SHALL list a group "Generators" after the actions, with at most four rows matched against
the label and the artisan name, such as "Generate a model in Checkout" with `module:make-model`. A row
SHALL open the form and MUST NOT run a command.

#### Scenario: Find a generator
- **WHEN** Checkout is selected and the user types "model" in the palette
- **THEN** the row "Generate a model in Checkout" shows, and Enter opens the model form of Checkout
