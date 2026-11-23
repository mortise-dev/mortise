# Design

## Context

See `proposal.md` for the reasons. Today `tasks_of` and the Module column of the tasks box
(`runtime/view.rs`) place a task with `module_of`: the module whose namespace starts the target. The
actuator writes the target of a task as its description, else its command, else the class of its job
(`fixtures/actuator/scheduledtasks.json`). laravel-actuator 1.3.1 writes a command task through
`Event::normalizeCommand`, which turns `'/usr/bin/php' 'artisan' blog:digest` into
`php artisan blog:digest` on every platform (checked: source read of
`src/Support/ScheduledTaskTarget.php` and the recorded answer).

The read of an app (`modules/module.rs`, `scan`) already parses every PHP file of each module once, on
all cores, and keeps the analysis of each file. Each declared class there already carries its command
name: `php::Declared::command` (signature or attribute) and `name_property` (`$name`). The public
surface scan (`surface.rs`) reads the same fields, with one rule: `$name` counts only in the commands
folder.

## Goals / Non-Goals

**Goals:**

- One rule finds a command name in a class; the read and the surface scan both use it.
- One function places a task in a module; the screens and `tasks_of` use it.
- The first read of an app is not slower.

**Non-Goals:**

- A task with a description or a closure is not placed (decision 9). The package's list of the tasks of
  each module (P10, PKG-2) is not read; no endpoint, field or flag is added for it.
- No change to what the actuator client asks for.
- No new interface text. "none" and the stamp are the ones the column shows today.

## Decisions

- **Command names come from the analyses the read already holds.** `scan` collects
  `command_of(declared, folder)` of each file outside the tests into `Scan::commands`; the snapshot
  gives them as `Module::commands`, sorted. Alternative: run the surface scan's reader per module during
  the read. Rejected: it reads and parses each file a second time. Alternative: parse command names
  lazily when a task is placed. Rejected: placing runs every frame in the view, and the analyses are
  already in memory.
- **`command_of` lives in `modules/module.rs`.** The surface scan imports it and drops its own copy of
  the rule and of the folder name `Commands`, which moves to `modules/kind.rs` beside `TESTS`.
- **`artisan_command` lives in `modules/module.rs` too.** It is pure text and must build without the
  feature `actuator`, because `surface.rs` always builds. It accepts `php artisan <name> ...`, the one
  form the actuator writes, and strips quotes around `<name>`. Anything else gives no command.
- **`module_of_task` is the one place.** It tries the class (as today), then the artisan command. The
  first module in the order of the snapshot (by name) wins when two declare the same command. When
  PKG-2 lands, its list is checked first in this function; nothing else changes.
- **The surface scan uses `artisan_command` for the schedule of a command.** Before, a target matched
  when any word of it equalled the command name, so a description that named a command matched too. Now
  the scan and the Module column agree on which task runs a command.

## Cost of reading command names

Measured on macOS (Apple M3 Max, 16 cores), release build, with a throwaway example that is not
committed: `read_app` on a fresh cache, once to warm the file system, then 21 timed reads; the median
and the fastest of each run, three runs before the change and two after.

The generated app is `fixtures/shop-api` with 53 copies of its Billing module renamed `Extra1` to
`Extra53`: 60 modules, 978 PHP files, 54 modules with two console commands each.

| App | Before: median, fastest | After: median, fastest |
|---|---|---|
| `fixtures/shop-api` | 86.5 ms, 80.0 ms / 90.1 ms, 81.0 ms / 87.0 ms, 82.2 ms | 92.7 ms, 83.3 ms / 87.7 ms, 81.6 ms |
| 60 modules | 113.9 ms, 109.1 ms / 112.6 ms, 108.8 ms / 114.0 ms, 109.9 ms | 118.2 ms, 108.7 ms / 115.3 ms, 109.6 ms |

The fastest reads differ by less than 1.5 ms, inside the spread between runs of the same build. The
change adds no file read and no parse: one pass over the declared classes already in memory.

## Risks / Trade-offs

- [A command registered under another name at run time, such as through `setName()` in a constructor]
  The read does not see it; the task stays "none". The surface scan has the same limit today.
- [Two modules declare one command name] Laravel keeps the last one registered. Mortise shows the first
  module by name. Rare; PKG-2 settles it.
- [A target written by another actuator in another form, such as `'php' 'artisan' x`] Not placed.
  laravel-actuator 1.3.1 always normalises; the recorded answer shows the form.
