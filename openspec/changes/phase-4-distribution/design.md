# Design

## Context

The release path is `release.yml` (opens the release pull request), then `tag-release.yml` on its merge.
`tag-release.yml` calls `installers.yml` with `sign: true`, which builds the DMG, the Windows setup and
the Linux packages as artifacts. Only the job `package-macos-signed` sets `environment: release` and gets
the six Apple secrets. The publish job then writes `Mortise-vX.Y.Z-SHA256SUMS.txt` over every file in
`dist/`, tags the commit and creates the release.

`mortise-cli` links `ring` (through `rustls`, for `mortise push`) and `blake3` (through the PHP parser).
Both compile C or assembly with the `cc` crate, so a build for another target needs a C compiler for
that target. The workspace sets `publish = false` for every crate.

## Goals / Non-Goals

**Goals:**

- One release path. The tool rides on the existing workflows: no tag, no release and no crate without
  the installers.
- The action computes every URL from the version and the runner. No API call except to find `latest`.
- A secret reaches only the step that needs it, as for the app today.

**Non-Goals:**

- A versionless copy of each archive (`releases/latest/download/...`). The action asks the API for the
  latest tag instead; a person opens the latest release page.
- Caching in the action. The archive is a few MB and downloads in seconds.
- Stapling. A bare Mach-O binary cannot hold a ticket.

## Decisions

### Native runners for all four targets

| Target | Runner |
|---|---|
| `x86_64-unknown-linux-musl` | `ubuntu-latest`, with `musl-tools` |
| `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm`, with `musl-tools` |
| `aarch64-apple-darwin` | `macos-latest` |
| `x86_64-pc-windows-msvc` | `windows-latest` |

The repository is public, so `ubuntu-24.04-arm` costs nothing. A native runner needs no C cross
compiler for `ring` and `blake3`: `musl-gcc` of the runner's own architecture compiles them. Each job
also runs the binary it built (`--version`, and `check fixtures/shop-api` must exit 1) before it packs
it, which a cross build could not do without an emulator.

Alternatives considered:

- `cross`: runs the build in Docker images that it publishes. Another image to trust in the release
  path, and its images lag behind a new Rust release (`rust-toolchain.toml` pins 1.99).
- `cargo-zigbuild`: one Linux runner for both targets, but adds Zig and cargo-zigbuild to the release
  path, and the arm64 binary could not run on the x86_64 runner without QEMU.

`cc` finds `musl-gcc` by itself only for x86_64. The jobs set `CC_<target>=musl-gcc` for both Linux
targets, so both use the same compiler. Checked: the arm64 build in a `rust:1.99-bookworm` container
on Apple silicon (native arm64), with `musl-tools` and this variable. The binary is static and runs.

Windows links the C runtime statically (`-C target-feature=+crt-static`, through
`CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS`, which reaches the target and not the build scripts),
so the binary needs no Visual C++ redistributable on the machine.

### Archive names: tag, system, architecture

`mortise-vX.Y.Z-<platform>.<ext>` with the platform words of the installers (`linux-x86_64`,
`linux-arm64`, `macos-arm64`, `windows-x64`). `.tar.gz` on Linux keeps the executable bit; `.zip` on
macOS is the file that was notarized; `.zip` on Windows opens without other tools. The archive holds the
binary, `LICENSE` and `licenses/THIRD-PARTY.md` (as `THIRD-PARTY.md`) at its top level, so the action
adds the extract folder to `PATH` as it is.

The archives go into `dist/` of the publish job, so the existing `sha256sum -- *` lists them in
`Mortise-vX.Y.Z-SHA256SUMS.txt`. One sums file per release, one name for the action to fetch.

The tool jobs set `CARGO_PROFILE_RELEASE_STRIP=symbols`: the arm64 Linux binary shrinks from 8.0 MB
to about 5.5 MB. The release profile of the app does not change.

`tools/check-cli.sh <target>` runs the binary: the version of `Cargo.toml`, exit 1 on
`fixtures/shop-api`, statically linked on Linux. `tools/pack-cli.sh <target> <archive name>` then builds
an archive from `target/<target>/release` without running anything from the build. It refuses a name
with `/` or `\`, or without `.tar.gz` or `.zip`, before it writes or removes anything. The pull request
run, the run by hand and the release run all call both.

### Signing the macOS binary: own script, own job

cargo-packager signs the app, not a bare binary, so `tools/sign-macos-cli.sh` does it:

1. Import `APPLE_CERTIFICATE` into a temporary keychain with a random password, and put it first in
   the search list.
2. `codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY"` on the binary.
3. Pack the zip with `tools/pack-cli.sh`.
4. `xcrun notarytool submit <zip> --wait --output-format json` with the API key, and fail unless the
   status is `Accepted`. On a refusal, print `notarytool log` for the submission.
5. Check: `codesign --verify --strict`, the Developer ID authority, the runtime flag and the timestamp,
   on the binary in the zip as uploaded.
