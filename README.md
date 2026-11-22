# Mortise

A desktop companion for developers who build Laravel apps with
[`nwidart/laravel-modules`](https://github.com/nWidart/laravel-modules).

Mortise opens the folder of an app, lists its modules, and shows what is in each one.
It reads files only: it needs no PHP and changes nothing in the app.

When the app also has [`nwidart/laravel-actuator`](https://github.com/nWidart/laravel-actuator)
and is running, Mortise can ask it for its runtime status, its routes and its scheduled tasks.
This is optional. Mortise asks for `health`, `info`, `mappings` and `scheduledtasks`, and never
for `env`. A token or a password goes to the credential store of the operating system and is
never written to a file.

Status: phase 1 (read only) is built. The plan is in `docs/plan/phase-1.md`. Where the
build differs from the specification is in `docs/deviations.md`.

## What it does

- Lists the modules of an app with their state, files, lines, kinds, tests and coverage.
- Shows which module uses which, from the PHP source, and finds circular use.
- Checks two rules, kept in `mortise.json` in the app folder so that a team shares them.
- Lists findings: what breaks a rule or a check, each where it lives.
- Scans a module and proposes its public surface. The scan writes nothing.
- With `nwidart/laravel-actuator`: runtime status, routes by module, scheduled tasks.

The only file Mortise writes in an app is `mortise.json`, and it asks before it creates it.

## Run

```
cargo run -p mortise-app -- /path/to/laravel-app
```

Rust 1.99 is needed; `rust-toolchain.toml` selects it.

Mortise keeps its settings in `settings.json` in the config folder of the platform.
`MORTISE_CONFIG_DIR` names another folder. `MORTISE_SECRETS=memory` keeps secrets out of
the credential store, for work on Mortise itself.

## Test

```
cargo test --workspace
```

| Tests | Check |
|---|---|
| `crates/mortise-core/tests` | Reading an app, git, the file watcher, the actuator client against a local stub, the public surface scan |
| `crates/mortise-app/tests` | The interface, run headless and asked by role and name, as a screen reader would |
| `crates/mortise-app/tests/images.rs` | Eighteen screens in both themes against the images in `tests/snapshots`. macOS only |
| `crates/mortise-app/tests/accessibility.rs` | Every control has a name and a target of 24 by 24 px |

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
| `fixtures` | Laravel apps for the tests. They are never installed or run. |
| `docs/spec` | The design specification of phase 0. |
| `docs/deviations.md` | Where the build differs from the specification, and why. |
| `licenses` | The licences of the crates Mortise is built from. |
| `tools` | The script that writes that list. |

## Licence

MIT, see `LICENSE`. The fonts in `assets/fonts` are under the SIL Open Font License 1.1;
each folder holds its `OFL.txt`. The crates Mortise is built from are listed with their
licences in `licenses/THIRD-PARTY.md`.
