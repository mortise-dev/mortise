# Proposal

## Why

Milestone 29 added the rule allowed use to `mortise.json`, `mortise check` and the push document. The
window showed its findings in the lists only: S5.3 had no row for the rule, and the graph did not mark a
link that breaks it. Milestone 30 of the phase 5 plan draws the rule in the window. Questions 3 and 4 of
the plan stand: no list editor in the window, and the third row is added to `docs/spec` first.

## What Changes

- `docs/spec` draws the third row of S5.3 and holds the text of a break in section 10.
- The rules box of S5.3 gets the switch "A module uses only the modules of its list" and a line with the
  count of modules that have a list. The switch writes `rules.allowed-use.enabled` and keeps the lists.
  Without the key the switch is off and the line says that no module has a list. The first change asks
  before `mortise.json` is created, as for the other rules.
- A link outside the list of its module breaks a rule in the window: `Link::is_problem` is true for it,
  so the graph of the app, the graph of one module and the table of links mark it as they mark the
  other rules. "Links that break a rule" lists it. The files of the link (S5.1) are all marked, the head
  says "Outside its list, 2 files" and the footer names the rule and the list.
- The findings of the rule take the text of the spec. Enter on a break opens the Boundary on the link.

Out of scope: an editor for the lists, and any change to the push document.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `boundary-graph`: a link outside the list of its module is marked as a link that breaks a rule.
- `rules-file`: the window has the switch of the rule allowed use.

## Impact

- `mortise-core`: `Link` gains `outside_allowed`, worked out where the links are built. The findings of
  the rule read it. It is not serialized, so the push document does not change.
- `mortise-app`: the Boundary tab, the Findings list and their strings.
- `docs/spec`, the docs site pages on Boundary, Findings, `mortise.json` and adding rules.
