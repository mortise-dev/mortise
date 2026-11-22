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

Status: phase 1 (read only) is being built. See `docs/plan/phase-1.md`.

## Run

```
cargo run -p mortise-app -- /path/to/laravel-app
```

Rust 1.99 is needed; `rust-toolchain.toml` selects it.

## Layout

| Path | Holds |
|---|---|
| `crates/mortise-core` | Everything that is not drawing: reading an app into a snapshot. No interface dependency. |
| `crates/mortise-app` | The window, built with egui and eframe. Binary `mortise`. |
| `assets/fonts` | Asap and Inconsolata, each with its `OFL.txt`. |
| `fixtures` | Laravel apps for the tests. They are never installed or run. |
| `docs/spec` | The design specification of phase 0. |
| `docs/deviations.md` | Where egui forces a difference from the specification. |

## Licence

MIT, see `LICENSE`. The fonts in `assets/fonts` are under the SIL Open Font License 1.1;
each folder holds its `OFL.txt`.
