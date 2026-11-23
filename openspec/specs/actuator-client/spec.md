# actuator-client Specification

## Purpose
Defines what the actuator client of Mortise asks a running app for, how often, and how it reads each answer.

## Requirements

### Requirement: The client asks for a fixed list of paths
The actuator client SHALL ask for the actuator base path, `health`, `info`, `mappings`, `scheduledtasks` and `prometheus` only. It SHALL NOT ask for any other endpoint.

#### Scenario: An app that exposes every endpoint
- **WHEN** the base path lists `health`, `info`, `mappings`, `scheduledtasks`, `prometheus`, `modules` and `env`
- **THEN** the client asks for none of `modules` and `env`, and for no path outside the list

### Requirement: The client never asks for env
The actuator client SHALL NOT ask for `env`, whatever the app exposes and whatever the user does.

#### Scenario: env is exposed
- **WHEN** the links at the base path include `env`
- **THEN** no request for `env` leaves Mortise

### Requirement: The client asks for an exposed endpoint only
The client SHALL read the links at the base path first. It SHALL ask for `info`, `mappings` and `scheduledtasks` only when the base path lists them, and SHALL name each one it reads that is not listed.

#### Scenario: mappings is not exposed
- **WHEN** the base path lists `health` and `info` only
- **THEN** the client asks for `health` and `info`, and reports `mappings` and `scheduledtasks` as not exposed

### Requirement: The client follows no redirect
The client SHALL NOT follow a redirect. A 3xx answer SHALL be an unexpected answer that names the full address it sends to.

#### Scenario: The app sends a redirect
- **WHEN** `health` answers 302 with `Location: /login`
- **THEN** the client asks for nothing at `/login`, and the answer names `/login` read against the address asked for

### Requirement: A 503 is an answer
The client SHALL read a 503 as an answer of the app, with its body, like a 200. A 401 or 403 SHALL be "not authorized". Any other status SHALL be an unexpected answer that names the status.

#### Scenario: The app is down
- **WHEN** `health` answers 503 with `{"status":"DOWN"}`
- **THEN** the client reports the app as Down, not as a failed request

#### Scenario: The credentials are refused
- **WHEN** `health` answers 401
- **THEN** the client reports "not authorized" with the address and the status

### Requirement: Every request carries the credentials of the environment
Each request SHALL carry the credentials of the environment: a bearer token, HTTP basic, or a named header. Each request SHALL ask for JSON with `Accept: application/json`.

#### Scenario: A bearer token
- **WHEN** the environment uses a bearer token
- **THEN** every request carries `Authorization: Bearer <token>`

### Requirement: No answer within the timeout is "no answer"
The client SHALL wait at most 10 s for one endpoint. A refused connection, a timeout or a name that does not resolve SHALL be "no answer", with a short reason.

#### Scenario: Nothing listens
- **WHEN** the base URL points to a port where nothing listens
- **THEN** the client reports "no answer" with the reason "connection refused"

### Requirement: health is asked every 15 s while the window is visible
While the window is visible, the client SHALL ask for `health` every 15 s. While the window is not visible, it SHALL ask for nothing.

#### Scenario: The window is hidden
- **WHEN** the window has not drawn itself for longer than the visibility grace
- **THEN** no request leaves Mortise until it draws itself again

### Requirement: Everything is asked on open, every 5 minutes and on refresh
The client SHALL ask for everything it reads when the environment opens, every 5 minutes while the window is visible, and when the user reads the app again. Between two of these, a `health` answer SHALL keep the rest of the last full answer.

#### Scenario: The user reads the app again
- **WHEN** the user presses Ctrl R
- **THEN** the client asks for the base path, `health` and each exposed endpoint it reads, at once
