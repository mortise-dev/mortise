# Recorded package output

Each `*.json` file holds the real output of one `php artisan` command of `nwidart/laravel-modules`.
The keys are `command`, `args`, `made_with`, `exit`, `stdout` and `stderr`, and the optional `root`. The text keeps its escape codes.
The fake artisan of the tests replays the file whose `args` match. `args` are unique across files.
The files were recorded once, from a scratch app that is not kept, with the versions in `made_with`.
Paths in the output are those of that app. `root` is the folder of that app, as the package prints it, with `/` as separator.
The fake artisan replaces every `root` in the output with its working folder, also written with `/`. A file without `root` replays as it is.
Recordings of 13.3.0 have no `root`. Recordings of 13.4.0 have one. Do not edit the files by hand; record them again.
The recordings were made with no agent variables in the environment, because `laravel/agent-detector` turns colours off when it finds one.

The `--json` commands of 13.4.0 were recorded with the app as it is described in `made_with`, with modules Billing and Checkout.
`module-migrate-status-no-table.json` was recorded before any migration ran, on an empty SQLite file.
`module-make-model-policy-exists-*.json` were recorded after `module:make-policy InvoicePolicy Billing`.
`module-make-model-factory-exists-dry-run.json` and `module-make-model-policy-taken-out-json.json` were recorded later, in another scratch app with the same versions.
`module-make-model-policy-taken-out-json.json` ran after `module:make-policy InvoicePolicy Billing`, and `module-make-model-factory-exists-dry-run.json` after it and `module:make-factory Invoice Billing`.
The script that recorded them wrote the folder of that app as `/tmp/scratch-app`, in `root` and in the output.
`module-test-fail.json` ran a test that fails on purpose, and `module-test-all.json` ran with that test in place.
The output of `module:generators --json` is one line of 48552 bytes (48553 with the newline), under the 64 KiB line limit of standard output.
