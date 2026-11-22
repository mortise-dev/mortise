# Proposal

## Why

Phase 2 makes Mortise run the commands of `nwidart/laravel-modules` in an app. Before it starts, the
project rules, the spec and six points left from the council review of PR #3 must match what phase 2
builds on. Stage 0 of `docs/plan/phase-2.md` does this groundwork.

## What Changes

- The rule "Mortise reads the files of an app. It needs no PHP and runs nothing in the app." in
  `AGENTS.md` and `openspec/config.yaml` becomes: "Mortise runs a command in the app only when the user
  starts it, with the exact command on screen first, through the PHP the user chose for the app.
  Reading an app still needs no PHP." The `mortise.json` rule stays. Files in the app are written by
  the package's commands, not by Mortise (plan, question 5).
- Section 12.2 of the spec follows the packages: P9 and A1 met, P10 and A2 met in part, 43 generators,
  and a prohibit flag for each destructive command. S9.1 reads "42 generators for Billing, and New
  module" (plan, question 9).
- The app name set in Settings reaches Findings, Boundary and the palette. One function gives the
  display name of an app.
- The Add panel scrolls when its content does not fit, for example at 150% text size.
- The other members of a circle keep the count of Q4. Their Summary gets one line: "Part of a circle
  counted on Checkout" (not in the spec).
- A rule finding counts only when both modules of the link are on. Boundary still draws the link, in
  the off style.
- Files in `tests/` are left out of links, circles and rules.
- The graph of 12 modules draws the code stamp in a node and the name only when it fits. The full name
  is the tooltip and the accessible name. A deviation records it.

## Capabilities

### New Capabilities

- `app-access`: what Mortise may read, write and run in an app.
- `app-display-name`: the name each screen shows for an app.
- `module-links`: which files make links and circles, and which links count as rule findings.
- `boundary-graph`: how the graph of an app draws its module nodes.
- `add-panel`: the layout of the Add panel.

### Modified Capabilities

None. `openspec/specs/` holds no capability yet.

## Impact

- Docs: `AGENTS.md`, `openspec/config.yaml`, `docs/spec-build` and the rebuilt `docs/spec`,
  `docs/deviations.md`.
- `mortise-core`: links, circles and rule findings (`links.rs`, `rules.rs`, `findings.rs`, and the
  reading of a module's files).
- `mortise-app`: `findings_view.rs`, `boundary.rs`, `palette.rs`, `summary.rs`, `add.rs`, `strings.rs`,
  and the image snapshots that change.
- No new dependency. No release (plan, question 8).
