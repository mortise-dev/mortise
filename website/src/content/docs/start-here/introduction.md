---
title: Introduction
description: What Mortise is, who it is for, what it needs from an app and how these docs are laid out.
sidebar:
  order: 1
---

Mortise is a desktop app for developers who build Laravel apps with
[`nwidart/laravel-modules`](https://github.com/nWidart/laravel-modules).
It opens the folder of an app, lists its modules and shows what is in each one.
It runs on macOS, Windows and Linux.

## What Mortise shows

- Every module of the app, with its state, files, lines, tests and coverage.
- Which module uses which, read from the PHP source, and any circular use.
- Three boundary rules, kept in `mortise.json` in the app folder so that a team shares them.
- Findings: what breaks a rule or a check, and where it lives.
- A proposed public surface for a module.
- With [`nwidart/laravel-actuator`](https://github.com/nWidart/laravel-actuator) and a running app:
  the runtime status, the routes, the scheduled tasks and the metrics of each module.

## What Mortise runs

Mortise runs the commands of the package: it turns modules on and off, migrates, seeds, runs tests
and runs the generators. A command runs only when you start it. The exact command is on screen
before it runs, and it runs through the PHP you chose for the app: Herd, Sail, Docker or a command
of your own.

## What Mortise does not do

- It needs no PHP to read an app.
- The only file it writes in an app is `mortise.json`, and it asks before it creates it.
  The commands of the package write the other files.
- It asks the actuator for a short list of endpoints only, and never for `env`.
- It keeps a token or a password in the credential store of the operating system, never in a file.

[What Mortise changes, and what it never does](/mortise/explanation/safety/) gives the reasons.

## What an app needs

- A Laravel app with `nwidart/laravel-modules`, with `composer install` run in it.
- To run commands: a way to run PHP in the app.
- For runtime status and metrics: `nwidart/laravel-modules` 13.4 or later, `nwidart/laravel-actuator`
  and `nwidart/laravel-metrics`. See [Package versions](/mortise/reference/package-versions/).

## How these docs are laid out

| Section | Use it to |
|---|---|
| Start here | Install Mortise and open your first app |
| [Tutorial](/mortise/tutorial/first-app/) | Learn Mortise by doing one task from start to end |
| How-to guides | Reach one goal, such as [running the tests of a module](/mortise/how-to/run-tests/) |
| Reference | Look up a screen, a key, a file or a setting, such as [Settings](/mortise/reference/settings/) |
| Explanation | Understand why Mortise works as it does, such as [Module boundaries](/mortise/explanation/boundaries/) |

Something wrong, or a change you want? [Report a problem](/mortise/how-to/report-a-problem/).

Next: [Installation](/mortise/start-here/installation/).
