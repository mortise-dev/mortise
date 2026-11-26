## ADDED Requirements

### Requirement: The allowed use rule needs no new schema version
The push document SHALL carry the check `AllowedUse`, its details `OutsideAllowedUse` and `UnknownAllowedModule`, and the member `rules.allowed_use` when `mortise.json` has the key, without raising `SCHEMA_VERSION`. The contract allows a new `Check` variant and an optional member. `rules.allowed_use` SHALL be left out when the file has no key.

#### Scenario: The fixture layered
- **WHEN** the document of `fixtures/layered` is built
- **THEN** it is valid against `push-document-1.schema.json`, `schema_version` is 1, and the first finding has the check `AllowedUse` and the detail `OutsideAllowedUse` with `of` Billing

#### Scenario: The fixture shop-api
- **WHEN** the document of `fixtures/shop-api`, which has no `mortise.json`, is built
- **THEN** it equals the committed file, with no `allowed_use` member
