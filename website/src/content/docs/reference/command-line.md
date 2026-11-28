---
title: Command line tool
description: The flags, output and exit codes of mortise check, which checks the boundary rules of an app in CI, of mortise push, which sends a read to Mortise Cloud, and of mortise backfill, which sends past commits.
sidebar:
  order: 16
---

The command line tool is the binary `mortise`. It reads an app through the same code as the window,
so the two report the same findings. It runs no PHP and writes nothing in the app. `mortise check`
writes only to standard output and standard error. `mortise push` also sends one document to Mortise
Cloud, and `mortise backfill` one document for each past week. The window is a separate binary,
`mortise-desktop`.
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
folder, with `/` between the parts on every platform. A path outside the app folder, such as a modules
folder that `config/modules.php` sets to an absolute path, is printed whole, also with `/`:
`D:/shared/Modules` or `//server/share/shared/Modules` on Windows. A `..` in the folder is resolved first. A control character in a value from the app,
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
- A module that uses a module outside its list in `allowed-use` gives one line for each file that
  does, at the first line that names a class of the other module: "Audit uses Billing. mortise.json
  lets Audit use no module." A name in `allowed-use` that is no module is a warning placed on
  `mortise.json`.
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
repository, and `shallow` always come from git. On a shallow clone the commits per module are left out,
and the tool prints one notice: "shallow clone: commits per module are not sent. Set `fetch-depth: 0`
on `actions/checkout`." It still sends the document and exits 0.

`git.committed_at` is the committer date of the commit that the document names, in UTC, such as
`2026-11-24T14:01:52Z`. It is `null` when the repository does not hold that commit. The author date
is not sent. `backfill` is `null`: only [`mortise backfill`](#mortise-backfill) sets it.

### Schema of the push document

The JSON Schema of the push document is public. Schema version 1 is at
`https://mortise-dev.github.io/mortise/schema/push-document-1.json`, and the same URL is its `$id`.
A new schema version gets a new file, such as `push-document-2.json`. The file of an older version
stays where it is.

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

## mortise backfill

```
mortise backfill [PATH] [--weeks <n>] [--dry-run]
                 [--branch <name>] [--repository <owner/name>]
```

`mortise backfill` sends one push document for the last commit of each past week of the default
branch, so that Mortise Cloud has a history on the first day. Mortise Cloud says how far back to go
and which commits it has. The tool reads each commit on its own: it compares nothing and keeps nothing.

| Argument | Meaning |
|---|---|
| `PATH` | The app folder. Default: the current folder |
| `--weeks <n>` | For a test run. Reads no commit older than `n` weeks. It can lower what the cloud asks for and never raises it. After a run that sent documents, the cloud takes the history as filled and answers that nothing is needed, although the older weeks were not sent. A run with `--weeks` that has nothing to send tells the cloud nothing |
| `--dry-run` | Prints the commits it would read, one line each with the SHA and the committer date, and sends nothing. With `--weeks` it needs no token and asks nothing |
| `--branch <name>` | The default branch. Default: the branch of `GITHUB_REF`, or the branch of `HEAD` |
| `--repository <owner/name>` | Default: `GITHUB_REPOSITORY` |

It reads `MORTISE_TOKEN` and `MORTISE_CLOUD_URL` as `mortise push` does.

### Which commits

1. The tool asks `GET /api/v1/backfill` on the cloud host. The answer gives the start of the period
   and the commits the cloud has. When nothing is needed, the tool prints "History is already filled."
   and exits 0.
2. It walks the first-parent history of `HEAD` and keeps the commits whose committer date is in the
   period.
3. It picks the last commit of each week by committer date. A week runs from Monday to Sunday in UTC.
4. It leaves out the commit at `HEAD`, which `mortise push` sends, and the commits the cloud has.

The answer can name the default branch that is set on the app in the cloud. When the run reads another
branch, the tool sends nothing, prints a warning that names both branches, and exits 0. Run it on that
branch, or change the default branch in the settings of the app in Mortise Cloud.

When the cloud asks for a period and the run has no document to send, the tool tells the cloud that
the run is done, with `POST /api/v1/backfill` and no body. The cloud then stops asking. When that call
fails, the tool warns and exits 0, and the next run tells it again. A run with `--weeks` never sends it.

Run it on the default branch: the history it walks is that of the commit that is checked out.

A shallow clone has no past commits. There the tool sends nothing, with or without a token, prints "backfill needs the full
history. Set `fetch-depth: 0` on `actions/checkout`, then run the workflow again." and exits 0.

### How a past commit is read

The tool asks git to write the files of the app folder at that commit to a temporary folder outside
the app, reads them as it reads any app, and removes the folder. The working tree, the index and
`HEAD` of the checkout stay as they are. The files are those of a checkout of that commit, without
what could run a program: no git hook and no filter program runs, such as the one of Git LFS, and a
link is written as a plain file. An app below the top of the repository is read under the
`.gitignore` files of the folders above it, as in the checkout. A commit whose `config/modules.php` keeps the modules outside the app folder is left out,
because those files are not in the commit.

Listing the commits downloads nothing. In a clone made with `--filter=blob:none`, git downloads the
files of the app folder that the commit needs, in one request per commit, and keeps them in `.git` as
it does for any command. The clone needs its credentials for that: on `actions/checkout`, leave
`persist-credentials` on.

It reads every commit before it sends the first document. A commit that cannot be read as an app, such
as one from before the app used `nwidart/laravel-modules`, is left out with one line, and the run goes on.
So is a commit whose document is over a limit of the cloud.

### The documents

Each document is a push document as `mortise push` writes it, with these values:

| Member | Value |
|---|---|
| `git.commit`, `git.committed_at` | The past commit and its committer date |
| `git.branch` | The default branch |
| `git.pull_request` and the head and base members | `null` |
| `git.shallow` | `false` |
| `environment` | `null` |
| `backfill` | `{ "position": n, "of": m }`: document `n` of the `m` that the run sends, counted from the newest |
| Coverage | None. The tests of a past commit did not run, so a report in the commit is not read |
| `churn` of a module | The commits of the 30 days before that commit |

The newest commit goes first, so a run that stops early keeps the most useful part.

### Answers of the cloud

| Answer | What the tool does |
|---|---|
| `200`, `202` | Goes on with the next commit |
| `429`, `5xx`, no answer | Waits for `Retry-After`, up to 60 seconds, or 2, 4 and 8 seconds, and sends the same document again. After 3 waits in a row it warns, stops and exits 0 |
| `402`, `403` | A plan limit or an inactive app. Warns, stops and exits 0 |
| Any other | Prints the problem, stops and exits 4 |

The last line counts what was sent: "Sent 12 past commits, back to 31 Aug 2026."

### Exit codes of backfill

| Code | Meaning |
|---|---|
| 0 | Sent, or nothing to send. Also: stopped because the cloud could not be reached, limited the rate or refused for a plan reason, with a warning; a shallow clone; or a pull request from a fork without a token |
| 2 | The command line is wrong, `MORTISE_TOKEN` is not set, the folder is in no git repository with a commit, no branch is known, or `MORTISE_CLOUD_URL` is not `https` |
| 4 | The cloud refused the token or a document |

## Related pages

- [How to check the rules in CI](/mortise/how-to/check-rules-in-ci/)
- [How to send a read to Mortise Cloud](/mortise/how-to/send-to-mortise-cloud/)
- [mortise.json](/mortise/reference/mortise-json/)
- [Findings tab](/mortise/reference/findings/)
