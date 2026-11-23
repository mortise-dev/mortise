# Recorded package output

Each `*.json` file holds the real output of one `php artisan` command of `nwidart/laravel-modules`.
The keys are `command`, `args`, `made_with`, `exit`, `stdout` and `stderr`. The text keeps its escape codes.
The fake artisan of the tests replays the file whose `args` match. `args` are unique across files.
The files were recorded once, from a scratch app that is not kept, with the versions in `made_with`.
Paths in the output are those of that app. Do not edit the files by hand; record them again.
