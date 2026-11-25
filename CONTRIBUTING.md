# Contributing

`AGENTS.md` holds the rules of the project and the commands to run before a commit. They apply to
people too. This file says how the workflows build, sign and release Mortise.

## Workflows

| Workflow | Runs on | Does |
|---|---|---|
| `ci.yml` | Every pull request and every push to `main` | Commit subjects, formatting, clippy and tests on macOS, Linux and Windows, coverage on macOS, the licence list, `cargo audit`, a secret scan, OpenSpec, and, not required, the package's commands in a real app and the actuator client against a real app |
| `installers.yml` | A run by hand, a pull request that changes the packaging, and a release | Builds the DMG, the Windows setup, the AppImage and the .deb as artifacts. It never publishes |
| `release.yml` | A run by hand on `main` | Opens the release pull request |
| `tag-release.yml` | The merge of a release pull request | Builds the installers on the merge commit, signs the macOS app, tags and publishes the release |
| `codeql.yml` | A push to `main`, weekly, by hand | CodeQL for Rust and for the workflows |

Commit subjects follow Conventional Commits. CI checks them, because git-cliff builds `CHANGELOG.md` from them.

The image tests compare with images made on macOS, so only the macOS job runs them. When an image
differs, the job keeps the new image and the difference as the artifact `images-that-differ`.

## Package

cargo-packager builds the installers from `[package.metadata.packager]` in `crates/mortise-app/Cargo.toml`.
They hold the window, binary `mortise-desktop`, under the name Mortise. The command line tool, binary
`mortise` of `crates/mortise-cli`, is not in them.
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

## Release

1. Run the Release workflow from the Actions tab, on `main`. Pick the bump, or leave `auto` to let git-cliff pick it from the commits.
2. The workflow runs `tools/release.sh`. It adds the section of the new version to `CHANGELOG.md`, sets the version in `Cargo.toml` and `Cargo.lock`, and opens the pull request `chore(release): vX.Y.Z` from `release/vX.Y.Z`.
3. Review the `CHANGELOG.md` section. Edit it on the branch if needed: the release notes come from it.
4. Merge with Squash and merge, and keep the subject `chore(release): vX.Y.Z`. A merge commit works too. A rebase merge does not.
5. The push to `main` starts the Tag release workflow. It builds and tests the merge commit, builds the installers, signs and notarizes the macOS app, then tags the commit and publishes the GitHub release with the installers and their SHA-256 sums.

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
passes it six secrets. That call runs the `package-macos-signed` job, the only job that sets
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

To check a download, mount the DMG and run:

```
tools/verify-macos-app.sh /Volumes/Mortise/Mortise.app
```

The keychain guards a saved token with the signature of the app that saved it. A token saved by a
build signed ad hoc can make the signed app ask once for access. Choose Always Allow.
