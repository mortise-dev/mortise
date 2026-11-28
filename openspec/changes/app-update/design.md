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
again, so the address and the signature are those of that moment. The install closes Mortise, so the
window starts it only when no command runs in an app, lets no command start while it runs, and does not
end before it has ended.

**D6. Once a day, by what `update.json` remembers.** `at` is when the window asked last, kept before the
request, and `newer` what the last answer named. The file sits beside `settings.json` and is not part of
it: the window writes it without the user doing anything, and a write of the whole settings at such a
moment could undo what another window just saved. A start within the day asks nobody and offers the remembered version again,
if it is still newer than the running one.

**D7. A copy that cannot replace itself gets a link.** `place()` names what an update replaces: the `.app`
on macOS, unless macOS runs it from a translocated place, the user cannot write its folder, or its path
holds a quote or a backslash; the AppImage on
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
- The Windows install is Mortise's own few lines and ran on no machine yet.

## Decisions after the first review

**D9. The version is trusted through the signature.** The manifest is not signed. minisign signs a comment
with the bytes, and `cargo packager signer sign` puts the file name in it. The window refuses a download
whose signed name is not `Mortise-v<offered version>-<this platform>`, so a manifest cannot pass an older
signed file as a newer version, or the file of another platform.

**D10. Mortise asks for the manifest itself.** The check of the crate sets two environment variables on
Linux from a worker thread, which Rust calls unsafe once other threads run. `ureq` is in the workspace
already and needs no such thing.

**D11. No administrator on macOS.** The crate falls back to an AppleScript that runs a shell command as
administrator, with the paths written into it unescaped. A folder that needs an administrator therefore
gets the download page.

**D12. Mortise starts the Windows setup itself.** The crate starts it through a hidden PowerShell and ends
the process at once. Mortise writes the verified setup to a temporary file, starts it with `/P /R`, and
closes the usual way, so a setup that cannot start is an error the user reads.
