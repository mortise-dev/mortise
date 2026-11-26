# mortise-core

The library behind [Mortise](https://github.com/mortise-dev/mortise): it reads a Laravel app that uses
[`nwidart/laravel-modules`](https://github.com/nWidart/laravel-modules) into a snapshot, without PHP.
The snapshot holds the modules, which module uses which, circular use, the boundary rules of
`mortise.json` and the findings.

It is published so that `cargo install mortise-cli` can build the command line tool. Its API serves the
Mortise desktop app and the tool, and changes with them: `mortise-cli` depends on exactly the version
of its own release. The feature `testing` gives a stub actuator for tests, with answers recorded from a real app inside
the crate. Its fake artisan replays recordings that only the Mortise repository holds.

## Licence

MIT.
