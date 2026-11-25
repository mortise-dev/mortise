---
title: Reading an app without PHP
description: How Mortise learns what is in a Laravel app from its files and a PHP parser, what it cannot know that way, and how it stays current.
sidebar:
  order: 1
---

Mortise shows the modules of an app without running the app. It reads files and parses PHP source,
but it does not call `artisan` or any other PHP to do so. This page explains why, how it works, and
what that choice leaves out.

## Why not ask artisan

The package already knows a great deal about an app. `php artisan module:list` prints the modules
and their state. So why not ask it?

To run artisan, Mortise would first need a working PHP for the app. That PHP can be Herd, Sail, a
Docker container or a binary on the path, and each needs to be set up and running. Artisan also boots
the whole app: it loads the service providers, reads the configuration and may connect to a database.
A broken provider, a missing `.env` or a stopped container would leave Mortise with nothing to show.

Reading files avoids all of that. You can add an app the moment you clone it, before PHP works for it.
A read runs no code of the app, so it cannot change a database or a cache. It is also fast: the
folder is read on a worker thread, and the window stays responsive while it works.

The trade-off is that Mortise sees what is written, not what PHP would compute. The rest of this page
is about where that line falls.

## What Mortise reads

| Source | What Mortise takes from it |
|---|---|
| `composer.lock` | The versions of `laravel/framework`, `nwidart/laravel-modules` and `nwidart/laravel-actuator` |
| `config/modules.php` | Where the modules live, their namespace, the folder of their classes, the activator and the statuses file |
| The statuses file, `modules_statuses.json` by default | Which modules are on |
| Each `module.json` | The name, description and priority of each module |
| The PHP files of each module | The classes each file declares and the classes it names |
| `.env` | `APP_URL` and `APP_ENV` only. No other line is read |
| A clover report, such as `build/coverage/clover.xml` | Test coverage per module |
| `git` | The branch and how often each module changed in 30 days |

Git is the one program Mortise runs during a read. It only reads. Without git, or outside a repository,
that data is absent and nothing else changes.

## The PHP parser

Mortise parses PHP with the parser of the Mago project, written in Rust. It reads two kinds of file.

The first is `config/modules.php`. The parser turns the returned array into plain values. A string,
a number, a boolean or `base_path('...')` can be known from the text. A call to `env()`, a
concatenation or a constant cannot, because only PHP can work out its value.

The second is each PHP file of each module. The parser resolves every name the way PHP does: through
`use` statements, aliases and the namespace of the file. It records the classes, interfaces, traits
and enums the file declares, and every other class the file names, with the first line that names it.
These names become the links between modules. [About module boundaries](/mortise/explanation/boundaries/)
explains what Mortise does with them.

A file with a syntax error does not stop the read. The parser gives what it can read around the error.

The parser's interface changes between minor versions. So all use of it stays in one place in the code,
and the version is pinned. The rest of Mortise sees plain data only.

## What Mortise cannot know

Some facts exist only while PHP runs. Mortise does not guess them; it says what it does not know.

- **A setting that needs PHP.** When a key of `config/modules.php` holds `env()` or another
  expression, Mortise uses the package default for it. The Summary of All modules then says which keys
  it assumed. If the modules folder is one of them and your modules live elsewhere, the list is wrong,
  and the Summary says so.
- **An activator that is not a file.** The package can keep the state of modules in a database or
  elsewhere. Mortise reads files only, so it then cannot tell which modules are on. The Summary
  says so, and the state reads "Unknown".
- **A class built at run time.** A class name put together from strings, or resolved from the
  container, is not a name in the source. It makes no link.
- **Routes and scheduled tasks.** These come from code that runs. Mortise reads them from a running app
  through the actuator. [About the runtime of each module](/mortise/explanation/runtime-per-module/)
  covers that.
- **The migration state.** Which migrations ran is in the database. With `nwidart/laravel-modules` 13.4
  or later, Mortise can ask the package for it, through PHP, when you press "Read migration state".
  That is a command you start, not part of a read.

## Staying current

An app changes while you work on it. Mortise watches the app folder and reads it again when a file
changes.

The watcher collects changes for 300 ms before a read starts, so a save that touches many files leads
to one read. A read after a change parses only the files that changed. Files in `vendor`,
`node_modules` and `storage` are ignored, and so is most of `.git`: only the files that move when you
commit or switch branch count. A coverage report counts wherever it is written.

Each read gives a new snapshot of the app. The window keeps showing the last snapshot until the next
one is ready, so the screen never shows a half-read app. You can also read the app again by hand with <kbd>Cmd</kbd> <kbd>R</kbd> on macOS
(<kbd>Ctrl</kbd> <kbd>R</kbd> on Windows and Linux).

Watching is a feature of the core that a build can leave out. A tool that reads an app once does not
need it.
