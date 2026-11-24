# Tasks

Each group runs in its own worktree and branch, from the milestone branch `change/phase-3-hardening`
after group 1 merged there, except where it says otherwise. The lead reviews each group before it merges.

## 1. The 24 hour memory test and the figure cost (Opus)

- [x] 1.1 Add the `#[cfg(test)]` count of bytes held to `MetricsStore` (design D1); verify with a short check in the new test that an empty store holds less than a store with one read
- [x] 1.2 Add the 26 hour test of D1 in `metrics/sample.rs`; verify it passes in a debug build in under 2 s and fails when the bound is lowered by one row
- [x] 1.3 Measure the resident size of the test binary with and without the 26 hour run (`/usr/bin/time -l`, release, three runs); write the numbers under "Measured" in `design.md`
- [x] 1.4 Time `Figures::of` on a full 24 hour window of the 60 module case (D2); write the numbers and the decision under "Measured"; if the slowest case is 8 ms or more, move the work as D2 says and verify with `cargo test --workspace`

## 2. The real-actuator CI job (Opus; branch from the milestone branch, its own PR to `main`)

- [x] 2.1 Write `tools/real-actuator-app.sh` and the PHP files in `tools/real-actuator/` (D3); verify by running it on the Mac and serving the app: `health` lists `module:blog:*` and `module:billing:*`, `prometheus` answers 200 after traffic
- [x] 2.2 Write `crates/mortise-core/tests/real_actuator.rs` with the ignored tests of D3; verify they pass against the app of 2.1 on the Mac, the empty answer and the 503 included, and are skipped by a plain `cargo test --workspace`
- [x] 2.3 Add the job "Real actuator (Linux, not required)" to `ci.yml` (D3); verify it passes on the pull request and that no required job changed
- [x] 2.4 Add the job and the script to `AGENTS.md` (Commands) and `CONTRIBUTING.md` if it lists jobs; verify the commands written there run as written

## 3. The environment switch (Opus)

- [x] 3.1 Give `draw::segmented` the wrapping form of D5 with its height; verify with a unit or screen test that eight 30-character names at 900 px wrap and every segment is inside the width
- [x] 3.2 Use it in S7.3 and in the runtime box (D5); verify in `tests/metrics.rs` or `tests/runtime.rs` (the test that already builds environments) that each environment can be chosen by pointer and by keyboard
- [x] 3.3 Add one image of S7.3 and of a runtime box with eight long environment names, both themes; look at each

## 4. Images and the accessibility sweep (Sonnet)

- [x] 4.1 Store images for S2.1, S4.1, S5.2, S5.6, S6.5 b to i, S8.1, S8.5 and the S4.2 tasks box with its Module column (D4); look at each, light and dark; list any id left out with its reason under "Images not stored" in `design.md`
- [x] 4.2 Fix the S7.3 day image so it reads "read every 15 s" (D4, in the test only); look at it
- [x] 4.3 Draw at most one point per pixel column in a sparkline (D4); regenerate `s7.3-metrics-all-*` and look at the "Last hour" column
- [x] 4.4 Add every phase 3 control to `tests/accessibility.rs` that is not there (runtime marks, runtime box, environment switch, range switch, chart, table, "Try again", "Open settings"); verify the test passes

## 5. The interface text sweep (Sonnet)

- [x] 5.1 Grep every `view.rs`, `app/*.rs` and drawing file of `mortise-app/src` for literal interface text (D7) and move each to its `strings.rs`; verify with the grep that none is left and with `cargo test --workspace` and the images that nothing visible changed

## 6. The spec-build scripts (Sonnet)

- [x] 6.1 Make `build.mjs`, `shoot.sh` and `offs.sh` write inside the repository or stop (D6); verify each with an output path outside the repository (stops with the message, writes nothing) and one inside (writes there)

## 7. The look on Linux and Windows (Sonnet steps; the lead runs the box)

- [ ] 7.1 Ask the orchestrator before using the box; check the lock; shoot the index with runtime marks, a module Summary with its runtime box, S7.1, and S7.3 if the recorded answers can be served (D8); look at each shot
- [ ] 7.2 Record each difference from macOS as a deviation row checked "run on Linux"; record Windows as built and tested in CI, not looked at; look at D1 to D3 and D5 only where the look shows them

## 8. Docs (Sonnet)

- [x] 8.1 `README.md`: what phase 3 adds (the runtime per module, the Metrics tab) and the actuator setup an app needs; verify every command and config key named there against `fixtures/actuator/README.md`
- [x] 8.2 `docs/plan/phase-3.md`: section 4 says reads past the last hour are thinned to one per 5 minutes; add "What the build did" (PRs #30 to #42 and this milestone's PRs, and the differences from the plan)
- [x] 8.3 `docs/deviations.md`: rows added or changed by groups 3, 4 and 7; verify each says how it was checked
- [x] 8.4 Verify `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passes, and `openspec validate phase-3-hardening --strict`

## Workflow follow-up

- Open each PR to `main`, get a Sol review on each code PR, handle the findings, merge once CI passes on
  Linux, macOS and Windows.
- Archive the change through its own PR with no reviewer.
- Send the final report to the phase 3 orchestrator.
