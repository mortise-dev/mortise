# Proposal

## Why

Mortise Cloud asks for a third boundary rule (its request R9): a list of the modules each module may
use. The cloud app needs it for its own layering, and customers get the same. The owner chose option (c)
of the phase 5 plan, section 2: the list lives in `mortise.json` beside the two rules of today, and
nwidart/larch reads the same key. Milestone 29 builds the key, the check, `mortise check` and the push
document. The window comes in milestone 30.

## What Changes

- `mortise.json` gains `rules.allowed-use`: `{ "enabled": true, "modules": { "<module>": ["<module>", ...] } }`.
  Without the key the rule does nothing, so the apps of today read the same.
- A module with a list may use the modules of that list only. An empty list means no module. A module
  without a list is not checked. A module that is off breaks no rule, and no link to it does.
- A new `Check` variant `AllowedUse`, a rule. A break is a problem with the detail `OutsideAllowedUse`
  (the module it may not use and how many files use it), counted on the module that uses.
- A name in the key that is no module of the app is a warning with the detail `UnknownAllowedModule`,
  so that a typing mistake does not pass in silence.
- `mortise check` marks each file of a break on its first line, in the text and github formats.
- The push document carries the new check and the key. `SCHEMA_VERSION` stays 1.
- A new fixture `fixtures/layered`: one break, one module without a list, one empty list.

Out of scope, by the line with Mortise Cloud: no baseline and no "new problems only". A team adopts
the rule one module at a time. No list for every module at once and no patterns in names. No screen:
the window shows the new findings in its lists, and milestone 30 draws the rule on S5.3 and S5.7.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rules-file`: the key `rules.allowed-use`, how it is read and saved, and what the rule checks.
- `cli-check`: the lines of a break and of a name that is no module.
- `push-document`: the new check and the key travel without a new schema version.

## Impact

- `mortise-core`: `boundary/rules.rs` gains `AllowedUse` and `Rules::allowed_use`; `findings.rs` gains
  `Check::AllowedUse`, `Detail::OutsideAllowedUse` and `Detail::UnknownAllowedModule`; `push/` names
  the two details. No new dependency and no new feature.
- `mortise-cli`: `check.rs` and `strings.rs` place and word the two details.
- `mortise-app`: the smallest arms for the new variants, so that it compiles. Their text is not in the spec.
- `fixtures/layered`, the docs site, `CHANGELOG.md`.
