# Spec Delta

## Purpose

Defines `mortise push`, the open source client that sends one read of an app to Mortise Cloud.

## ADDED Requirements

### Requirement: The token comes from the environment only
`mortise push` SHALL read the token from `MORTISE_TOKEN` only, send it only in the `Authorization` header to the cloud host, and never print or log it. Text from the cloud SHALL be printed with the token replaced by a mark.

#### Scenario: No token on a branch
- **WHEN** `MORTISE_TOKEN` is not set and the run is not a pull request from a fork
- **THEN** the tool says so on standard error and exits 2, without reading the app

#### Scenario: The cloud echoes the token
- **WHEN** a warning or an error of the cloud holds the token
- **THEN** the printed text holds `[MORTISE_TOKEN]` in its place

#### Scenario: No token on a fork
- **WHEN** `MORTISE_TOKEN` is not set on a `pull_request` run whose head repository is not the base repository
- **THEN** the tool prints a notice and exits 0

### Requirement: The token reaches the cloud host only
The tool SHALL read the host from `MORTISE_CLOUD_URL`, with `https://mortisecloud.com` as default. It SHALL parse the URL, refuse one that names a user, and refuse `http` unless the parsed host is exactly `localhost`, `127.0.0.1` or `[::1]`. A plain `http` request SHALL use no proxy.

#### Scenario: A user name before another host
- **WHEN** `MORTISE_CLOUD_URL` is an http URL whose user info reads `localhost:x` and whose host is `evil.example`
- **THEN** the tool exits 2 and sends nothing

#### Scenario: A proxy in the environment
- **WHEN** `HTTP_PROXY` is set and the host is `http://127.0.0.1:8000`
- **THEN** the request goes to `127.0.0.1:8000`, not to the proxy

### Requirement: A dry run sends nothing
With `--dry-run`, `mortise push` SHALL build the document, print it on standard output, or write it to the file of `--output`, and send nothing. It SHALL need no token.

#### Scenario: Dry run of shop-api
- **WHEN** `mortise push --dry-run fixtures/shop-api` runs with `--commit` given
- **THEN** standard output is the push document, and no request is made

### Requirement: The CI environment fills the envelope
On GitHub Actions the tool SHALL take the commit, the repository and the run from the environment, and on `pull_request` and `pull_request_target` events the number, the head commit and branch, and the base commit and branch from the event file. A flag SHALL win over the environment. Elsewhere the branch SHALL come from git. `git.shallow` and `app_path` SHALL always come from git.

#### Scenario: A pull request event
- **WHEN** `GITHUB_ACTIONS` is `true`, `GITHUB_EVENT_NAME` is `pull_request` and `GITHUB_EVENT_PATH` names an event file
- **THEN** the document's `git.pull_request`, `head_commit`, `branch`, `base_branch` and `base_commit` come from the file, and `ci.provider` is `github-actions`

### Requirement: Flags stand in for a missing event file
When the event file of a pull request run cannot be read, `--pull-request`, `--base-branch` and `--base-commit` SHALL give the pull request. Only without them SHALL the tool exit 2.

#### Scenario: No event file
- **WHEN** `GITHUB_EVENT_PATH` is not set on a `pull_request` run and the three flags are given
- **THEN** the document holds the pull request of the flags

### Requirement: The commit is the one that was read
Without `--commit`, the commit SHALL be `HEAD` of the checkout that was read when git gives it, else `GITHUB_SHA`. When the two differ the tool SHALL warn and name both.

#### Scenario: A job that checks out the head of a pull request
- **WHEN** `HEAD` is not `GITHUB_SHA`
- **THEN** the document names `HEAD`, and standard error names both

### Requirement: A flag does not take a flag as its value
A value given as a separate argument SHALL NOT start with `-`; such a value is a usage error. A value after `=` may start with `-`.

#### Scenario: A flag left without its value
- **WHEN** `mortise push --output --dry-run fixtures/shop-api` runs
- **THEN** the tool exits 2 and sends nothing

### Requirement: Answers of the cloud and exit codes
The tool SHALL send the document gzipped to `POST {host}/api/v1/snapshots` and follow no redirect. It SHALL exit 0 on `200` or `202` and print the snapshot URL and any warning. It SHALL retry `429`, any `5xx` and a failed connection 3 times, after 2, 4 and 8 seconds or after `Retry-After` up to 60 seconds, then warn and exit 0. It SHALL warn and exit 0 on `402` and `403`. It SHALL exit 4 on any other answer, and print the problem's title, detail and errors.

#### Scenario: Accepted
- **WHEN** the cloud answers `202` with a URL
- **THEN** the tool prints the URL and exits 0

#### Scenario: A refused token
- **WHEN** the cloud answers `401`
- **THEN** the tool prints the problem and exits 4

#### Scenario: An outage
- **WHEN** the cloud answers `503` four times
- **THEN** the tool asks four times, warns and exits 0

### Requirement: Limits are checked before sending
The tool SHALL check the limits of the contract before it sends, and exit 4 with the limit named when the document is over one: the counts and lengths of the core, 16 MiB for the document and 4 MiB for the gzipped body.

#### Scenario: A class name over the limit
- **WHEN** a module names a class of another module with a name of 1,100 characters
- **THEN** the tool sends nothing and exits 4

### Requirement: The client sends one read
`mortise push` SHALL send only the push document. It SHALL NOT compare with a base, keep a history, call the GitHub API, or post a comment or a check run.

#### Scenario: A pull request run
- **WHEN** `mortise push` runs on a pull request
- **THEN** it makes one request to the cloud host and none to another host
