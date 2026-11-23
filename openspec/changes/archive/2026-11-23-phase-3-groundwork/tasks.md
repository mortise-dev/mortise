# Tasks

## 1. Rules and capability

- [x] 1.1 Write the rule of decision 1 in `AGENTS.md`, `README.md`, `openspec/config.yaml` and the doc comment of `runtime/actuator.rs`. Verify with `grep -rn "prometheus" AGENTS.md README.md openspec/config.yaml crates/mortise-core/src/runtime/actuator.rs`
- [x] 1.2 Add the capability `actuator-client`. Verify with `openspec validate phase-3-groundwork --strict`

## 2. Recorded answers

- [x] 2.1 Build a Laravel app outside the repository with laravel-modules 13.4.0, laravel-actuator 1.3.1, laravel-metrics 1.x and two modules, each with a health indicator. Expose `health` with details, `info`, `mappings`, `scheduledtasks` and `prometheus`
- [x] 2.2 Make traffic: requests to module routes, a 5xx, a job. Record discovery, `health`, `mappings`, `scheduledtasks` and `prometheus` quiet, busy, empty and 503 into `fixtures/actuator/`, with a README that gives each command and the package versions. Verify that no secret, token, host name or local path is in the folder
- [x] 2.3 Compare `http_route` with the `uri` of `mappings`, job names with module namespaces, the answer to `Accept: application/json`, and the size of an answer. Write the findings into `design.md`

## 3. Spec text

- [x] 3.1 Update `docs/spec-build` for S7.2, section 12.2 and section 2.5. Rebuild `docs/spec` with its script and verify the text is in the built HTML
- [x] 3.2 Update deviation S6 in `docs/deviations.md`

## 4. Interface text

- [x] 4.1 Move the S7.4 text of the Metrics tab from `app/workspace.rs` to `app/strings.rs`. Correct the sentence about the metrics endpoint and mark it "not in the spec"
- [x] 4.2 Move the table of shortcuts from `settings/view.rs` to `settings/strings.rs`. What the user sees does not change

## 5. Integration

- [x] 5.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [x] 5.2 Run `UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images` and look at each changed image before it is committed
- [x] 5.3 `openspec validate phase-3-groundwork --strict` passes
