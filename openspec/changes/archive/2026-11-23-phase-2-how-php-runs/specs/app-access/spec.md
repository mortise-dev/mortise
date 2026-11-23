# Spec Delta

## ADDED Requirements

### Requirement: Mortise reads two lines of .env
From the `.env` file of an app, Mortise SHALL read `APP_URL` and `APP_ENV` and no other line.
`APP_URL` proposes the base URL of an environment. `APP_ENV` tells Mortise to ask before a command in
production.

#### Scenario: A .env with secrets
- **WHEN** `.env` holds `APP_ENV=production`, `APP_URL=https://shop.test` and `DB_PASSWORD=secret`
- **THEN** the app reads `production` and `https://shop.test`, and the password is not kept
