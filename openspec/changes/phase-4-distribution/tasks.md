# Tasks

## 1. Crates

- [x] 1.1 Workspace metadata (`repository`, `homepage`), exact version on the `mortise-core` workspace dependency, `mortise-cli` uses it. `mortise-app` keeps `publish = false`. Verify with `cargo metadata` and `cargo build --workspace`
- [x] 1.2 `mortise-core` and `mortise-cli`: `publish = true`, readme, keywords, categories, `license-file`, `include`; a README in each crate. Verify with `cargo package --locked -p mortise-core -p mortise-cli --list` (no fixture, no test data) and `cargo publish --dry-run --locked -p mortise-core -p mortise-cli`
- [x] 1.3 `tools/release.sh` moves the exact core version with the workspace version. Verify by running its perl lines on a copy of `Cargo.toml` and `cargo metadata` on the result

## 2. Release archives

- [x] 2.1 `tools/pack-cli.sh <target> <archive>`: binary, `LICENSE`, `THIRD-PARTY.md`, flat, `.tar.gz` or `.zip`. Verify on macOS with `shellcheck` and a local run on `aarch64-apple-darwin`
- [x] 2.2 `tools/sign-macos-cli.sh`: temporary keychain, codesign with runtime and timestamp, notarize the zip, checks, cleanup. Verify with `shellcheck`; the signing itself runs in a release only
- [x] 2.3 `installers.yml`: jobs `package-cli` (Linux x86_64, Linux arm64, Windows), `package-cli-macos`, `package-cli-macos-signed`, `cli-sums`; path filter. Verify with `actionlint` and the pull request run
- [x] 2.4 `tag-release.yml`: download the tool artifacts into `dist/`; job `crates` in the `release` environment. Verify with `actionlint`
- [x] 2.5 `ci.yml`: the Linux job of the command line tool packages both crates. Verify with `actionlint` and the pull request run

## 2b. Review round 1

- [x] 2b.1 `tools/check-cli.sh` runs the binary; `tools/pack-cli.sh` and `tools/sign-macos-cli.sh` never do. The signed job checks before and after the step with the secrets. Verify with `shellcheck` and `actionlint`
- [x] 2b.2 `tools/pack-cli.sh` refuses a name with `/` or `\` or a wrong suffix before it removes anything. Verify with the CI step that runs it on `../LICENSE`
- [x] 2b.3 Move `fixtures/actuator` into `crates/mortise-core/fixtures/actuator` and pack it. Verify with `cargo check --no-default-features --features testing --lib` in the unpacked package

## 3. The action (repository `nWidart/setup-mortise`)

- [x] 3.1 `action.yml` and `setup.sh`: inputs, platform, download, SHA-256, extract, `PATH`. Verify with `shellcheck` and a local run against a fixture release served from `file://`
- [x] 3.2 README with the onboarding YAML, MIT `LICENSE`, CI on `ubuntu-latest`, `ubuntu-24.04-arm`, `macos-latest`, `windows-latest` against a fixture release, plus a job against a real release that waits for one. Verify with `actionlint` and the pull request run

## 4. Docs and checks

- [x] 4.1 `CONTRIBUTING.md`, `README.md`, the documentation site (installation, check the rules in CI, send a read to Mortise Cloud, command line reference). Verify with `cd website && npm ci && npm run build`
- [x] 4.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `openspec validate phase-4-distribution --strict`

## Workflow follow-up

- Pull requests to `main` of Mortise and of `nWidart/setup-mortise`. Neither is merged here.
- The first release after the merge: watch the signed macOS job and the crates job, then pin the
  action's CI to that release and tag the action `v1.0.0` and `v1`.
- Archive the change through its own pull request.
