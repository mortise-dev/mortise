# app-access Specification

## Purpose
States what Mortise may read, write and run in a Laravel app, so every phase keeps the same limits.

## Requirements

### Requirement: Reading an app needs no PHP
Mortise SHALL read an app from its files alone. Opening and reading an app MUST NOT start PHP or any program of the app.

#### Scenario: Open an app on a machine without PHP
- **WHEN** the user opens an app folder and no PHP is installed
- **THEN** Mortise lists the modules of the app and what is in each one

### Requirement: A command runs only when the user starts it
Mortise SHALL run a command in an app only when the user starts it. The exact command MUST be on screen first. The command MUST run through the PHP the user chose for the app.

#### Scenario: No command runs on its own
- **WHEN** the user opens, reads or refreshes an app
- **THEN** Mortise runs no command in the app

### Requirement: The only file Mortise writes is mortise.json
The only file Mortise itself SHALL write in an app is `mortise.json`, and it asks before it creates it. Files that a package command writes are written by that command, not by Mortise.

#### Scenario: Save the rules of a new app
- **WHEN** the user saves the rules of an app that has no `mortise.json`
- **THEN** Mortise asks first, and writes only `mortise.json` through a temporary file beside it

### Requirement: Mortise reads two lines of .env
From the `.env` file of an app, Mortise SHALL read `APP_URL` and `APP_ENV` and no other line.
`APP_URL` proposes the base URL of an environment. `APP_ENV` tells Mortise to ask before a command in
production.

#### Scenario: A .env with secrets
- **WHEN** `.env` holds `APP_ENV=production`, `APP_URL=https://shop.test` and `DB_PASSWORD=secret`
- **THEN** the app reads `production` and `https://shop.test`, and the password is not kept
