# Spec Delta

## MODIFIED Requirements

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
