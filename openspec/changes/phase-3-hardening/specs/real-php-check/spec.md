## MODIFIED Requirements

### Requirement: A CI job runs the package's commands in a real app
A Linux job in CI, not required, SHALL install PHP 8.4 and Composer, create a Laravel app with
`nwidart/laravel-modules` `^13.3` at test time, and run the core tests marked as ignored. It and the job
of the `real-actuator-check` capability MUST be the only jobs that use the network for Composer.

#### Scenario: The package changes a sentence
- **WHEN** a new release of the package prints a generator line that the output reader does not read
- **THEN** the job fails, and the other jobs still pass
