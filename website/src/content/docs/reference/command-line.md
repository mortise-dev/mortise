---
title: Command line tool
description: The flags, output formats and exit codes of mortise check, the command line tool that checks the boundary rules of an app in CI.
sidebar:
  order: 16
---

The command line tool is the binary `mortise`. It reads an app through the same code as the window,
so the two report the same findings. It runs no PHP, writes nothing in the app, and writes only to
standard output and standard error. The window is a separate binary, `mortise-desktop`.

## mortise check

```
mortise check [PATH] [--coverage <clover.xml>] [--coverage-limit <0-100>]
              [--format text|json|github] [--fail-on warnings]
```

| Argument | Meaning |
|---|---|
| `PATH` | The app folder: the one that holds `artisan`. Default: the current folder |
| `--coverage <clover.xml>` | The clover report, relative to the current folder. Wins over `coverage.report` in `mortise.json`. A report that cannot be read, or is no clover report, is an error (exit 2): no other report is used instead |
| `--coverage-limit <0-100>` | A module covered less than this per cent is a warning. Wins over `coverage.limit` in `mortise.json` |
| `--format text\|json\|github` | How the findings are printed. Default: `text` |
| `--fail-on warnings` | Warnings fail the check too |
| `-h`, `--help` | Prints the help |
| `-V`, `--version` | Prints the version |

A flag takes its value as the next argument or after `=`: `--format github` or `--format=github`.

`mortise check` judges one commit. It fails on every problem it finds, with no baseline and no
comparison with another branch.

## Output

Problems come first, then warnings, in the order of the Findings tab. Paths are relative to the app
folder, with `/` between the parts on every platform. A control character in a value from the app,
such as a line end in a file name, is printed as its escape (`\n`), so content of the app never starts
a line of its own. The `github` format escapes `%`, carriage returns and line ends as the GitHub runner
reads them.

### text

One line per problem, with its file and line, then a summary:

```
Modules/Checkout/app/Listeners/RequestShipment.php:6: problem: Checkout and Shipping use each other. 2 files in Checkout, 4 files in Shipping.
Modules/Checkout/app/Http/Controllers/OrderController.php:5: problem: Checkout reads Modules\Billing\Models\Invoice directly. Billing offers Contracts and Events.
Modules/Checkout/app/Listeners/CreateInvoiceOnPaid.php:5: problem: Checkout reads Modules\Billing\Models\Invoice directly. Billing offers Contracts and Events.
Modules/Checkout/app/Services/InvoiceSync.php:5: problem: Checkout reads Modules\Billing\Models\Invoice directly. Billing offers Contracts and Events.
Modules/Notifications: warning: Notifications is 39% covered. The limit is 50%.
2 problems, 1 warning. 7 modules checked against 2 rules and 4 checks.
```

- A circle is placed on the first reference of the link from its module to the next module of the
  circle, the same reference the Boundary tab lists first.
- A module that reads internals of another gives one line for each file that does, at the first line
  that names an internal class.
- A finding of a whole module, such as a coverage warning, is placed on the module folder.

### github

The same lines as GitHub Actions workflow commands: `::error` for a problem, `::warning` for a warning,
with the file, the line and the name of the check as the title. The summary follows as a plain line.

```
::error file=Modules/Checkout/app/Services/InvoiceSync.php,line=5,title=Public surface only::Checkout reads Modules\Billing\Models\Invoice directly. Billing offers Contracts and Events.
```

### json

The findings as one JSON object, as `mortise-core` writes them: `list` (the findings), `checks` (each
check that ran, with how many things it looked at and how many findings it gave) and `todo`. Nothing
else is printed on standard output.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | No finding fails the check. Also when a problem is found and `ci.fail` in `mortise.json` is `false` |
| 1 | A problem is found, or a warning with `--fail-on warnings` |
| 2 | The command line is wrong: an unknown command or flag, a missing or wrong value, or a report given with `--coverage` that cannot be read |
| 3 | The folder cannot be read as an app: it is not a Laravel app, or it has no `composer.lock`, or it does not use `nwidart/laravel-modules` |
| 5 | The report could not be written to standard output, such as on a full disk. A closed pipe, as with `\| head`, is not an error |

Code 4 is kept for a later command.

## Related pages

- [How to check the rules in CI](/mortise/how-to/check-rules-in-ci/)
- [mortise.json](/mortise/reference/mortise-json/)
- [Findings tab](/mortise/reference/findings/)
