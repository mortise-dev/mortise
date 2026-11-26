# Proposal

## Why

The second boundary rule asks whether another module reads only the public part of a module. Until now
Mortise could only guess that part from a list of folders, because the package could not store a
module's public surface. laravel-modules 13.5.0 (P6, pull request 2309) adds the `surface` key of
`module.json` and the command `module:surface`. Milestone 27 of the phase 4 plan uses both: the rule
checks an accepted surface, and the window can accept one.

## What Changes

- `mortise-core` reads the `surface` key of each `module.json` without PHP: three lists, `classes`,
  `routes` and `commands`. A key that the package would refuse counts as no surface.
- The second rule checks a module with an accepted surface against it: a class is public when the
  surface lists it, in any case. A module without a surface keeps the folder list. The rule stays in one
  place in the core, so the window and `mortise check` cannot disagree.
- A run can take standard input. The window runs `module:surface <Module> --set --stdin --json` with the
  new surface as one JSON object on standard input, after a dialog shows the exact line.
- S5.7 gets a working Accept. S5.8 shows the accepted surface, with Change and Scan again. S5.9 lists
  what a later scan finds different, and accepts the checked differences.
- Accept is off with its reason when the package is older than 13.5, when no PHP is chosen, or while a
  command runs. A command that stores nothing says why.
- The push document carries the surface of each module as a new member `surface`. Schema version 1
  stays: the contract raises the version only for a member that is removed, renamed or changes type.

Out of scope, by section 1 of the plan: no date of acceptance, no history of surfaces, no owners, and
nothing across apps.

## Capabilities

### New Capabilities

- `public-surface`: the accepted public surface of a module, how the second rule uses it, and how the
  window accepts and compares one.

### Modified Capabilities

- `push-document`: a module carries its accepted surface.
- `command-runs`: a run can take standard input.

## Impact

- `mortise-core`: `surface.rs` (the accepted surface, differences, the answer of `module:surface`),
  `modules/module.rs`, `boundary/links.rs`, `package.rs` (`stores_surface`), `run/process.rs` and
  `run/read.rs` (standard input).
- `mortise-app`: `boundary/surface.rs`, `boundary/strings.rs`, `app/surface.rs`, the reads of
  `app/runs.rs`.
- `mortise-cli`: the sentence of a finding of the second rule names an accepted surface.
- Fixtures: `Modules/Users` of shop-api has a `surface` key; `fixtures/output` has a recording of
  `module:surface` 13.5.0.
- Docs: `docs/deviations.md` S71 to S75, the Boundary reference, the how-to of the public surface, the
  explanation of boundaries and the command line reference.
