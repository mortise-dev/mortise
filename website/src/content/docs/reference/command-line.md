---
title: Command line tool
description: The flags, output and exit codes of mortise check, which checks the boundary rules of an app in CI, and of mortise push, which sends a read to Mortise Cloud.
sidebar:
  order: 16
---

The command line tool is the binary `mortise`. It reads an app through the same code as the window,
so the two report the same findings. It runs no PHP and writes nothing in the app. `mortise check`
writes only to standard output and standard error. `mortise push` also sends one document to Mortise
Cloud. The window is a separate binary, `mortise-desktop`.
[Install the command line tool](/mortise/start-here/installation/#install-the-command-line-tool) says
how to get it.

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
A value that starts with `-` goes after `=`: `--output=-x.json`. As the next argument it is an error.

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
  that names an internal class. The sentence ends with what the other module offers: its folders, such
  as "Billing offers Contracts and Events.", or "Billing offers its accepted public surface." when its
  `module.json` has a `surface` key.
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

### Exit codes of check

| Code | Meaning |
|---|---|
| 0 | No finding fails the check. Also when a problem is found and `ci.fail` in `mortise.json` is `false` |
| 1 | A problem is found, or a warning with `--fail-on warnings` |
| 2 | The command line is wrong: an unknown command or flag, a missing or wrong value, or a report given with `--coverage` that cannot be read |
| 3 | The folder cannot be read as an app: it is not a Laravel app, or it has no `composer.lock`, or it does not use `nwidart/laravel-modules` |
| 5 | The report could not be written to standard output, such as on a full disk. A closed pipe, as with `\| head`, is not an error |

## mortise push

```
mortise push [PATH] [--coverage <clover.xml>] [--environment <name>]
             [--dry-run] [--output <file>]
             [--commit <sha>] [--branch <name>] [--pull-request <n>]
             [--base-branch <name>] [--base-commit <sha>] [--repository <owner/name>]
```

`mortise push` reads the app as `check` does and sends one push document to Mortise Cloud: the
modules, links, cycles, rules and findings of one commit, with the commit and the CI run it belongs to.
It sends no source code, nothing from `.env` and no absolute path. Paths are relative to the app folder.

| Argument | Meaning |
|---|---|
| `PATH` | The app folder. Default: the current folder |
| `--coverage <clover.xml>` | The clover report, relative to the current folder. Wins over `coverage.report` in `mortise.json` |
| `--environment <name>` | This run deploys the commit to this environment, such as `production`. The cloud records a deploy. Lower-case letters, digits and `-`, at most 32 |
| `--dry-run` | Prints the document on standard output and sends nothing. Needs no token |
| `--output <file>` | Writes the document to this file. With `--dry-run`, instead of standard output |
| `--commit <sha>` | The full SHA of the commit that was read. Default: `HEAD` of the checkout, else `GITHUB_SHA`. When the two differ, the tool warns and names both |
| `--branch <name>` | Default: the head branch of the pull request, the branch of `GITHUB_REF`, or the branch of `HEAD` |
| `--pull-request <n>` | The pull request number. Default: the event of a GitHub Actions run. Needs `--base-branch` and `--base-commit` when no event file gives them |
| `--base-branch <name>` | The base branch of the pull request |
| `--base-commit <sha>` | The tip of the base branch |
| `--repository <owner/name>` | Default: `GITHUB_REPOSITORY` |

A flag wins over the environment. `app_path`, the app folder relative to the top of the git
repository, and `shallow` always come from git. On a shallow clone the commits per module are left out.

### Environment

| Variable | Meaning |
|---|---|
| `MORTISE_TOKEN` | The token of the app in Mortise Cloud. Read from here only, sent only in the `Authorization` header, never printed |
| `MORTISE_CLOUD_URL` | The cloud host. Default: `https://mortisecloud.com`. Must be `https` and name no user. `http` works only with the host `localhost`, `127.0.0.1` or `[::1]`, and then uses no proxy |
| `GITHUB_ACTIONS`, `GITHUB_SHA`, `GITHUB_REF`, `GITHUB_REPOSITORY`, `GITHUB_RUN_ID`, `GITHUB_RUN_ATTEMPT`, `GITHUB_SERVER_URL` | Read on GitHub Actions for the commit, the branch and the run |
| `GITHUB_EVENT_NAME`, `GITHUB_EVENT_PATH` | On `pull_request` and `pull_request_target`, the event file gives the number, the head and the base |

### Sending

The document goes gzipped to `POST /api/v1/snapshots` on the cloud host. A redirect is not followed.
Text from the cloud is printed with the token replaced by `[MORTISE_TOKEN]`.
The tool checks the limits of the cloud before it sends: 2,000 modules, 50,000 links, 100,000
references, 50,000 findings, 1,024 characters for any path or the class name of a reference, 16 MiB of JSON
and 4 MiB as sent.

| Answer | What the tool does |
|---|---|
| `200`, `202` | Prints `Sent to Mortise Cloud:` and the snapshot URL, and any warning of the cloud |
| `402`, `403` | A plan limit or an inactive app. Warns and exits 0 |
| `429`, `5xx`, no answer | Tries again after 2, 4 and 8 seconds, or after `Retry-After` up to 60 seconds. Then warns and exits 0 |
| Any other | Prints the problem and each member the cloud names, and exits 4 |

An outage of the cloud never fails the build. A wrong token or document fails it, because only you can
fix it.

### Exit codes of push

| Code | Meaning |
|---|---|
| 0 | Sent. Also: not sent because the cloud could not be reached or refused for a plan reason, with a warning; or a pull request from a fork without a token, with a notice |
| 2 | The command line is wrong, `MORTISE_TOKEN` is not set, the commit is unknown, or `MORTISE_CLOUD_URL` is not `https` |
| 3 | The folder cannot be read as an app |
| 4 | The cloud refused the document: token, schema, size or repository. Or the document is over a limit before sending |
| 5 | The document could not be written to `--output` or standard output |

`mortise push` does not compare with a base branch, keep a history, call the GitHub API or post a
comment. Mortise Cloud does that with what it receives.

## Related pages

- [How to check the rules in CI](/mortise/how-to/check-rules-in-ci/)
- [How to send a read to Mortise Cloud](/mortise/how-to/send-to-mortise-cloud/)
- [mortise.json](/mortise/reference/mortise-json/)
- [Findings tab](/mortise/reference/findings/)
