# push-document Specification

## Purpose
Defines the push document: the one read of an app that `mortise push` sends to Mortise Cloud, version 1.

## Requirements

### Requirement: The core builds the push document behind a feature
`mortise-core` SHALL build the push document in its folder `push/`, compiled only with the feature `push`. The feature SHALL add no dependency. `SCHEMA_VERSION` SHALL be a public constant, `1`.

#### Scenario: The window's build
- **WHEN** `mortise-app` is built
- **THEN** `mortise-core` is built without the feature `push`

### Requirement: The projection keeps no machine detail
The push document SHALL hold the snapshot of the app without `root`, `app_url`, `app_env`, `git` and `package_data`. Every path in it SHALL be relative to the app folder with `/` between parts, on every platform: the layout's folders, module folders, the coverage report, the file of each reference and the file or folder of each finding. `uncommitted` SHALL be null for every module. `churn` SHALL be null for every module when the clone is shallow.

#### Scenario: A shallow clone
- **WHEN** the document of shop-api is built with `git.shallow` true
- **THEN** no member holds an absolute path, and `churn` and `uncommitted` are null for each module

#### Scenario: Modules outside the app folder
- **WHEN** the modules folder and the statuses file lie beside the app
- **THEN** the serialized document holds no part of their absolute path, and their paths, the reference files and the finding files start with `../`

#### Scenario: A report outside the app folder
- **WHEN** the coverage report lies in a folder beside the app
- **THEN** its path in the document starts with `../`

### Requirement: The shape is frozen by a test
A test SHALL serialize the document of `fixtures/shop-api` with a fixed envelope and compare it with a committed file. The same document SHALL be valid against the cloud's `push-document-1.schema.json`, and SHALL hold every member of the cloud's `example-shop-api.json` with the same value, except `churn`. A change that removes or renames a member or changes its type SHALL raise `SCHEMA_VERSION`.

#### Scenario: A renamed member
- **WHEN** a field of a type in the document is renamed
- **THEN** the committed file and the cloud's example no longer match, and the test fails

### Requirement: The schema has one public copy
The repository SHALL keep the JSON Schema of the push document once, as `website/public/schema/push-document-1.json`. The docs site SHALL publish it at its `$id`, `https://mortise-dev.github.io/mortise/schema/push-document-1.json`. The tests SHALL read that file. A new schema version SHALL get a new file, and the file of an older version SHALL stay.

#### Scenario: The shape test
- **WHEN** the shape test validates a document
- **THEN** it loads the schema from `website/public/schema/` and checks that the `$id` is the public URL

### Requirement: The size limits of the contract
The core SHALL report when a document has more than 2,000 modules, 50,000 links, 100,000 references or 50,000 findings, or a path or a class name of a reference longer than 1,024 characters. The paths are `app_path`, the layout's folders, module folders, the coverage report, reference files and finding files.

#### Scenario: A long class name
- **WHEN** a module names a class of another module with a name of 1,100 characters
- **THEN** the document is over a limit, and the limit names the class

### Requirement: A module carries its accepted surface
Each module of the push document SHALL carry its accepted public surface as the member `surface`: an object with `classes`, `routes` and `commands`, or null when it has none. The member SHALL be added without raising `SCHEMA_VERSION`, because the contract raises it only for a member that is removed, renamed or changes type.

#### Scenario: The document of shop-api
- **WHEN** the document of shop-api is built
- **THEN** Users has a `surface` with one class, every other module has a null `surface`, and the document is valid against `push-document-1.schema.json`

### Requirement: The allowed use rule needs no new schema version
The push document SHALL carry the check `AllowedUse`, its details `OutsideAllowedUse` and `UnknownAllowedModule`, and the member `rules.allowed_use` when `mortise.json` has the key, without raising `SCHEMA_VERSION`. The contract allows a new `Check` variant and an optional member. `rules.allowed_use` SHALL be left out when the file has no key.

#### Scenario: The fixture layered
- **WHEN** the document of `fixtures/layered` is built
- **THEN** it is valid against `push-document-1.schema.json`, `schema_version` is 1, and the first finding has the check `AllowedUse` and the detail `OutsideAllowedUse` with `of` Billing

#### Scenario: The fixture shop-api
- **WHEN** the document of `fixtures/shop-api`, which has no `mortise.json`, is built
- **THEN** it equals the committed file, with no `allowed_use` member

### Requirement: The commit time and the backfill member need no new schema version
The push document SHALL carry `git.committed_at`, a string or null, and `backfill`, null or an object with the integers `position` and `of`, both from 1, without raising `SCHEMA_VERSION`. The schema SHALL require neither member, so a document of Mortise 1.0.0 stays valid. When `backfill` is an object, the schema SHALL require `git.committed_at` to be a string, and `git.pull_request` and `environment` to be null.

#### Scenario: The document of shop-api
- **WHEN** the document of shop-api is built with a commit time and no backfill
- **THEN** it equals the committed file, with `git.committed_at` and a null `backfill`, and it is valid with and without the two members

#### Scenario: A backfill document of a pull request
- **WHEN** `backfill` is an object and `git.pull_request` is a number, `git.committed_at` is null or `environment` is set
- **THEN** the document is not valid against `push-document-1.schema.json`
