# Tasks

## 1. Core

- [x] 1.1 Move the rule that finds a command name in a declared class to one function in `modules/module.rs`. The surface scan uses it
- [x] 1.2 Add `commands` to `Module`: the command names of the module outside its tests, from the analyses the read already holds
- [x] 1.3 Add `module_of_task` to `runtime/live.rs`: a class of the module, else `php artisan <name>` with `<name>` a command of the module. `tasks_of` uses it; `SurfaceScan::finish` uses the same `artisan_command`
- [x] 1.4 Test the match once, in the core test that already builds the shop-api answers (`tests/actuator.rs`)

## 2. App

- [x] 2.1 The Module column of the tasks box (`runtime/view.rs`) uses `module_of_task`
- [x] 2.2 Assert the column and the Billing Summary in the screen tests that already render them (`tests/runtime.rs`)

## 3. Spec text

- [x] 3.1 Update the note of S4.2 on P10 in `docs/spec-build/p_read.py` and rebuild `docs/spec`

## 4. Measure and verify

- [x] 4.1 Measure the first read of `fixtures/shop-api` and a generated 60-module app, before and after, in release. Write the figures in `design.md`
- [x] 4.2 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [x] 4.3 Run `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images` and look at each changed image before it is committed. No image changed: no stored image shows the tasks box
- [x] 4.4 `openspec validate phase-3-module-tasks --strict` passes
