# cli-check Specification

## Purpose
Defines `mortise check`, the command line tool that checks the boundary rules of one app at one commit.

## Requirements

### Requirement: The tool reads without the optional features of the core
The binary `mortise` of the crate `mortise-cli` SHALL depend on `mortise-core` with `default-features = false`. It SHALL run no PHP, need no window system and write nothing in the app.

#### Scenario: Build the tool alone
- **WHEN** `cargo build -p mortise-cli` runs on a Linux machine without window libraries
- **THEN** the build succeeds and `mortise-core` is built with no feature

#### Scenario: Check an app
- **WHEN** `mortise check` reads an app
- **THEN** no file of the app is created or changed, and no PHP runs

### Requirement: Check reports problems first, each with its place
`mortise check [PATH]` SHALL read the app in `PATH`, or in the current folder, and print its findings in the order of `Findings`: problems first. In the text format each line SHALL give the place (a path relative to the app with `/` separators, and the line when there is one), the severity and the sentence. A finding of files that read internal classes SHALL give one line per file. A summary line SHALL follow.

#### Scenario: The fixture shop-api
- **WHEN** `mortise check fixtures/shop-api` runs
- **THEN** it prints one line for the circle between Checkout and Shipping, one line for each of the three files of Checkout that read `Modules\Billing\Models\Invoice`, the coverage warning of Notifications, and the summary, and exits 1

### Requirement: Three formats
`--format` SHALL accept `text` (the default), `json` and `github`. `json` SHALL print `Findings` as serde writes it and nothing else on standard output. `github` SHALL print each line of the text format as a GitHub Actions workflow command, `::error` for a problem and `::warning` for a warning, with `file` and `line` when the place is a file, the check as `title`, and the escapes of the runner.

#### Scenario: JSON
- **WHEN** `mortise check --format json` runs on shop-api
- **THEN** standard output parses as the `Findings` that `read_app` gives for shop-api

#### Scenario: GitHub
- **WHEN** `mortise check --format github` runs on shop-api
- **THEN** each of the three files that read `Invoice` gets its own `::error file=...,line=...` line, and the coverage warning has no `file`

### Requirement: Exit codes
`mortise check` SHALL exit 0 when no finding fails the check, 1 when one does, 2 on a usage error or a report given with `--coverage` that cannot be read as clover, 3 when the folder cannot be read as an app, and 5 when the report cannot be written to standard output for a reason other than a closed pipe. A problem fails the check; a warning fails it only with `--fail-on warnings`. Code 4 SHALL stay unused by `check`.

#### Scenario: No problem
- **WHEN** both rules are off in `mortise.json` and the only finding is a warning
- **THEN** the exit code is 0, and 1 with `--fail-on warnings`

#### Scenario: A usage error
- **WHEN** `--format sarif` is given
- **THEN** the tool prints the error on standard error and exits 2

#### Scenario: A given report that is no clover report
- **WHEN** `--coverage` names a file without a `coverage` element
- **THEN** the tool exits 2 and uses no other report

#### Scenario: Not an app
- **WHEN** `PATH` is an empty folder
- **THEN** the tool prints the read error on standard error and exits 3

### Requirement: Content of the app cannot start a line
Every value from the app or the command line that the tool prints, on standard output or standard error, SHALL have its control characters and line separators escaped. The github format SHALL also escape `%`, carriage returns and line ends as the GitHub runner reads them.

#### Scenario: A line end in a statuses entry
- **WHEN** `modules_statuses.json` lists an entry whose name holds a line end followed by `::error::`
- **THEN** no line of the output starts with `::error::` from that entry

### Requirement: Fail CI can be turned off
When `ci.fail` in `mortise.json` is `false`, `mortise check` SHALL print the same report, say on standard error that the check passes, and exit 0 where it would exit 1.

#### Scenario: A problem with ci.fail off
- **WHEN** `mortise.json` sets `"ci": { "fail": false }` and the app has problems
- **THEN** the problems are printed and the exit code is 0

### Requirement: The check judges one commit
`mortise check` SHALL NOT read or write a baseline, compare with another branch or commit, call the GitHub API, write SARIF, or store any history of reads.

#### Scenario: A problem that was there before
- **WHEN** a problem exists on the base branch too
- **THEN** `mortise check` reports it and fails as for a new one

### Requirement: Paths are written the same on every platform
Every path in the output of `mortise check`, in each format, SHALL be relative to the app folder with `/` between its parts on Linux, macOS and Windows. `.` and `..` SHALL be resolved by name before the test for inside the app. A path outside the app folder SHALL be written whole, with `/` between its parts and a Windows prefix kept: `D:`, or `//server/share` for a UNC path. A verbatim drive or share SHALL be written as the ordinary path it names; any other verbatim path SHALL keep its `\`.

#### Scenario: A file inside the app on Windows
- **WHEN** `mortise check fixtures/shop-api` runs on Windows
- **THEN** the circle is placed on `Modules/Checkout/app/Listeners/RequestShipment.php:6`, and no line holds a backslash in its path

#### Scenario: A modules folder outside the app
- **WHEN** the modules folder is `D:\shared\Modules` and the app is in `C:\ci\app`
- **THEN** the folder is written `D:/shared/Modules`, and on Unix `/shared/Modules` is written `/shared/Modules`

#### Scenario: A modules folder above the app
- **WHEN** `config/modules.php` sets `base_path('../shared/Modules')` and the app is in `/ci/app`
- **THEN** the folder is written `/ci/shared/Modules`, not `../shared/Modules`

#### Scenario: A modules folder on a share or a verbatim path
- **WHEN** the app is in `\\server\share\repo` and the modules folder is `\\server\share\shared\Modules` or `\\?\D:\shared\Modules`
- **THEN** the folder is written `//server/share/shared/Modules` or `D:/shared/Modules`, and the push document writes the first as `../shared/Modules`

#### Scenario: A broken module.json in json
- **WHEN** the job checks a copy of shop-api whose `Modules/Catalog/module.json` is not JSON, in the json format
- **THEN** the `file` of the one `InvalidManifest` finding is `Modules/Catalog/module.json`

### Requirement: CI runs the built tool on three platforms
A CI job SHALL build `mortise` alone on Linux, macOS and Windows and run it on the fixture shop-api: `check` in the text, github and json formats, `check` in json on a copy with a broken `module.json`, `check` on a folder that is not an app, and `push --dry-run` without `MORTISE_TOKEN`. It SHALL fail when an exit code or the key output differs.

#### Scenario: The fixture shop-api
- **WHEN** the job runs on any of the three platforms
- **THEN** each format exits 1 and names the circle and the three reads of Billing internals, the folder that is not an app exits 3, and the dry run exits 0 with the accepted surface of Users and only relative paths with `/`
