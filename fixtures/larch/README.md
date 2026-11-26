# The shared fixtures of Larch

These apps are a copy of `tests/Fixtures/mortise` of [nwidart/larch](https://github.com/nWidart/larch),
version 0.2.0, commit `4be61378d28770dd5cae973cada32528a4163cf0`. Larch's preset,
`arch()->preset()->mortise()`, checks the rules of `mortise.json` in PHP. Each app has an
`expected.json` with the result Larch expects. `crates/mortise-core/tests/larch.rs` reads each app
with Mortise and compares. The README of that folder in Larch describes the apps and the format of
`expected.json`.

The copy keeps every file of Larch as it is. Mortise reads two files that Larch does not, so each
app has them in addition:

- `artisan`, so the folder is a Laravel app. It is never run.
- `composer.lock`, with the entry of `nwidart/laravel-modules` only, so Mortise knows the version
  of the package and its layout.

## Where Larch differs

`all-off`, `custom-folders` and `defaults` have no `modules_statuses.json`. Larch turns off only a
module whose entry is `false`, so it counts each module of these apps on. laravel-modules turns off a
module that has no entry (`FileActivator::hasStatus`), and Mortise does the same. So in Mortise every
module of these apps is off, and nothing breaks a rule. The test checks that, and then checks the
apps again with a statuses file that turns every module on: Mortise then gives what Larch expects.

To copy a newer version, replace the folders, keep `artisan` and `composer.lock`, and change the
version and the commit above.
