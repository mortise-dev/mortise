# Mortise

A desktop companion for developers who build Laravel apps with
[`nwidart/laravel-modules`](https://github.com/nWidart/laravel-modules).

Mortise opens the folder of an app, lists its modules, and shows what is in each one.
To read an app, Mortise uses no PHP and changes nothing in it. It runs a package command in the
app only when you start it, with the exact command on screen first, through the PHP you chose
for the app: Herd, Sail, Docker or a command of your own. Sail is off on Windows, where it runs
in WSL only; a Custom command is the way there.

When the app also has [`nwidart/laravel-actuator`](https://github.com/nWidart/laravel-actuator)
and is running, Mortise can ask it for its runtime status, its routes and its scheduled tasks.
This is optional. Mortise asks for `health`, `info`, `mappings` and `scheduledtasks`, and never
for `env`. A token or a password goes to the credential store of the operating system and is
never written to a file.

Status: phase 1 (read only) and phase 2 (running the package's commands) are built. The plans are
`docs/plan/phase-1.md` and `docs/plan/phase-2.md`. Where the build differs from the specification is in
`docs/deviations.md`.

## What it does

- Lists the modules of an app with their state, files, lines, kinds, tests and coverage.
- Shows which module uses which, from the PHP source, and finds circular use.
- Checks two rules, kept in `mortise.json` in the app folder so that a team shares them.
- Lists findings: what breaks a rule or a check, each where it lives.
- Scans a module and proposes its public surface. The scan writes nothing.
- With `nwidart/laravel-actuator`: runtime status, routes by module, scheduled tasks.
- Runs the package's commands, each one only when you start it:
  - The runner drawer shows the output as written, with Stop and a time limit. Mortise keeps
    the last 50 runs of each app in its own data folder, not in the app.
  - The Actions tab turns a module on or off, migrates, rolls back, seeds and checks
    translations. Reset, refresh, drop all tables and delete module sit in a closed group,
    and each asks first. When the app reports the environment `production`, they do not run:
    Mortise only copies the command.
  - The Generate tab offers the 42 generators of `nwidart/laravel-modules` 13.3, New module
    among them. It shows the command, and after the run the files the command wrote. Replace
    runs a generator again over a file or a module that exists, after a question.
  - The palette lists actions and generators, and your recent runs.
- Closing Mortise stops every run and waits up to 5 seconds for each to end. On Windows, the
  PHP child of a launcher such as `herd.bat` that already exited goes on (`docs/deviations.md`, S53).

Not yet: the file lists before a run, the migration state (counts, tables) and Run tests wait
for data from the package. Until then the Actions and Generate tabs say so where they leave
something out.

The only file Mortise writes in an app is `mortise.json`, and it asks before it creates it.
The package's commands write the other files. Mortise never writes them.

## Install

Each [release](https://github.com/nWidart/mortise/releases/latest) has:

| Platform | File |
|---|---|
| macOS, Apple silicon | `Mortise-macos-arm64.dmg`. Signed with a Developer ID and notarized by Apple. |
| Windows, x64 | `Mortise-windows-x64-setup.exe`. Not signed: SmartScreen warns once. Choose More info, then Run anyway. |
| Linux, x86_64 | `Mortise-linux-x86_64.AppImage`, or `Mortise-linux-amd64.deb` for Debian and Ubuntu. Not signed. |

`Mortise-vX.Y.Z-SHA256SUMS.txt` holds the SHA-256 of each file.

## Run

```
cargo run -p mortise-app -- /path/to/laravel-app
```

Rust 1.99 is needed; `rust-toolchain.toml` selects it.

Mortise keeps its settings in `settings.json` in the config folder of the platform.
`MORTISE_CONFIG_DIR` names another folder. The history of runs lives in the data folder;
`MORTISE_DATA_DIR` names another one. `MORTISE_SECRETS=memory` keeps secrets out of
the credential store, for work on Mortise itself.

## Test

```
cargo test --workspace
```

| Tests | Check |
|---|---|
| `crates/mortise-core/tests` | Reading an app, git, the file watcher, the actuator client against a local stub, the public surface scan |
| `crates/mortise-app/tests` | The interface, run headless and asked by role and name, as a screen reader would |
| `crates/mortise-app/tests/images.rs` | The screens in both themes against the images in `tests/snapshots`. macOS only |
| `crates/mortise-app/tests/accessibility.rs` | Every control has a name and a target of 24 by 24 px |
| `crates/mortise-core/tests/real_php.rs` | The package's commands in a real Laravel app. Ignored by default; a CI job that is not required runs them (see `AGENTS.md`) |

After a change that is meant to be seen, make the images again and look at what changed:

```
UPDATE_SNAPSHOTS=1 cargo test -p mortise-app --test images
```

`cargo test -p mortise-app --test screens -- --ignored` writes every screen to
`target/screens`, to be looked at beside the drawings in `docs/spec`.

## Layout

| Path | Holds |
|---|---|
| `crates/mortise-core` | Everything that is not drawing: reading an app into a snapshot, the actuator client, the scan. No interface dependency. |
| `crates/mortise-app` | The window, built with egui and eframe. Binary `mortise`. |
| `assets/fonts` | Asap and Inconsolata, each with its `OFL.txt`. |
| `assets/icon` | The app icon: two drawings, `mortise.svg` and `mortise-16.svg`, and the files made from them for macOS (`.icns`), Windows (`.ico`) and Linux (hicolor PNGs). |
| `fixtures` | Laravel apps for the tests. They are never installed or run. |
| `docs/spec` | The design specification of phase 0. |
| `docs/deviations.md` | Where the build differs from the specification, and why. |
| `licenses` | The licences of the crates Mortise is built from. |
| `tools` | `third-party-licenses.py` writes that list. `app-icon.mjs` makes the icon files from the drawings; it needs macOS and Google Chrome. `release.sh`, `pack-installer.sh` and `verify-macos-app.sh` serve the release workflows. |
| `.github/workflows` | CI, the installers and the release. `CONTRIBUTING.md` says how a release works. |

## Licence

MIT, see `LICENSE`. The fonts in `assets/fonts` are under the SIL Open Font License 1.1;
each folder holds its `OFL.txt`. The crates Mortise is built from are listed with their
licences in `licenses/THIRD-PARTY.md`.
