# Spec Delta

## Purpose

States how Mortise checks its actuator client, the runtime per module and the placing of metrics against
a live Laravel app with laravel-modules, laravel-actuator and laravel-metrics: a CI job that is not
required, and core tests that are ignored by default.

## ADDED Requirements

### Requirement: A CI job serves a real app with the actuator
A Linux job in CI, not required, SHALL install PHP 8.4 and Composer, create a Laravel app with `nwidart/laravel-modules` `^13.4`, `nwidart/laravel-actuator` `^1.3` and `nwidart/laravel-metrics` `^1.1`, give it two modules with a health indicator each, serve it on the local address, make traffic, and run the ignored core tests against it.

#### Scenario: A package changes its answer
- **WHEN** a new release of one of the three packages writes an answer that Mortise no longer reads
- **THEN** the job fails, and the required jobs still pass

### Requirement: The ignored tests read health through the client
The ignored core tests SHALL read `health` of the served app through Mortise's actuator client and check the status of each module. Without the served app they MUST be skipped by default.

#### Scenario: Two modules with an indicator each
- **WHEN** the served app has `Blog` and `Billing` with one indicator each, both Up
- **THEN** both modules read Up, and their indicators leave the components of the app

#### Scenario: A plain test run
- **WHEN** `cargo test --workspace` runs without `--ignored`
- **THEN** the tests that need the served app do not run

### Requirement: The ignored tests read prometheus through the client
The ignored core tests SHALL read `prometheus` of the served app through Mortise's actuator client, after traffic, and place what they read by route and by job class with the routes of `mappings`.

#### Scenario: Traffic to a route of a module
- **WHEN** the job made requests to a route of `Blog` and ran a job of `Billing`
- **THEN** the figures give `Blog` requests and `Billing` jobs, and a request to a route of the app counts as "Not in a module"

### Requirement: The ignored tests read an empty answer and a 503
When the job serves the same app with metrics turned off, and with a metrics store that cannot be read, the ignored tests SHALL read each answer through the client.

#### Scenario: Metrics are off
- **WHEN** the app answers `prometheus` with 200 and an empty body
- **THEN** the client reports that metrics are off

#### Scenario: The store cannot be read
- **WHEN** the app answers `prometheus` with 503
- **THEN** the client reports the 503 with the line the app wrote