6. Delete the keychain and the key file on exit, also on SIGTERM.

The script never runs the built binary, so no code of the build runs while the secrets are in the
environment. `tools/check-cli.sh` runs the binary in the step before, and the signed binary in the step
after, both without secrets.

The job pair copies the app's: `package-cli-macos` without secrets when `sign` is false,
`package-cli-macos-signed` with `environment: release` when it is true. The build step runs before the
step that holds the secrets. No `spctl` check: for a bare binary its answer depends on Apple's online
ticket service right after notarization, which can lag, and a false failure would stop a release.

### Pull request run

`installers.yml` already runs on a pull request that changes the packaging. Its path filter gains the
manifest of `mortise-cli` and the two scripts. A new job `cli-sums` downloads the four tool artifacts,
writes their sums and fails unless exactly the four expected names are there. A run by hand of the
Installers workflow does the same. Nothing is uploaded to a release.

### crates.io

- `[workspace.package]` gains `repository` and `homepage`. `mortise-core` and `mortise-cli` set
  `publish = true`, `readme` (a short README in each crate: the root README has images with relative
  paths), `keywords`, `categories` and `license-file = "../../LICENSE"`, which cargo copies into the
  package. `mortise-app` keeps `publish = false`. `include` limits each package to `src`, the readme
  and the licence, so fixtures and test data stay out.
- The workspace dependency becomes `mortise-core = { path = "crates/mortise-core", version = "=X.Y.Z" }`.
  An exact version: the core has no stable API, and the tool of a version is tested with the core of
  that version only. `tools/release.sh` moves this version with the workspace version; if it did not,
  `cargo update` would fail on the release branch.
- The recorded actuator answers that `testing.rs` reads with `include_str!` live in
  `crates/mortise-core/fixtures/actuator`, inside the crate, and `include` packs them, so the published
  crate builds with `--features testing`. CI checks that from the unpacked package, outside the workspace.
- `mortise-cli` uses the workspace dependency. The self dev-dependency of the core has no version, so
  `cargo package` drops it.
- Cargo 1.99 packages and publishes several crates of a workspace in one command, in dependency
  order, with the unpublished core overlaid for the check of the tool. CI runs
  `cargo package --locked -p mortise-core -p mortise-cli` in the Linux job of the command line tool.
- The release job `crates` runs after `publish`, in the `release` environment. It reads the sparse
  index (`https://index.crates.io/mo/rt/<name>`) to skip a version already there, runs `cargo package`
  without the token (this step compiles, so build scripts run), then `cargo publish --no-verify` with
  `CARGO_REGISTRY_TOKEN` in that step only. A failure there leaves the GitHub release in place; the job
  can be re-run.

Alternatives considered: publishing before the GitHub release. A crate cannot be removed from
crates.io, while a failed GitHub release can be re-run, so the irreversible step goes last.

### The action

A composite action with one bash script, run by `shell: bash` on all three systems (Git Bash on
Windows). Inputs `version` (default `latest`) and `token` (default `${{ github.token }}`, used only for
the API call that resolves `latest`). Output `version`.

- Platform from `RUNNER_OS` and `RUNNER_ARCH`. macOS x64 fails with a pointer to `cargo install mortise-cli`.
- `curl --fail --location --retry 3` for the archive and the sums file from
  `https://github.com/nWidart/mortise/releases/download/<tag>/`.
- SHA-256 with `sha256sum`, or `shasum -a 256` on macOS. The line is matched on the exact name in the
  second field.
- Extract to `$RUNNER_TEMP/setup-mortise/<tag>` with `tar` or `unzip` (`7z` on Windows), add it to
  `GITHUB_PATH` (through `cygpath -w` on Windows), then run `mortise --version`.
- The base URL can be replaced by the environment variable `SETUP_MORTISE_BASE_URL` for the action's own
  tests, which serve a fixture release from `file://`. It is not an input and not documented for users.

## Risks / Trade-offs

- [Signing and notarization run only in a release] → The script follows the app's job line by line
  where it can, checks every secret first, and the first release is watched. A failure leaves no tag.
- [`ubuntu-24.04-arm` is a newer runner image] → A failure there fails the release before the tag, like
  any installer. The pull request run exercises it on each packaging change.
- [crates.io publish fails after the GitHub release] → The job skips what is already published and can
  be re-run. The pull request check packages both crates, so metadata errors show before a release.
- [A dependency change breaks the musl build] → The path filter of the pull request run lists the
  manifests and the scripts, not `Cargo.lock`, so a dependency update does not build the archives.
  The release run catches it before the tag; a run by hand of the Installers workflow checks a branch.
- [The action asks the API for `latest` with the run's token] → A fork's pull request gets a read-only
  token, which is enough for a public repository. Without a token the call is anonymous and limited to
  60 an hour per address.

## Migration Plan

Nothing to migrate. The next release after the merge carries the archives and publishes the crates.
The action's `v1` tag waits for that release; until then its README says so.
