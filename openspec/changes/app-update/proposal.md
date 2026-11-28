# Proposal

## Why

The desktop app cannot learn that a newer version exists. A user stays on the version they first
downloaded until they visit the releases page. Issue 92 asks for an update from the window. The owner
decided two things on that issue: the window installs an update after one click, and the check for a
newer version runs by itself at start, at most once a day, with a setting that turns it off.

## What Changes

- The window asks GitHub for the update manifest of the newest release: at start, at most once a day, and
  when the user presses "Check now". This is the one request Mortise makes by itself. It carries nothing
  of an app.
- General gains the row "Updates": the choice "At start" or "Off", where the update is, and one button.
- A newer version shows in a toast that stays. "Install and restart" downloads it, checks its signature
  with a public key built into the app, replaces the app and starts it again. Nothing is downloaded before
  the press, and the install waits while a command runs in an app.
- A copy that cannot replace itself gets a link to the download page instead.
- A release signs the macOS app archive, the Windows setup and the AppImage with the update key, and
  publishes the update manifest `latest.json`. A release without the key, or with another key than the one
  in the source, fails before it publishes.
- `settings.json` gains `general.check_for_updates` and `update` (when the last check ran, and the newer
  version it found).
- New dependency of `mortise-app`: `cargo-packager-updater` 0.2.3, with `reqwest` behind it.

Out of scope: the command line tool, an update that installs without a press, release notes in the window,
a channel for pre-releases, and builds for platforms that have no installer today.

## Impact

- `crates/mortise-app`: new `update/` and `app/update.rs`; `settings/view.rs`, `ui/toast.rs`, `app/mod.rs`.
- `crates/mortise-core`: `Settings` and `General`.
- `.github/workflows/installers.yml`, `tag-release.yml`, `tools/sign-updates.sh`: a new secret pair in the
  `release` environment.
- Docs: `docs/deviations.md` S80, `CONTRIBUTING.md`, the site, `README.md`, `AGENTS.md`.
- The text "Mortise sends nothing by itself" is no longer true and is corrected where the site says it.
