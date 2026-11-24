# real-php-check Specification

## Purpose
States how Mortise checks its runner and output reader against the real commands of
`nwidart/laravel-modules`: a CI job that is not required, and core tests that are ignored by default.

## Requirements

### Requirement: A CI job runs the package's commands in a real app
A Linux job in CI, not required, SHALL install PHP 8.4 and Composer, create a Laravel app with
`nwidart/laravel-modules` `^13.3` at test time, and run the core tests marked as ignored. It and the job
of the `real-actuator-check` capability MUST be the only jobs that use the network for Composer.

#### Scenario: The package changes a sentence
- **WHEN** a new release of the package prints a generator line that the output reader does not read
- **THEN** the job fails, and the other jobs still pass

### Requirement: The ignored tests run the real commands
The ignored core tests SHALL run `module:make`, one generator, `module:migrate` on SQLite and
`module:delete` through the runner Mortise uses, and check what the output reader reads from each. With
13.4 or later they SHALL also run one generator dry run, one generator run with `--json`,
`module:migrate-status --json`, `module:test` and `module:lang --json`, and read each JSON answer with
Mortise's own types. Without the app they MUST be skipped by default.

#### Scenario: Generator output
- **WHEN** the tests run a generator in the real app
- **THEN** the output reader lists the files the generator created

#### Scenario: A plain test run
- **WHEN** `cargo test --workspace` runs without `--ignored`
- **THEN** the tests that need PHP do not run

#### Scenario: A dry run in the real app
- **WHEN** the tests run a generator with `--dry-run --json`
- **THEN** the answer lists its files and the module folder is unchanged
