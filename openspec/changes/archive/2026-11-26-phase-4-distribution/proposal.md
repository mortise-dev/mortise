# Proposal

## Why

A team that runs `mortise check` or `mortise push` in CI today builds the tool from source with
`cargo install --git`, which needs a Rust toolchain and several minutes. Mortise Cloud's onboarding
YAML runs `uses: nwidart/setup-mortise@v1` and expects the tool on `PATH` in seconds (cloud request
R7, `mortise-cloud/docs/spec/04-mortise-requests.md`). Milestone 26 of the phase 4 plan
(`docs/plan/phase-4.md`, question 7 and 8) builds that distribution.

## What Changes

- Every GitHub release also carries the command line tool for four platforms, built by the existing
  release path (`tag-release.yml` calls `installers.yml`): Linux x86_64 and arm64 (static, musl),
  macOS arm64 and Windows x64. Each is one archive with a name computed from the version, the system
  and the architecture. The existing `Mortise-vX.Y.Z-SHA256SUMS.txt` lists them with the installers.
- In a release the macOS binary is signed with the Developer ID of the app, with the hardened runtime
  and a secure timestamp, and notarized inside its zip. A bare binary cannot hold a stapled ticket, so
  Gatekeeper checks the ticket online on first run.
- A pull request that changes the packaging of the tool, and a run by hand of the Installers
  workflow, build the four archives and a sums file as workflow artifacts. Nothing is uploaded to a
  release.
- `mortise-core` and `mortise-cli` become publishable on crates.io with full metadata, so
  `cargo install mortise-cli` works. `mortise-app` stays unpublished. The Tag release workflow
  publishes both after the GitHub release, core first, and skips a version that crates.io already has.
  CI checks on every pull request that both crates package.
- The action `nwidart/setup-mortise`, in its own repository: a composite action that downloads the
  archive for the runner, checks its SHA-256 against the sums file and puts `mortise` on `PATH`.
- Docs: `CONTRIBUTING.md` (release steps), `README.md` (three ways to install the tool), the
  documentation site (installation, check the rules in CI, send a read to Mortise Cloud).

Out of scope, by section 1 of the plan: nothing on the cloud's side of the line. No installer change
for the window. No Homebrew, Scoop or apt repository. No macOS x86_64 or Windows arm64 binary.

## Capabilities

### New Capabilities

- `cli-distribution`: the release archives of the command line tool, their names, sums and signing,
  the crates on crates.io, and what the action relies on.

### Modified Capabilities

None. `mortise check` and `mortise push` do not change.

## Impact

- `.github/workflows/installers.yml`: jobs that build, check and archive the tool on four native
  runners, a signed variant for macOS, and a job that lists the archives with their sums.
- `.github/workflows/tag-release.yml`: the publish job adds the tool archives to the release; a new
  job publishes the crates, in the `release` environment, with `CARGO_REGISTRY_TOKEN`.
- `.github/workflows/ci.yml`: the Linux job of the command line tool packages both crates.
- `Cargo.toml`, `crates/mortise-core/Cargo.toml`, `crates/mortise-cli/Cargo.toml`,
  `crates/mortise-app/Cargo.toml`: publish settings and metadata; the core dependency gets an exact
  version. `tools/release.sh` moves that version with the release.
- New: `tools/check-cli.sh`, `tools/pack-cli.sh`, `tools/sign-macos-cli.sh`, `crates/mortise-core/README.md`,
  `crates/mortise-cli/README.md`.
- No new dependency. The Rust code changes only the paths of the recorded actuator answers, which move
  into `crates/mortise-core/fixtures/actuator`.
- A second repository, `nWidart/setup-mortise`: `action.yml`, its script, README, LICENSE and CI.
