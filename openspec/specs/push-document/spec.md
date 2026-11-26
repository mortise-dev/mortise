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
A test SHALL serialize the document of `fixtures/shop-api` with a fixed envelope and compare it with a committed file. The same document SHALL be valid against a copy of the cloud's `push-document-1.schema.json` kept in the repository, and SHALL hold every member of the cloud's `example-shop-api.json` with the same value, except `churn`. A change that removes or renames a member or changes its type SHALL raise `SCHEMA_VERSION`.

#### Scenario: A renamed member
- **WHEN** a field of a type in the document is renamed
- **THEN** the committed file and the cloud's example no longer match, and the test fails

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
