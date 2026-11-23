# Spec Delta

## MODIFIED Requirements

### Requirement: The client asks for an exposed endpoint only
The client SHALL read the links at the base path first. It SHALL ask for `info`, `mappings`, `scheduledtasks` and `prometheus` only when the base path lists them. It SHALL name each of `info`, `mappings` and `scheduledtasks` that is not listed. The Metrics tab SHALL say when `prometheus` is not listed.

#### Scenario: mappings is not exposed
- **WHEN** the base path lists `health` and `info` only
- **THEN** the client asks for `health` and `info`, and reports `mappings` and `scheduledtasks` as not exposed

#### Scenario: prometheus is not exposed
- **WHEN** the base path does not list `prometheus` and the Metrics tab is open
- **THEN** no request for `prometheus` leaves Mortise

### Requirement: Every request carries the credentials of the environment
Each request SHALL carry the credentials of the environment: a bearer token, HTTP basic, or a named header. Each request SHALL carry `Accept: application/json`, the request for `prometheus` included, so that the app answers an error as JSON and not as a redirect to a login page.

#### Scenario: A bearer token
- **WHEN** the environment uses a bearer token
- **THEN** every request carries `Authorization: Bearer <token>`

#### Scenario: The text endpoint
- **WHEN** the client asks for `prometheus`
- **THEN** the request carries `Accept: application/json` and the credentials, and the client reads the answer as text

### Requirement: Everything is asked on open, every 5 minutes and on refresh
The client SHALL ask for the base path, `health`, `info`, `mappings` and `scheduledtasks` when the environment opens, every 5 minutes while the window is visible, and when the user reads the app again. Between two of these, a `health` answer SHALL keep the rest of the last full answer. `prometheus` is not part of this: it has its own pace.

#### Scenario: The user reads the app again
- **WHEN** the user presses Ctrl R
- **THEN** the client asks for the base path, `health` and each exposed endpoint of `info`, `mappings` and `scheduledtasks`, at once

#### Scenario: The environment opens with the Metrics tab closed
- **WHEN** an environment opens and the Metrics tab is not open
- **THEN** the client does not ask for `prometheus`

## ADDED Requirements

### Requirement: prometheus is read as text
The client SHALL read the answer of `prometheus` as text. A 200 with text SHALL be the metrics. A 200 with an empty body SHALL mean that metrics are off. A 503 SHALL be an answer of the app, read with the first line of its body. A 401 or 403 SHALL be "not authorized". Any other status SHALL be an unexpected answer that names the status.

#### Scenario: The store of the app cannot be read
- **WHEN** `prometheus` answers 503 with `The metrics store cannot be read.`
- **THEN** the client reports that the app has no metrics now, with that line as the app wrote it

#### Scenario: Metrics are off
- **WHEN** `prometheus` answers 200 with an empty body
- **THEN** the client reports that metrics are off, not a failed request

### Requirement: The body of a metrics answer has a limit
The client SHALL read at most 16 MiB (16,777,216 bytes) of a `prometheus` answer, counted after decompression. A longer body SHALL be the answer "too large", which names the address and the limit. It SHALL NOT fail the worker and SHALL NOT be kept.

#### Scenario: A body over the limit
- **WHEN** `prometheus` answers 200 with 16 MiB and one byte of text
- **THEN** the client reports "too large" with the limit, and the next `health` request leaves on time

#### Scenario: A body at the limit
- **WHEN** `prometheus` answers 200 with exactly 16 MiB of text
- **THEN** the client reads it as the metrics

### Requirement: prometheus is asked only while the Metrics tab is open and the window is visible
The client SHALL ask for `prometheus` every 15 s while the Metrics tab is open, the window is visible, the last full answer lists `prometheus`, and the environment answered. In every other case it SHALL NOT ask for `prometheus`: no request SHALL start after the frame in which the tab closes. The first request SHALL leave when the tab opens, unless one left less than 15 s ago.

#### Scenario: The tab is closed
- **WHEN** an environment is open for a minute and the Metrics tab was never opened
- **THEN** no request for `prometheus` left Mortise, and `env` was never asked for

#### Scenario: The tab closes
- **WHEN** the user leaves the Metrics tab
- **THEN** no further request for `prometheus` leaves Mortise from that frame on, a read already on its way ends, and `health` is still asked every 15 s

#### Scenario: The window is hidden
- **WHEN** the Metrics tab is open and the window has not drawn itself for longer than the visibility grace
- **THEN** no request leaves Mortise until it draws itself again

### Requirement: A metrics read does not replace a health request
The client SHALL send one request at a time for an environment. When `health` and `prometheus` are due together, `health` SHALL go first. A `prometheus` read SHALL delay a `health` request by at most the time that read takes, and SHALL NOT skip one.

#### Scenario: A slow metrics answer
- **WHEN** `prometheus` takes 10 s to give no answer
- **THEN** the next `health` request leaves when that read ends, and the status on screen stays that of the last `health` answer
