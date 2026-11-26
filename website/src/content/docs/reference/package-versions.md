---
title: Package versions
description: What Mortise offers by version of nwidart/laravel-modules, and the versions of laravel-actuator and laravel-metrics it needs.
sidebar:
  order: 15
---

Mortise reads the versions of the packages from the `composer.lock` of the app. It reads them from
the file. It runs no PHP for this.

## nwidart/laravel-modules

An app needs `nwidart/laravel-modules` in `composer.lock`. Without it, Mortise says the app does not
use the package and shows `composer require nwidart/laravel-modules`.

The package gives Mortise more data from 13.4.0 on. Mortise uses that data only when the version is a
released 13.4.0 or later, written as three numbers, such as `13.4.0` or `13.5.2`. A dev version, a
branch alias or a pre-release, such as `dev-master` or `13.4.0-beta1`, gets the 13.3 behaviour,
because it may lack a feature.

| Feature | 13.3 and before, or a dev or pre-release version | 13.4.0 and later |
|---|---|---|
| Read the app: modules, links, rules, findings | Yes | Yes |
| Actions: turn on and off, migrate, roll back, seed, check translations | Yes | Yes |
| Destructive actions: reset, refresh, drop all tables, delete | Yes | Yes |
| List of generators | The 42 generators of 13.3.0, shipped with Mortise | The 43 generators of 13.4.0, shipped with Mortise, or the list read from the app with `module:generators --json` |
| `module:make-health-indicator` | No | Yes |
| A generator the app adds itself | No | Yes, after a read of the generators, under **Other** |
| **List files** before a run (`--dry-run --json`) | No: "The files are listed after the run." | Yes, for the generators the package ships |
| Result of a generator run | Read from the output lines | Read from the package's JSON answer (`--json`), else from the output lines |
| Migration state (`module:migrate-status --json`), after **Read migration state** | No | Yes: waiting and ran migrations, the last batch, the tables a reset drops |
| Counts in the roll back, reset and refresh dialogs | No | Yes, with the migration state |
| **Run tests** | Off: the package has no command for the tests of one module | `module:test`, with **Filter** and **Run in parallel**. The toast counts the tests |
| Check translations | `module:lang`. The output shows in the runner | `module:lang --json`. Findings lists the translation keys a language lacks |
| Findings: "To do, not counted as findings" | No | Yes, with the migration state, such as waiting migrations |
| Health indicators per module, named `module:{lower case name}:{key}` | No | Yes. The index and the Summary show the runtime status per module |

The picker of the Generate tab says which list it shows when that list is of another minor version
than the app's, for example "for nwidart/laravel-modules 13.3".

## nwidart/laravel-actuator

The runtime status, the routes, the scheduled tasks and the metrics need `nwidart/laravel-actuator`
in the app, and a running copy of the app.

| Feature | Needs |
|---|---|
| Runtime status, routes, scheduled tasks | `nwidart/laravel-actuator`, with `health`, and `info`, `mappings` and `scheduledtasks` in `exposure.include` of `config/actuator.php` |
| Runtime status per module | Also `nwidart/laravel-modules` 13.4.0 or later |
| Every scheduled task of a module, also a closure, a shell command or a described task | `nwidart/laravel-actuator` 1.5.0 or later, and the next release of `nwidart/laravel-modules`, which adds the `module` tag to `scheduledtasks`. With older versions, only a job of the module and a `php artisan` command that the module declares show in the module |
| Metrics tab | `nwidart/laravel-actuator` 1.3 or later, with `prometheus` in `exposure.include`, and `nwidart/laravel-metrics` |

Mortise checks only that `composer.lock` has an entry for `nwidart/laravel-actuator`. It does not
check its version. It asks for an endpoint only when the base path of the actuator lists it.
See [Actuator endpoints](/mortise/reference/actuator-endpoints/).

## nwidart/laravel-metrics

The Metrics tab reads what `nwidart/laravel-metrics` records, through the `prometheus` endpoint.
The app needs:

- `METRICS_ENABLED=true`,
- a store, such as `METRICS_STORE=cache`.

When metrics are off, `prometheus` answers with an empty body, and the Metrics tab says so.

Database time reads "unknown" until `nwidart/laravel-modules` labels database metrics with the module.

## Versions Mortise is tested with

The recorded answers of the tests come from an app with these versions:

| Package | Version |
|---|---|
| `laravel/framework` | 13.43.0 |
| `nwidart/laravel-modules` | 13.4.0 |
| `nwidart/laravel-actuator` | 1.3.1 |
| `nwidart/laravel-metrics` | 1.1.0 |

The CI job "Real actuator" builds an app with `nwidart/laravel-modules` `^13.4`,
`nwidart/laravel-actuator` `^1.3` and `nwidart/laravel-metrics` `^1.1`, on PHP 8.4.

## Related pages

- [Generate tab](/mortise/reference/generate/)
- [Actions tab](/mortise/reference/actions/)
- [How to connect an actuator](/mortise/how-to/connect-actuator/)
- [How to read metrics](/mortise/how-to/read-metrics/)
