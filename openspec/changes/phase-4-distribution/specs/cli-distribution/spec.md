# cli-distribution Specification

## Purpose
Defines how a team gets the command line tool `mortise`: the archives on each GitHub release, their
sums and signatures, the crates on crates.io, and the action `nwidart/setup-mortise` that installs it in CI.

## ADDED Requirements

### Requirement: Each release carries the tool for four platforms
Each GitHub release of Mortise SHALL carry one archive of the command line tool for each of these
platforms, named from the release tag `vX.Y.Z`:

| Platform | Archive |
|---|---|
| Linux x86_64, static (musl) | `mortise-vX.Y.Z-linux-x86_64.tar.gz` |
| Linux arm64, static (musl) | `mortise-vX.Y.Z-linux-arm64.tar.gz` |
| macOS arm64 | `mortise-vX.Y.Z-macos-arm64.zip` |
| Windows x64 | `mortise-vX.Y.Z-windows-x64.zip` |

#### Scenario: Names from the tag
- **WHEN** the release `v0.3.0` is published
- **THEN** it has the four archives named with `v0.3.0` above, and each opens to a binary that prints `mortise 0.3.0` for `--version`

#### Scenario: A static Linux binary
- **WHEN** the Linux archives are built
- **THEN** each binary is statically linked and runs on a Linux of its architecture with no C library installed

#### Scenario: The binary is checked before it is packed
- **WHEN** an archive is built
- **THEN** its binary first runs on a machine of its platform, prints its version and exits 1 on `mortise check fixtures/shop-api`

### Requirement: An archive holds the binary and its licences at the top level
An archive SHALL hold, at its top level and in no folder, the binary (`mortise`, or `mortise.exe` on
Windows), `LICENSE` and `THIRD-PARTY.md`.

#### Scenario: Unpack an archive
- **WHEN** `mortise-v0.3.0-linux-x86_64.tar.gz` is unpacked into an empty folder
- **THEN** the folder holds `mortise`, `LICENSE` and `THIRD-PARTY.md` and nothing else

### Requirement: The archives ship only with the installers
The release SHALL publish the archives only together with the installers, from the same workflow run.

#### Scenario: An archive fails to build
- **WHEN** one of the four archives fails to build or to pass its checks in a release run
- **THEN** nothing is tagged and no release is published

### Requirement: The sums file lists every archive
The release's `Mortise-vX.Y.Z-SHA256SUMS.txt` SHALL hold one line per archive in the format of
`sha256sum`: the lower-case hexadecimal SHA-256, two spaces, the file name.

#### Scenario: Check a download
- **WHEN** a user runs `sha256sum -c --ignore-missing Mortise-v0.3.0-SHA256SUMS.txt` beside a downloaded archive
- **THEN** the line of the archive ends in `OK`

### Requirement: The macOS binary is signed and notarized in a release
In a release the macOS binary SHALL be signed with the Developer ID Application certificate of the app,
with the hardened runtime and a secure timestamp, and its zip SHALL be accepted by Apple's notary
service before it is uploaded. A missing Apple secret SHALL fail the release. A build that is not a
release SHALL keep the ad hoc signature of the linker and SHALL get no secret.

#### Scenario: A release
- **WHEN** the Tag release workflow builds the macOS archive
- **THEN** `codesign` reports the Developer ID authority, the runtime flag and a timestamp, and notarytool reports the zip `Accepted`

#### Scenario: A pull request
- **WHEN** a pull request builds the archives
- **THEN** no job of that run has the Apple secrets, and the macOS archive is signed ad hoc

### Requirement: The archives build without a release
A pull request that changes the packaging of the tool, and a run by hand of the Installers workflow,
SHALL build the four archives and a sums file as workflow artifacts. They SHALL not tag, publish or
upload to a release.

#### Scenario: A pull request on the packaging
- **WHEN** a pull request changes `tools/pack-cli.sh`
- **THEN** the run has the artifacts of the four archives and a sums file that lists exactly those four names

### Requirement: The crates are publishable
`mortise-core` and `mortise-cli` SHALL be publishable on crates.io, with a description, the MIT
licence, the repository, the documentation site, a readme, keywords and categories. `mortise-cli` SHALL
depend on exactly the `mortise-core` of its own version. `mortise-app` SHALL not be publishable.

#### Scenario: Every pull request packages both crates
- **WHEN** CI runs on a pull request that changes code
- **THEN** `cargo package` of both crates succeeds, so a release cannot fail on the metadata

### Requirement: A release publishes the crates
After a GitHub release, the Tag release workflow SHALL publish `mortise-core`, then `mortise-cli`, and
SHALL skip a crate whose version crates.io already has. The token SHALL come from the secret
`CARGO_REGISTRY_TOKEN` of the `release` environment and SHALL reach only the step that uploads.

#### Scenario: Install from crates.io
- **WHEN** a user runs `cargo install mortise-cli` after the release `v0.3.0`
- **THEN** cargo builds `mortise-cli` 0.3.0 with `mortise-core` 0.3.0 and installs the binary `mortise`

#### Scenario: A version already published
- **WHEN** the crates job runs again for a release whose `mortise-core` is already on crates.io
- **THEN** it publishes `mortise-cli` only, and it succeeds when both are already there

### Requirement: The action picks the release and the archive
The action `nwidart/setup-mortise` SHALL take an input `version` (`latest` by default, or a tag
`vX.Y.Z` or `X.Y.Z`) and an input `token` for the GitHub API (the token of the run by default). It SHALL
compute the archive name for the runner (Linux x64 or arm64, macOS arm64, Windows x64). It SHALL need
no Rust toolchain and no Node build step.

#### Scenario: An unsupported runner
- **WHEN** the action runs on macOS on Intel
- **THEN** it fails with a message that names the runner and `cargo install --locked mortise-cli`

### Requirement: The action checks the archive before it installs it
The action SHALL download the archive and the sums file of the release from `github.com/nWidart/mortise`
and compare the SHA-256 of the archive with its line in the sums file. It SHALL fail when the line is
missing or the sums differ. Only then SHALL it extract the binary and add its folder to `PATH`.

#### Scenario: The onboarding YAML
- **WHEN** a workflow on `ubuntu-latest` runs `uses: nwidart/setup-mortise@v1`, then `mortise check . --format github`
- **THEN** the step finds `mortise` on `PATH` and the job needs no Rust toolchain

#### Scenario: A changed archive
- **WHEN** the SHA-256 of the downloaded archive differs from its line in the sums file
- **THEN** the action fails, names the archive, and puts nothing on `PATH`
