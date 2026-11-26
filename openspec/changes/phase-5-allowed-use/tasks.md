# Tasks

## 1. Specification

- [x] 1.1 This change, with the deltas of `rules-file`, `cli-check` and `push-document`. Verify with `openspec validate --strict`

## 2. Core

- [x] 2.1 `AllowedUse` and `Rules::allowed_use`: read `rules.allowed-use` with its defaults and the parser message, save it and keep the lists. Verify with the tests of `boundary/rules.rs`
- [x] 2.2 `Check::AllowedUse` and `Detail::OutsideAllowedUse` in `findings::find`: a module without a list gives none, a module that is off gives none, an empty list means no module. Verify with one test on `fixtures/layered` in `tests/read_app.rs`
- [x] 2.3 `Detail::UnknownAllowedModule`: a warning per name that is no module of the app. Verify in the same test
- [x] 2.4 The fixture `fixtures/layered`: one break, one module without a list, one empty list

## 3. Command line tool

- [x] 3.1 One line per file of a break, a line on `mortise.json` for an unknown name, the title "Allowed use". Verify with one test in `tests/check.rs`: text, github and json on `layered`, exit 1, and exit 0 with `ci.fail` off

## 4. Push document

- [x] 4.1 The document of `layered` is valid against the schema copy, `SCHEMA_VERSION` stays 1, and the document of `shop-api` does not change. Verify with `cargo test -p mortise-core --test push`

## 5. Window and docs

- [x] 5.1 The smallest arms in `mortise-app` for the new variants, marked "not in the spec"
- [x] 5.2 The docs site pages on `mortise.json` and `mortise check`, and `CHANGELOG.md`
