# Proposal

## Why

Phase 3 reads the running app per module, over HTTP, from the actuator of the chosen environment.
Before it starts, the project rules, the spec and the tests must match what phase 3 builds on.
Milestone 18 of `docs/plan/phase-3.md` does this groundwork. It settles the top risk of the phase on real
answers: whether a series of the `prometheus` endpoint can be placed in a module through `mappings`.

## What Changes

- The actuator rule (decision 1 of the plan) becomes: "The actuator client asks for the actuator base
  path, `health`, `info`, `mappings`, `scheduledtasks` and `prometheus` only. It never asks for `env`
  and follows no redirect." It is written in `AGENTS.md`, `README.md`, `openspec/config.yaml` and the
  doc comment of `runtime/actuator.rs`. No code asks for `prometheus` yet; milestone 20 does.
- A new capability `actuator-client` specifies what the client does today, and the new rule.
- `fixtures/actuator/` holds answers recorded from one real Laravel app with `nwidart/laravel-modules`
  13.4.0, `nwidart/laravel-actuator` 1.3.1 and `nwidart/laravel-metrics` 1.x. A README gives the
  command that made each file and the package versions.
- `design.md` records what the answers show: how `http_route` relates to the `uri` of `mappings`, how
  job names relate to module namespaces, what `prometheus` answers to `Accept: application/json`, and
  the size of an answer.
- Spec text (decisions 8 and 10): S7.2 reads "Add prometheus to exposure.include in
  config/actuator.php, next to health." Section 12.2 follows laravel-modules 13.4.0 and
  laravel-actuator 1.3.1. Sections 2.5 and 12.2 name phase 4 for M1 and P6. `docs/spec` is rebuilt from
  `docs/spec-build`. Deviation S6 follows.
- Interface text outside `strings.rs` moves into the right `strings.rs`: the S7.4 block of the Metrics
  tab, the tab labels, "Open in editor", and the text of the Settings view with its table of shortcuts. The S7.4 paragraph says the actuator has no metrics
  endpoint, which is no longer true; it is corrected and marked "not in the spec".

## Capabilities

### New Capabilities

- `actuator-client`: what the actuator client asks for, how often, and how it reads each answer.

### Modified Capabilities

None. `code-layout` already requires interface text in `strings.rs`; this change makes the code meet it.

## Impact

- Docs: `AGENTS.md`, `README.md`, `openspec/config.yaml`, `docs/spec-build` and the rebuilt
  `docs/spec`, `docs/deviations.md`.
- `mortise-core`: the doc comment of `runtime/actuator.rs`. No behaviour change.
- `mortise-app`: `app/workspace.rs`, `app/strings.rs`, `settings/view.rs`, `settings/strings.rs`,
  `boundary/view.rs`, `strings.rs`, and the image of the Metrics tab (S7.4).
- `docs/spec-build/lib.py` writes to `docs/spec`. `.gitattributes` keeps the fixtures' bytes.
- New folder `fixtures/actuator/`. No test reads it yet; milestone 20 serves it from `StubActuator`.
- No new dependency. No release (decision 11).
