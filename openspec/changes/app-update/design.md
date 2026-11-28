# Design

## Context

cargo-packager builds the installers. Its sister crate `cargo-packager-updater` checks a manifest, downloads,
verifies a minisign signature and replaces the app on macOS (`.app`), Windows (NSIS setup) and Linux
(AppImage). Releases live on GitHub, which serves `releases/latest/download/<name>` for the newest release.

## Decisions

**D1. Use `cargo-packager-updater`, not own code.** Downloading, verifying and replacing a running app is
the part where a mistake costs most, and the crate matches the packages cargo-packager builds. The cost is
`reqwest` and about 56 more crates in the desktop app. The command line tool gets none of them.

**D2. The update code lives in `mortise-app`.** `mortise-core` is published and the command line tool
builds on it. The code replaces the desktop app only. It draws nothing: `update/mod.rs` holds the logic,
`settings/view.rs` and the toast draw.

**D3. The manifest is a release asset.** `latest.json` at `releases/latest/download/latest.json`. No server
of our own, and the request shows GitHub nothing but the address it comes from. A draft or a pre-release is
not "latest", so it reaches no window.

**D4. A `Channel` trait stands between the window and the releases.** `Release` is the real one. The tests
of the window bring releases of their own, so no test needs a network or replaces a program.

**D5. Check, then download on a press, then install when nothing runs.** The download asks for the manifest
again, so the address and the signature are those of that moment. The install closes Mortise, and on
Windows the crate ends the process itself, so the window starts it only when no command runs in an app.

**D6. Once a day, by what `settings.json` remembers.** `update.at` is when the last check ended and
`update.newer` what it found. A start within the day asks nobody and offers the remembered version again,
if it is still newer than the running one.

**D7. A copy that cannot replace itself gets a link.** `place()` names what an update replaces: the `.app`
on macOS, unless macOS runs it from a translocated place or its folder cannot be written; the AppImage on
Linux in a folder the user can write; the installed program on Windows, known by `uninstall.exe` beside
it. A `.deb` install and a build from source are none of these.

**D8. The update key meets no build.** The job `sign-updates` downloads the built files, signs them with
`cargo packager signer sign`, verifies each signature with `minisign` against `PUBLIC_KEY` of the source,
and writes the manifest. It sets `environment: release`. The macOS archive is made in the job that
notarized the app, and the unpacked app must pass `tools/verify-macos-app.sh` again.

## Risks

- A lost private key strands the installed windows: they must be updated by hand once. `CONTRIBUTING.md`
  says to keep a copy.
- No release with a manifest exists yet, so no update ran end to end. The second release after this change
  is the first that can prove it. `docs/deviations.md` S80 lists what was not run.
- `cargo-packager-updater` sets two environment variables on Linux from a worker thread.
