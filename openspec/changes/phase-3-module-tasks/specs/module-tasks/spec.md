## ADDED Requirements

### Requirement: A module lists the console commands it declares

The snapshot of a module SHALL hold the name of each console command a class of the module declares,
outside its test folders. A name SHALL come from the signature of the class, from its command attribute,
or, for a class in the commands folder only, from its `$name` property. Reading the names SHALL NOT read
or parse a file a second time.

#### Scenario: A module with two commands

- **WHEN** the Billing module has a class with the signature `billing:send-invoices {--force}` and a class with the attribute `#[Signature('billing:retry-failed-payments')]`
- **THEN** the snapshot of Billing lists `billing:retry-failed-payments` and `billing:send-invoices`

#### Scenario: A command in a test folder

- **WHEN** a class with a signature is in the module's tests folder
- **THEN** its name is not in the list

### Requirement: A scheduled task belongs to the module that runs it

A scheduled task SHALL belong to a module when its target is a class name in the namespace of the
module, or when its target is `php artisan <name>` followed by any arguments and the module declares the
command `<name>`. Any other task, such as one with a description, a closure or a shell command, SHALL
belong to no module. When two modules declare the same command name, the task SHALL belong to the first
of them in the order of the snapshot.

#### Scenario: An artisan task

- **WHEN** the target of a task is `php artisan billing:send-invoices --force` and Billing declares `billing:send-invoices`
- **THEN** the task belongs to Billing

#### Scenario: An app with its own artisan path

- **WHEN** the app defines `ARTISAN_BINARY` as `./artisan`, the target of a task is `php ./artisan billing:send-invoices` and Billing declares `billing:send-invoices`
- **THEN** the task belongs to Billing

#### Scenario: A job

- **WHEN** the target of a task is `Modules\Inventory\Jobs\SyncStock`
- **THEN** the task belongs to Inventory

#### Scenario: A described task

- **WHEN** the target of a task is the description `Poll shipment tracking`
- **THEN** the task belongs to no module

#### Scenario: A command no module declares

- **WHEN** the target of a task is `php artisan queue:prune-batches`
- **THEN** the task belongs to no module

### Requirement: The screens use the same place for a task

The Module column of scheduled tasks on the app Summary (S4.2), the scheduled tasks on a module's
Summary (S4.3) and the schedule of a command in the public surface scan SHALL place a task with the rule
above. The public surface scan of a module SHALL be given only the tasks of that module. A task that belongs to no module SHALL read "none" in the Module column.

#### Scenario: The app Summary and the module Summary agree

- **WHEN** an environment answers the task `php artisan billing:send-invoices` and Billing declares that command
- **THEN** the Module column of the task reads `BIL`, and the Summary of Billing lists the task under "Scheduled tasks"
