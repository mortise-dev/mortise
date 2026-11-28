# Tasks

## 1. Specification

- [x] 1.1 This change, with the new spec `app-update`. Verify with `openspec validate --strict`

## 2. App

- [x] 2.1 `update/mod.rs`: `Channel`, `Release` on `cargo-packager-updater`, `place()`, and `Updates` with its phases. Verify with `update::tests`: a download is kept only with the signature of the key for that version, and installs only once no command runs
- [x] 2.2 `General::check_for_updates` and `UpdateCheck` with its own file in `mortise-core`. Verify through `tests/settings.rs`
- [x] 2.3 `app/update.rs`: the check at start, the day between two checks, the toast, the install that waits for commands, the restart
- [x] 2.4 The row "Updates" in General and a done toast that stays. Verify with `tests/settings.rs`, the audit of General in `tests/accessibility.rs` and the image S8.3

## 3. Release

- [x] 3.1 `installers.yml`: pack the notarized app and check it unpacked; the job `sign-updates`
- [x] 3.2 `tools/sign-updates.sh`. Verify by a run with a key made for it, with the wrong key and with no key
- [x] 3.3 `tag-release.yml` passes the key and publishes `latest.json`
- [ ] 3.4 The owner makes the update key, stores its private half in the `release` environment, and its public half goes into `PUBLIC_KEY`

## 4. Docs

- [x] 4.1 `docs/deviations.md` S80, `CONTRIBUTING.md` (Update key), `AGENTS.md`
- [x] 4.2 The site: the how-to "Update Mortise", Settings, installation, safety, files and folders, report a problem; `README.md`
- [x] 4.3 `licenses/THIRD-PARTY.md`

## 5. After the first release with this change

- [ ] 5.1 Update from that release to the next on macOS, Windows and the AppImage, and record the result in `docs/deviations.md` S80
