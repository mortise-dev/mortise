# mortise-cli

The command line tool of [Mortise](https://github.com/nWidart/mortise), binary `mortise`. It reads a
Laravel app that uses [`nwidart/laravel-modules`](https://github.com/nWidart/laravel-modules), without
PHP, and checks its boundary rules in CI.

- `mortise check` prints the problems of the app, each with its file and line, and exits 1 when a rule
  breaks. `--format github` marks each problem on its line in a pull request.
- `mortise push` sends one read of the app to [Mortise Cloud](https://mortisecloud.com). The token comes
  from `MORTISE_TOKEN` only.

The tool writes nothing in the app.

## Install

```
cargo install --locked mortise-cli
```

Each [GitHub release](https://github.com/nWidart/mortise/releases/latest) also has a binary for Linux
(x86_64 and arm64), macOS (Apple silicon) and Windows (x64), with SHA-256 sums. In GitHub Actions,
the action [`nwidart/setup-mortise`](https://github.com/nWidart/setup-mortise) installs it.

## Use

```
mortise check /path/to/laravel-app --format github
MORTISE_TOKEN=... mortise push /path/to/laravel-app
```

The flags and exit codes are in the
[command line reference](https://nwidart.github.io/mortise/reference/command-line/).

## Licence

MIT.
