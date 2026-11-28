# cli-backfill Specification

## Purpose
Defines `mortise backfill`, which sends one push document for each past week of the default branch, so that Mortise Cloud has a history on the first day. It reads each commit on its own, compares nothing and keeps nothing.

## Requirements

### Requirement: The token and the host are those of push
`mortise backfill` SHALL read the token from `MORTISE_TOKEN` and the host from `MORTISE_CLOUD_URL` under the rules of `mortise push` (`cli-push`). Without a token it SHALL exit 2, except on a pull request from a fork, where it prints a notice and exits 0, and on a dry run with `--weeks`.

#### Scenario: No token
- **WHEN** `mortise backfill --dry-run` runs without `MORTISE_TOKEN` and without `--weeks`
- **THEN** the tool exits 2 and names `--dry-run` with `--weeks`

### Requirement: A shallow clone sends nothing
On a shallow clone, with or without a token, the tool SHALL ask nothing and send nothing, say that it needs the full history and name `fetch-depth: 0` of `actions/checkout`, and exit 0.

#### Scenario: A clone of depth 1
- **WHEN** `mortise backfill` runs in a clone made with `--depth 1`
- **THEN** no request is made, standard error holds the notice, and the exit code is 0

### Requirement: The cloud says which period to fill
The tool SHALL ask `GET /api/v1/backfill` on the cloud host first. The answer holds `needed`, `since` (RFC 3339), `every` and `have` (commits the cloud has). When `needed` is false the tool SHALL print "History is already filled." and exit 0. When `every` is not `week`, or the answer cannot be read, it SHALL warn, send nothing and exit 0.

#### Scenario: Nothing is needed
- **WHEN** the cloud answers `{"needed": false}`
- **THEN** standard output is "History is already filled.", one request was made, and the exit code is 0

### Requirement: One commit per week, newest first
The tool SHALL walk the whole first-parent history of the commit that was checked out when the run began, keep the commits whose committer date is at or after `since`, and pick the last one of each ISO week in UTC by committer date. From these it SHALL leave out the commit at `HEAD` and every commit in `have`. It SHALL send the newest first. `--weeks <n>` SHALL move the start of the period to at most `n` weeks ago and never earlier than `since`.

#### Scenario: A history of ten weeks
- **WHEN** the cloud has the commit of one week and asks for a period that holds four weeks with commits, besides the week of `HEAD`
- **THEN** the tool sends the last commit of each of the three other weeks, the newest first

#### Scenario: A lower number of weeks
- **WHEN** `--weeks 1` is given and every past commit is older than one week
- **THEN** the tool prints "No past commits to send." and sends nothing

### Requirement: A past commit is read without a change to the checkout
The tool SHALL read the app at each commit from a temporary folder outside the app, which holds the files of the app folder at that commit and is removed after the read. The working tree, the index and `HEAD` of the checkout SHALL stay as they are, and no git hook SHALL run. Listing the commits SHALL download nothing. In a clone without file contents (`--filter=blob:none`), git downloads the missing files of the app folder of one commit in one request.

#### Scenario: A clone without file contents
- **WHEN** `mortise backfill` runs in a clone made with `--filter=blob:none`
- **THEN** each document holds the modules of its commit, and `git status` of the clone is clean

### Requirement: A past commit is read from its own files only
No filter program and no git hook of the computer SHALL run on the files of a past commit, and a link SHALL be written as a plain file. A commit whose modules folder or statuses file is outside the app folder SHALL be left out before anything is read from there. No file of a past commit SHALL be written inside the repository, also when the temporary folder of the computer is there. The tree SHALL be the one of the commit that the document names, not of a replacement.

#### Scenario: A filter that fails
- **WHEN** the repository sets a required filter whose program fails for every file
- **THEN** the past commit is still read

#### Scenario: Modules outside the app folder
- **WHEN** the `config/modules.php` of a past commit names an absolute modules folder outside the app
- **THEN** the commit is left out with one line

### Requirement: The app keeps its place in the repository
An app below the top of the repository SHALL be read under the `.gitignore` files that the commit holds in the folders above it, as a read of the checkout obeys them.

#### Scenario: A tracked file that is ignored from above
- **WHEN** the `.gitignore` at the top of the repository names a tracked file of a module
- **THEN** the document of a past commit counts the files of that module as a push of the checkout does

### Requirement: Each document stands for its commit
Each document SHALL carry the commit, its committer date as `git.committed_at`, `git.branch` set to the default branch, null pull request members, `git.shallow` false, a null `environment`, and `backfill` set to `{ "position": n, "of": m }`. The document SHALL hold no coverage, also when the commit holds a coverage report. The churn of a module SHALL count the commits of its history dated in the 30 days up to its committer date, also behind a commit with an older date.

#### Scenario: A commit older than 30 days
- **WHEN** a module had 3 commits in the 30 days before a past commit, all older than 30 days now
- **THEN** the document of that commit gives the module 3 commits

### Requirement: The default branch has three sources
The default branch SHALL be `--branch`, else the branch of the run on GitHub Actions, else the branch that is checked out. Without any of them the tool SHALL exit 2, except on a dry run, which writes no document.

#### Scenario: A detached HEAD
- **WHEN** no branch is checked out, `--branch` is not given and the run is not on GitHub Actions
- **THEN** the tool exits 2 and names `--branch`

### Requirement: The place of a document counts the documents that are sent
The tool SHALL read every picked commit before it sends the first document. A commit that cannot be read as an app, or whose document is over a limit of the cloud, SHALL be left out with one line on standard error, and the run goes on. The size limits SHALL be checked on the body as it is sent. `of` SHALL be the number of documents left, and `position` SHALL count from 1 with the newest.

#### Scenario: A commit from before the app
- **WHEN** one of three picked commits has no app folder
- **THEN** standard error names that commit, and the two documents are 1 of 2 and 2 of 2

### Requirement: A refusal never fails the build, a wrong token does
For each request the tool SHALL retry a `429`, a `5xx` and no answer as `mortise push` does: three waits, each as long as `Retry-After` asks, up to 60 seconds. After that, and on a `402` or a `403`, it SHALL warn, stop the run and exit 0. On any other refusal, such as a `401`, it SHALL print the problem, stop and exit 4. It SHALL end with one line that counts the documents sent and gives the day of the oldest one, such as "Sent 12 past commits, back to 31 Aug 2026."

#### Scenario: A rate limit between two documents
- **WHEN** the cloud answers `429` with `Retry-After` once, then accepts
- **THEN** the same document is sent again, and the run goes on

#### Scenario: A plan limit after the first document
- **WHEN** the cloud accepts one document and answers `402` to the next
- **THEN** the tool warns, prints "Sent 1 past commit, back to" and the day, and exits 0

### Requirement: A dry run reads and sends nothing
With `--dry-run` the tool SHALL print one line per picked commit, the SHA and the committer date, newest first, read no commit and send no document. With a token it asks the cloud for the period. With `--weeks` and no token it SHALL make no request.

#### Scenario: A dry run without a token
- **WHEN** `mortise backfill --dry-run --weeks 100` runs without `MORTISE_TOKEN`
- **THEN** standard output lists the last commit of each past week, and no request is made
