# Proposal

## Why

The Module column of scheduled tasks on S4.2 and the tasks on a module's Summary (S4.3) place a task in
a module only when its target is a class name. Most tasks are artisan commands, such as
`php artisan billing:send-invoices`, so they show "none". The package does not list the tasks of a module
yet (P10, PKG-2 in `docs/plan/phase-3.md`). Decision 9 of the plan: match a task `php artisan <command>`
to the module that declares that command. Milestone 22 builds this.

## What Changes

- The snapshot of a module holds the names of the console commands it declares, outside its tests.
  They come from the files the read of an app already parses. The rule that finds a command name in a
  class is the one the public surface scan uses today; it moves to one function that both use.
- A scheduled task belongs to a module when its target is a class of the module (as today), or when its
  target is `php artisan <name> ...` and the module declares `<name>`. A task with a description, a
  closure or a shell command stays "none".
- One function places a task. The Module column of S4.2, the tasks of a module's Summary and the
  schedule of a command in the public surface scan all use it. When the package lists the tasks of each
  module (PKG-2), that function is where its answer wins. No code for PKG-2 is written now.
- The note of S4.2 on P10 says that command tasks are placed and described tasks wait for P10.
- The cost of reading command names when an app is read is measured on `fixtures/shop-api` and on a
  generated app with 60 modules, before and after, and written in `design.md`.

## Capabilities

### New Capabilities

- `module-tasks`: which module a scheduled task belongs to, and where Mortise shows it.

### Modified Capabilities

None.

## Impact

- `mortise-core`: `modules/module.rs` (the command names of a module), `app/mod.rs`,
  `runtime/live.rs` (the place of a task), `surface.rs` (uses the shared rules), `lib.rs`.
- `mortise-app`: `runtime/view.rs` (the tasks box only) and `summary/view.rs`. No new text.
- Tests: one core test of the match; assertions in the screen tests that already render these boxes.
- Spec text: the note of S4.2 on P10 says which tasks are placed today. `docs/spec` is rebuilt from
  `docs/spec-build/p_read.py`. The spec already marks the column as limited, so no deviation is added.
- No new dependency. No change to what the actuator client asks for. No release (decision 11).
