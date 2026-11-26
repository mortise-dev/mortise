# Proposal

## Why

Milestones 24 to 27 built `mortise check`, `mortise push`, the distribution of the tool and the accepted
public surface. Section 9 of the phase 4 plan leaves one question open: whether the paths in the output
open in editors and CI logs on Windows. Milestone 28 settles it and closes phase 4: the built tool runs
on the three platforms in CI, the new screens have images and pass the accessibility audit, and the
docs site and the deviations cover what phase 4 built.

## What Changes

- The job "Command line tool" of `ci.yml` runs the built `mortise` on the fixture shop-api on Linux,
  macOS and Windows: `check` in each format, a folder that is not an app, and `push --dry-run` without
  a token. It checks the exit codes, the circle, the three reads of Billing internals, the accepted
  surface of Users, and that no path names a drive, a root or a backslash.
- A path outside the app folder, such as a modules folder set to an absolute path, was printed as
  `//shared/Modules` on Unix and `C:/\/shared/Modules` on Windows. It is printed whole, with `/`
  between the parts: `/shared/Modules`, `D:/shared/Modules`. The two copies of the function that
  shows a path become one, in `app/mod.rs`.
- `tests/accessibility.rs` finds the switch "Fail CI when a rule breaks" of S5.3 among the audited
  controls, and audits the dialog of Accept. The other screens of milestones 24 to 27 (S5.7 to S5.9)
  are audited already.
- The images of S5.8 and S5.9 exist in both themes already. No screen changes, so no image changes.
- The docs site: the how-to for CI shows a Windows runner and a pinned version of `setup-mortise`; the
  reference of the command line tool says how a path outside the app is printed.
- `docs/deviations.md`: row S76, the form of the paths in the output and in the push document.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `cli-check`: the paths of the output are written the same on every platform, a path outside the app
  included, and CI runs the built tool on the three platforms.

## Impact

- `crates/mortise-core`: `app/mod.rs` (`shown`), `findings.rs` (uses it).
- `crates/mortise-app`: `tests/accessibility.rs`.
- `.github/workflows/ci.yml`: the job "Command line tool".
- Docs: `website` (two pages), `docs/deviations.md`.
- No new crate and no new dependency. The push document does not change: its paths were relative
  already.
