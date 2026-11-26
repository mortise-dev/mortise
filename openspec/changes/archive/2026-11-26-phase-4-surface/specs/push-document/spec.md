# Spec Delta

## Purpose

Defines the push document: the one read of an app that `mortise push` sends to Mortise Cloud, version 1.

## ADDED Requirements

### Requirement: A module carries its accepted surface
Each module of the push document SHALL carry its accepted public surface as the member `surface`: an object with `classes`, `routes` and `commands`, or null when it has none. The member SHALL be added without raising `SCHEMA_VERSION`, because the contract raises it only for a member that is removed, renamed or changes type.

#### Scenario: The document of shop-api
- **WHEN** the document of shop-api is built
- **THEN** Users has a `surface` with one class, every other module has a null `surface`, and the document is valid against `push-document-1.schema.json`
