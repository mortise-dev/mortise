# Tasks

## 1. Core

- [ ] 1.1 Add `runtime/modules.rs` with `ModuleRuntime`, `RuntimeByModule` and `by_module`, behind the feature `actuator`, re-exported from `lib.rs`. Verify with `cargo build -p mortise-core --no-default-features --features actuator` and without `actuator`
- [ ] 1.2 Test `by_module` once on the recorded `fixtures/actuator/health.json`, served by `StubActuator` and read by the client: `module:blog:storage` belongs to `Blog` with the key `storage`; `Billing` and `Blog` read Up; `Legacy` has none; the app keeps its six components. Add the rank rule (Down, Out of service, Up, Unknown), an error of the `modules` component (Down with its sentence), an error for an unknown name, and an indicator of no module to the same test file. Verify with `cargo test -p mortise-core`
- [ ] 1.3 `StubActuator::healthy()` adds the recorded `module:billing:gateway` and `modules` components. Verify with `cargo test --workspace`

## 2. Index

- [x] 2.1 Draw the runtime stroke mark 8 px left of the finding mark on two-line and one-line rows. The finding mark is unchanged. Verify in the image tests of task 4.1
- [x] 2.2 Widen the strip to 96 px while a row it shows has a runtime mark; `strip_width` takes that into account. Verify in the image tests of task 4.1
- [x] 2.3 Give each mark an accessible name and a tooltip. Verify with an assertion in the screen test that already builds the index with an environment, and with `tests/accessibility.rs`
- [x] 2.4 Assert in an existing app test that a Down module raises no pip on the rail. Verify with `cargo test -p mortise-app`

## 3. Summary

- [x] 3.1 The app runtime box lists the app's own components and names each module that is not Up. Verify with an assertion in the existing runtime screen test
- [x] 3.2 The module Summary draws the module runtime box above the tasks box when the module has a status, and none otherwise. Verify with an assertion in the existing runtime screen test, with hidden components and with no module component

## 4. Strings, images, deviations

- [ ] 4.1 Interface text in the `strings.rs` of each feature. Images: the marks on two-line and one-line rows and on the strip, the module box, the module box with an error, light and dark. Run `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images` and look at each changed or new image
- [ ] 4.2 Rows in `docs/deviations.md`: the strip width, the place of the module box, the module lines in the app box. Each says how it was checked

## 5. Integration

- [ ] 5.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass on macOS. Windows and Linux through CI
- [ ] 5.2 `openspec validate phase-3-runtime-per-module --strict` passes

## Workflow follow-up

- Pull request to `main` with a Sol review; merge once CI passes on Linux, macOS and Windows.
- Archive the change through its own pull request, with no reviewer.
