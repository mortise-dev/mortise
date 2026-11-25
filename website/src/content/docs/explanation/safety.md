---
title: What Mortise changes, and what it never does
description: The rules that limit what Mortise runs, writes and asks for, and the reasons behind each.
sidebar:
  order: 2
---

Mortise points at the code of your app, and often at a database. A tool in that place must be easy to
trust. So a few rules hold in every build, on every platform. This page explains each rule and why it
is there.

## A command runs only when you start it

Mortise reads an app without PHP (see [Reading an app without PHP](/mortise/explanation/reading-without-php/)).
It runs a command of the package only when you start it: with a button, a key or the palette. It never
runs one by itself, not on a schedule, not when a file changes, not when you open a screen.

Before you start a command, the exact line is on screen. The Actions and Generate tabs show it under
**Command**: the PHP you chose for the app, `artisan`, the command, `--no-interaction --ansi`, and the
options. That line is the one the button runs. If the line is too long to see, the button stays off
until you scroll to the whole command. The `$` line of the runner drawer shows it again once it runs.

The reason is plain. A command of the package can create files, change a database or delete a module.
You should never have to guess what Mortise did on your behalf. A line on screen also teaches the
command: you can copy it and run it yourself.

A few more limits follow from this rule:

- Mortise starts the program with its arguments, never through a shell. On Windows, a `.bat` launcher
  such as `herd.bat` goes through `cmd.exe`, because that is the only way Windows runs it; Rust refuses
  an argument it cannot pass safely there, and the drawer then says the command did not start.
- The PHP is the one you chose for the app: Herd, Sail, Docker or a command of your own. Mortise does
  not look for another one. Sail runs in WSL only, so on Windows the Sail choice is off and a Custom
  command is the way to reach WSL.
- Even the test of that choice, `artisan --version`, runs only when you press **Test**. Its result is
  kept in memory, so after a restart Settings reads "Not tested" until you test again.
- A run has a time limit, and **Stop** ends it. Mortise keeps 1 MB of output and the last 50 runs of
  each app in its own data folder, never in the app.

### Questions before harm

Some commands lose data. For those, Mortise asks first, and the question says what is lost:

- Turning off a module that other modules use, and rolling back a migration batch, open a dialog.
- Reset and refresh name the tables that change. Dropping all tables asks you to type the name of the
  app. Deleting a module asks you to type the name of the module.
- When the package itself would ask a question, Mortise asks it instead and then passes `--force`.
  The command runs with `--no-interaction`, so it never waits for an answer you cannot see.

### The production guard

Mortise reads `APP_ENV` from the `.env` file of the app. It reads it again just before a destructive
command starts. When the value is exactly `production`:

- Migrate and Seed ask first, and say that the command runs with `--force` on that database.
- Reset, refresh, drop all tables and delete module do not run at all. Mortise shows the line and
  lets you copy it, so you run it where you can watch the database.

A file on disk is a weak signal: `.env` can lie, or be missing. So the guard adds a question, and it
never removes one. Destructive commands ask in every environment.

Delete module has one more guard. When Mortise cannot read where the app keeps its modules, it assumes
the package default, and the package could then delete another folder. In that case Mortise does not
run the command either.

## The only file Mortise writes is mortise.json

Mortise writes one file in an app: `mortise.json`, which holds the boundary rules. It asks before it
creates the file. Every other file in the app is written by the package's commands, which you start.

The rules live in the app, not in Mortise's settings, so that a team shares them through version
control. [About module boundaries](/mortise/explanation/boundaries/) says more about that choice.

A save never writes `mortise.json` in place. Mortise writes a temporary file beside it, named
`.mortise.json.` followed by random letters, then puts it in place of `mortise.json` in one step.
This means the file is never half written. The unique name
means no other file of yours is touched. Only a crash in the middle of a save can leave the temporary
file behind. Other keys in `mortise.json` are kept when the rules change.

Everything else Mortise keeps is outside the app: its settings, the history of runs and the secrets.
[Files and folders](/mortise/reference/files-and-folders/) lists where they live.

## The actuator client asks for six things only

With `nwidart/laravel-actuator`, Mortise can read a running app over HTTP. The client asks for the
actuator base path, `health`, `info`, `mappings`, `scheduledtasks` and `prometheus`. Nothing else.

It never asks for `env`. That endpoint can list environment variables, and those often hold database
passwords and API keys. Mortise has no use for them, so it does not ask, even when the app exposes the
endpoint.

It follows no redirect. A redirect could send the request, with its credentials, to another host.
When the app answers with a redirect, Mortise says where it points and asks you to correct the base
URL or the actuator path in Settings.

`health` is asked for every 15 seconds while the window is visible. `prometheus` is asked for only while
the Metrics tab is open and the window is visible. The others are asked for when the environment opens,
every 5 minutes, and when you read the app again. Every request is a GET: Mortise reads and changes
nothing in the running app.

HTTPS uses the certificates your computer trusts, so a local site with its own certificate authority,
such as a Herd `.test` site, works as it does in the browser.

## Secrets stay in the credential store

An environment can need a bearer token, a password or the value of a custom header. Mortise puts that
secret in the credential store of the operating system: Keychain on macOS, Credential Manager on
Windows, and the Secret Service on Linux. It is never written to a file and never written to a log.
Settings keeps only the kind of credentials, a user name or a header name.

When no credential store is running, as on some minimal Linux desktops, Mortise keeps the secret in
memory until it closes, and you enter it again next time. It does not fall back to a file.

Removing an app from Mortise forgets its environments and its saved credentials. The app folder stays
as it is.

## Platform limits

These rules hold on macOS, Windows and Linux. Two limits come from the platform:

- **Stop on Windows.** Mortise ends the whole process tree it started. When a launcher such as
  `herd.bat` started `php.exe` and already exited, there is no tree left to walk, and that PHP process
  can go on after Stop. The Windows calls that would hold the whole tree (job objects) are `unsafe` in
  Rust, and Mortise forbids `unsafe` code. On macOS and Linux the process group holds the tree, and a stop reaches every
  member.
- **Closing Mortise.** Closing the window stops every run and waits up to about 5 seconds for each to
  end. The runs are kept in the history as stopped. The Windows limit above applies here too.
