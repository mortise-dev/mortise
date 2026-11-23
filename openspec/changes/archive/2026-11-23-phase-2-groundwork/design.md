# Design

## Context

See `proposal.md` for the reasons. The six code points come from the council review of PR #3 and
question 7 of `docs/plan/phase-2.md`. Links, circles and rule findings are built in `mortise-core`.
The screens in `mortise-app` only draw them.

## Goals / Non-Goals

**Goals:**

- Each rule of `specs/` is held at the layer that owns it and tested once there.
- The visible changes are checked in the image tests on macOS.

**Non-Goals:**

- Nothing of milestones 10 to 16: no command runs, no PHP settings.
- No release (plan, question 8).

## Decisions

- **Test files.** The reader of a module already knows which files are tests (the test count uses it).
  Links skip those files where links are built, so circles and rule findings follow with no second
  check. The "tests" test of a module stays as it is.
- **Off modules.** The link keeps its references. The decision "this link is a finding" checks that both
  modules are on, in the core, where `is_problem` and the circle findings live. Boundary reads the same
  state to draw the off style. A circle with a member that is off makes no finding: the circle does not
  run.
- **Circle line.** The core already knows the carrier of each circle (Q4). The Summary of each other
  member asks for it and shows one line. The text is in `strings.rs`, marked "not in the spec".
- **Display name.** One function on the settings entry, or beside it, gives the display name. Every
  screen calls it instead of reading `app.name`. Alternative: write the Settings name into the snapshot.
  Rejected: the snapshot is what was read from the files.
- **Graph nodes.** The node draws the code stamp first. The name is drawn only when the whole galley fits
  the room left; else nothing is cut. The tooltip and `WidgetInfo` carry the full name.
  A row in `docs/deviations.md` records the difference with S5.
- **Add panel.** The panel body goes in an `egui::ScrollArea` with the C16 scrollbar style.
- **Spec 12.2.** Edited in `docs/spec-build` (`data.py`, `p_open.py`, `p_write.py`), then
  `docs/spec` is rebuilt with its script. The HTML is never edited by hand.

## Risks / Trade-offs

- Tests in fixtures that relied on links from `tests/` change their counts. Mitigation: update the
  assertion in the test that builds that fixture, not a new test.
- Images change for Boundary, Summary and the Add panel. Mitigation: look at each changed image before
  it is committed.
- Parallel builders touch `strings.rs` and `boundary.rs`. Mitigation: merge one at a time and resolve
  each conflict in review.
