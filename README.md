# Mortise

A desktop companion for developers who build Laravel apps with
[`nwidart/laravel-modules`](https://github.com/nWidart/laravel-modules).

Mortise opens the folder of an app, lists its modules, and shows what is in each one.
It reads files only: it needs no PHP and changes nothing in the app.

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
