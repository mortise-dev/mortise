# Contributing

`AGENTS.md` holds the rules of the project and the commands to run before a commit. They apply to
people too. This file says how the workflows build, sign and release Mortise.

## Workflows

| Workflow | Runs on | Does |
|---|---|---|
| `ci.yml` | Every pull request and every push to `main` | Commit subjects, formatting, clippy and tests on macOS, Linux and Windows, coverage on macOS, the licence list, `cargo audit`, a secret scan, OpenSpec, and, not required, the package's commands in a real app and the actuator client against a real app |
| `installers.yml` | A run by hand, a pull request that changes the packaging, and a release | Builds the DMG, the Windows setup, the AppImage and the .deb, and the four archives of the command line tool with their sums, as artifacts. It never publishes |
| `release.yml` | A run by hand on `main` | Opens the release pull request |
| `tag-release.yml` | The merge of a release pull request | Builds the installers and the command line tool on the merge commit, signs the macOS app and tool and the updates, tags and publishes the release, then publishes the crates to crates.io |
| `codeql.yml` | A push to `main`, weekly, by hand | CodeQL for Rust and for the workflows |

Commit subjects follow Conventional Commits. CI checks them, because git-cliff builds `CHANGELOG.md` from them.

The image tests compare with images made on macOS, so only the macOS job runs them. When an image
differs, the job keeps the new image and the difference as the artifact `images-that-differ`.

## Issues

A new issue offers two forms: a problem and a request (`.github/ISSUE_TEMPLATE/`). Blank issues are off.
The window has a "Report a problem" button in Settings, General. It opens the problem form with the
Mortise version and the platform filled in, and nothing of the app.

Check each request against the line first (`docs/plan/phase-4.md`, section 1): Mortise covers one app,
at one commit, on one machine. When a request is not clearly on one side, the owner decides before
anything is built.

- One app, one commit, one machine: label `free`. It may go in the feedback slot of the plan, one pull
  request for each change.
- Across time, across commits, across apps or across people: label `cloud`. Answer with the reason, then
  tell the cloud thread. Mortise Cloud sells that, so the free Mortise does not build it.

## Package

cargo-packager builds the installers from `[package.metadata.packager]` in `crates/mortise-app/Cargo.toml`.
They hold the window, binary `mortise-desktop`, under the name Mortise. The command line tool, binary
`mortise` of `crates/mortise-cli`, is not in them: it ships as archives of its own (see below).
To build the macOS app and DMG on a Mac:

```
cargo install cargo-packager --locked --version 0.11.8
cargo packager -p mortise-app --release --formats app,dmg
```

The files land in `target/packages`. A local build is signed ad hoc (`signing-identity = "-"`).

| Platform | Formats | Signed |
|---|---|---|
| macOS, Apple silicon | `app`, `dmg` | Developer ID and notarized, in a release only |
| Windows, x64 | `nsis` | No |
| Linux, x86_64 | `appimage`, `deb`, built on Ubuntu 22.04 | No |

## Command line tool

The Installers workflow builds the tool on four runners, each for its own architecture, so `ring` and
`blake3` compile their C with the runner's own compiler and each binary runs before it is packed.

| Platform | Runner | Target | Archive | Signed |
|---|---|---|---|---|
| Linux, x86_64 | `ubuntu-latest` | `x86_64-unknown-linux-musl`, static | `mortise-vX.Y.Z-linux-x86_64.tar.gz` | No |
| Linux, arm64 | `ubuntu-24.04-arm` | `aarch64-unknown-linux-musl`, static | `mortise-vX.Y.Z-linux-arm64.tar.gz` | No |
| macOS, Apple silicon | `macos-latest` | `aarch64-apple-darwin` | `mortise-vX.Y.Z-macos-arm64.zip` | Developer ID and notarized, in a release only |
| Windows, x64 | `windows-latest` | `x86_64-pc-windows-msvc`, static C runtime | `mortise-vX.Y.Z-windows-x64.zip` | No |

