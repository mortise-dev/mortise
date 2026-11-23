## ADDED Requirements

### Requirement: A CI job runs the package's commands in a real app
A Linux job in CI, not required, SHALL install PHP 8.4 and Composer, create a Laravel app with
`nwidart/laravel-modules` `^13.3` at test time, and run the core tests marked as ignored. It MUST be
the only job that uses the network for Composer.

#### Scenario: The package changes a sentence
- **WHEN** a new release of the package prints a generator line that the output reader does not read
- **THEN** the job fails, and the other jobs still pass

### Requirement: The ignored tests run the real commands
The ignored core tests SHALL run `module:make`, one generator, `module:migrate` on SQLite and
`module:delete` through the runner Mortise uses, and check what the output reader reads from each.
Without the app they MUST be skipped by default.

#### Scenario: Generator output
- **WHEN** the tests run a generator in the real app
- **THEN** the output reader lists the files the generator created

#### Scenario: A plain test run
- **WHEN** `cargo test --workspace` runs without `--ignored`
- **THEN** the tests that need PHP do not run
