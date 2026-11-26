# mortise-core

The library behind [Mortise](https://github.com/nWidart/mortise): it reads a Laravel app that uses
[`nwidart/laravel-modules`](https://github.com/nWidart/laravel-modules) into a snapshot, without PHP.
The snapshot holds the modules, which module uses which, circular use, the boundary rules of
`mortise.json` and the findings.

It is published so that `cargo install mortise-cli` can build the command line tool. Its API serves the
Mortise desktop app and the tool, and changes with them: `mortise-cli` depends on exactly the version
of its own release. The feature `testing` builds only in the Mortise repository, where its fixtures are.

## Licence

MIT.