Each archive holds `mortise` (or `mortise.exe`), `LICENSE` and `THIRD-PARTY.md` at its top level.
The action [`mortise-dev/setup-mortise`](https://github.com/mortise-dev/setup-mortise) computes these names
from the version, the system and the architecture: do not rename them without changing the action.

`tools/check-cli.sh` runs a binary: it must print the version of `Cargo.toml`, exit 1 on the fixture
`shop-api`, and on Linux be statically linked. `tools/pack-cli.sh` then packs it without running it. It
takes a file name in `dist/` only, ending in `.tar.gz` or `.zip`. The job `cli-sums` fails unless exactly
the four names above are there. To build one archive on your machine:

```
cargo build --release --locked -p mortise-cli --target aarch64-apple-darwin
tools/check-cli.sh aarch64-apple-darwin
tools/pack-cli.sh aarch64-apple-darwin mortise-v0.3.0-macos-arm64.zip
```

A pull request that changes `crates/mortise-cli/Cargo.toml` or one of the three scripts builds the four
archives and their sums as artifacts. A dependency update does not; run the Installers workflow by hand
on the branch to check one.

## crates.io

`mortise-core` and `mortise-cli` are published to crates.io, so `cargo install mortise-cli` works.
`mortise-app` is not. `mortise-cli` depends on exactly the `mortise-core` of its own version: the
workspace dependency in `Cargo.toml` holds `version = "=X.Y.Z"`, and `tools/release.sh` moves it with
the version. CI packages both crates on every pull request (`cargo package -p mortise-core -p mortise-cli`).

After the GitHub release, the job `crates` of the Tag release workflow publishes `mortise-core`, then
`mortise-cli`. It skips a version that crates.io already has, so a re-run is safe. It uses the secret
`CARGO_REGISTRY_TOKEN`, an environment secret of `release`, which reaches only the upload step: the step
before it builds both packages without the token, and the upload runs with `--no-verify`.

```
gh secret set CARGO_REGISTRY_TOKEN --env release
```

The token is an API token of crates.io with the scope `publish-update`, limited to the crates
`mortise-core` and `mortise-cli`. For the first publish of a crate it needs `publish-new` too.
A crate version cannot be removed from crates.io, only yanked.

## Release

1. Run the Release workflow from the Actions tab, on `main`. Pick the bump, or leave `auto` to let git-cliff pick it from the commits.
2. The workflow runs `tools/release.sh`. It adds the section of the new version to `CHANGELOG.md`, sets the version in `Cargo.toml` and `Cargo.lock`, and opens the pull request `chore(release): vX.Y.Z` from `release/vX.Y.Z`.
3. Review the `CHANGELOG.md` section. Edit it on the branch if needed: the release notes come from it.
4. Merge with Squash and merge, and keep the subject `chore(release): vX.Y.Z`. A merge commit works too. A rebase merge does not.
5. The push to `main` starts the Tag release workflow. It builds and tests the merge commit, builds the installers and the archives of the command line tool, signs and notarizes the macOS app and the macOS tool, signs what a window updates from (see Update key), then tags the commit and publishes the GitHub release with the installers, the update manifest, the archives and their SHA-256 sums.
6. The same workflow then publishes `mortise-core` and `mortise-cli` to crates.io. If that job fails, the GitHub release stays; fix the cause, then re-run the failed job.

The tag exists only once every installer built. If the Tag release workflow did not start, or the
build failed for a reason the merge commit does not hold, run Tag release by hand with the tag.

The Release workflow needs the repository secret `RELEASE_TOKEN`: a fine-grained personal access token
with Contents and Pull requests read and write on this repository. A pull request opened with the
token of the workflow starts no CI, so the workflow opens it with this token.

```
gh secret set RELEASE_TOKEN
```

## macOS signing

Only a release signs. The Tag release workflow calls the Installers workflow with `sign: true` and
passes it six secrets. That call runs the jobs `package-macos-signed` (the app) and
`package-cli-macos-signed` (the command line tool), the only jobs of the Installers workflow that set
`environment: release`. The six secrets are environment secrets of `release`, which accepts runs from
`main` only. A branch, a pull request, a fork and a run by hand of the Installers workflow get no
secret, and build an app signed ad hoc. When a release runs and a secret is empty or missing, the build
fails with an error that names it, and nothing is tagged. A release never ships signed ad hoc.

Do not add a `refs/pull/*/merge` rule to the environment: it would let any pull request reach the secrets.

| Secret | What it is |
|---|---|
| `APPLE_CERTIFICATE` | The Developer ID Application certificate with its private key, as a `.p12` file in base64 |
| `APPLE_CERTIFICATE_PASSWORD` | The password set when the `.p12` was exported |
| `APPLE_SIGNING_IDENTITY` | The certificate name, such as `Developer ID Application: Your Name (TEAMID)` |
| `APPLE_API_ISSUER` | The issuer ID of an App Store Connect API key |
| `APPLE_API_KEY` | The key ID of that API key |
| `APPLE_API_KEY_P8` | The contents of the `AuthKey_<key id>.p8` file of that key |

To set them up:

1. The `release` environment exists, with one deployment branch rule: `main`. It is under Settings, Environments.
2. Create the certificate. In Keychain Access, choose Certificate Assistant, then Request a Certificate From a Certificate Authority, and save the request to disk. At [developer.apple.com](https://developer.apple.com/account/resources/certificates/add), choose Developer ID Application, upload the request, download the `.cer` and open it to install it. Only the Account Holder can create this certificate. A certificate made for another app works: it names the team, not the app.
3. Export it. In Keychain Access, under My Certificates, select the certificate with its private key under it. Choose File, then Export Items, save a `.p12` and set a password. `security find-identity -v -p codesigning` prints the name for `APPLE_SIGNING_IDENTITY`.
4. Create the API key. In App Store Connect, open Users and Access, then Integrations, then Team Keys, and generate a key with the Developer role. The page shows the issuer ID and the key ID. The `.p8` file downloads once. A key made for another app works too.
5. Add the six secrets as environment secrets of `release`, not as repository secrets:

   ```bash
   base64 -i cert.p12 | gh secret set APPLE_CERTIFICATE --env release
   gh secret set APPLE_CERTIFICATE_PASSWORD --env release
   gh secret set APPLE_SIGNING_IDENTITY --env release --body 'Developer ID Application: Your Name (TEAMID)'
   gh secret set APPLE_API_ISSUER --env release --body '<issuer id>'
   gh secret set APPLE_API_KEY --env release --body '<key id>'
   gh secret set APPLE_API_KEY_P8 --env release < AuthKey_XXXXXXXXXX.p8
   ```

   Use `--body` for the one-line values, so none ends in a newline. Delete the `.p12` and the `.p8` from your disk afterwards, or keep them in a password manager.

What the build does:

- cargo-packager 0.11.8 reads no signing identity from the environment, so the packaging step writes `APPLE_SIGNING_IDENTITY` over the `signing-identity` line of `crates/mortise-app/Cargo.toml` in its own checkout. The `-` stays in the repository for local builds.
- cargo-packager imports `APPLE_CERTIFICATE` into a temporary keychain, signs the binary and then the app with the hardened runtime and a secure timestamp, and deletes the keychain. It then notarizes the app with `notarytool` and staples the ticket to the app before it builds the DMG. The DMG is signed but not notarized.
- The app is built in a step before the one that holds the secrets, and that step replaces the `before-packaging-command` with `true`, so cargo-packager builds nothing while it holds them. Both steps share one machine, so a build script of a dependency could still reach the secrets. Review each change to `Cargo.lock` as you would code that signs releases.
- Before the job uploads anything, `tools/verify-macos-app.sh` must pass on the app inside the DMG, mounted read-only, and on the built app: `codesign --verify --deep --strict`, the Developer ID authority, the hardened runtime, the timestamp, `stapler validate`, and `spctl --assess` that sees it as notarized.
- The app has no entitlements file. It uses Metal, the keychain through the Security framework and the network, and none of them needs an exception from the hardened runtime. This follows from reading, not from a test of a signed build: open the first signed release on a Mac and check that it starts, opens an app folder and saves a token.

The command line tool is a bare binary, which cargo-packager does not sign, so `tools/sign-macos-cli.sh` does:

- It imports `APPLE_CERTIFICATE` into a temporary keychain with a random password, signs the binary with
  the hardened runtime and a secure timestamp, packs the zip with `tools/pack-cli.sh`, and sends that zip
  to `notarytool`. It fails unless Apple answers `Accepted`, and prints Apple's log when it does not.
- A bare binary cannot hold a stapled ticket. Gatekeeper looks the ticket up online the first time a
  downloaded binary runs. The script checks the signature of the binary in the zip as uploaded; it does
  not run `spctl`, whose answer for a bare binary depends on that online lookup right after notarizing.
- The keychain, the key file and the search list of keychains are restored when the script ends.
- The script never runs the built binary, so no code of the build runs while the secrets are in the
  environment. `tools/check-cli.sh` runs it in the step before, and again on the signed binary in the
  step after, both without the secrets.
- The binary has no entitlements file. It reads files and opens an HTTPS connection, and neither needs an
  exception from the hardened runtime. This follows from reading, not from a signed build: run the first
  signed tool from a download on a Mac.

To check a download, mount the DMG and run:

```
tools/verify-macos-app.sh /Volumes/Mortise/Mortise.app
```

The keychain guards a saved token with the signature of the app that saved it. A token saved by a
build signed ad hoc can make the signed app ask once for access. Choose Always Allow.

## Update key

A window installs a newer version only when the download carries a signature of the update key
(`docs/deviations.md`, S80). The key is a minisign key pair that cargo-packager makes. It has nothing to do
with the Apple certificate.

- The public half is the constant `PUBLIC_KEY` in `crates/mortise-app/src/update/mod.rs`. Every build carries it.
- The private half and its password are the secrets `CARGO_PACKAGER_SIGN_PRIVATE_KEY` and
  `CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD` of the `release` environment. Only the step "Sign and write
  the manifest" of the job "Sign the updates" in `installers.yml` gets them. That job builds nothing of the app.

Set it up once:

1. Make the key pair, with a password:
   ```
   cargo packager signer generate --path ~/.mortise/update.key
   ```
2. Put the contents of `~/.mortise/update.key.pub` in `PUBLIC_KEY`, through a pull request.
3. Store the private half and the password:
   ```
   gh secret set CARGO_PACKAGER_SIGN_PRIVATE_KEY --env release < ~/.mortise/update.key
   gh secret set CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD --env release
   ```
4. Keep a copy of `~/.mortise/update.key` and its password somewhere safe that is not this computer, such as a
   password manager. GitHub never shows a secret again.

What a release does with it:

- The job "Build dmg (signed)" packs the notarized app as `Mortise-vX.Y.Z-macos-arm64.app.tar.gz` and checks
  that the app still passes `tools/verify-macos-app.sh` once it is unpacked.
- `tools/sign-updates.sh` signs that archive, the Windows setup and the AppImage, checks each signature with
  the public key of the source, and writes `latest.json`: the version, and per platform the address, the
  signature and the format. A missing key, or a key that is not the one in the source, fails the release
  before anything is published.
- `tag-release.yml` publishes `latest.json` with the release. A window reads it at
  `https://github.com/mortise-dev/mortise/releases/latest/download/latest.json`.

Take care of two things:

- A lost private key cannot be replaced for the windows that are installed: they trust the old public key
  only. Their users must install the next version by hand. So keep the copy of step 4.
- A release that is marked "pre-release" or "draft" is not the latest release, so no window sees it. A
  release that must not reach the windows yet can use that.
