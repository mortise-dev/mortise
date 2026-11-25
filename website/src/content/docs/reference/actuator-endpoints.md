---
title: Actuator endpoints
description: The endpoints of nwidart/laravel-actuator that Mortise reads, how often, with which credentials, and what each gives.
sidebar:
  order: 14
---

Mortise reads a running app over HTTP, through `nwidart/laravel-actuator`. It needs no PHP and
writes nothing. Each environment of an app in [Settings](/mortise/reference/settings/#environments)
names a base URL and an actuator path. An endpoint is at `<base URL>/<actuator path>/<endpoint>`,
such as `https://shop-api.test/actuator/health`.

## The endpoints

Mortise asks for these six only, with `GET`.

| Endpoint | When Mortise asks | What it gives Mortise |
|---|---|---|
| Base path (`/actuator`) | When the environment opens, every 5 minutes, and when you read the app again | The endpoints the app exposes, from the keys of `_links`. An endpoint that is not listed is not asked for |
| `health` | As above, and every 15 s while the window is visible | The status of the app and of each component: Up, Down, Out of service or Unknown, with the details of each component. The index marks each module by its own health indicators |
| `info` | With the base path, when it is exposed | The environment, the Laravel version, the git branch and commit, the PHP version and SAPI, and the memory in use, its peak and its limit |
| `mappings` | With the base path, when it is exposed | The routes: methods, URI, name, action and middleware. Mortise places each route in the module of its action |
| `scheduledtasks` | With the base path, when it is exposed | The scheduled tasks: target, cron expression or interval, timezone and next run. Each task shows in the module that declares its command |
| `prometheus` | Every 15 s, only while the Metrics tab is open and the window is visible | The metrics, in the Prometheus text format. See [Metrics tab](/mortise/reference/metrics/) |

The rules of the client:

- It never asks for `env`, or for any endpoint not in this table.
- It follows no redirect. A redirect could carry a custom header with the secret to another host.
  A 3xx answer comes back as an unexpected answer that names where it sends to.
- It sends `Accept: application/json` on every request, so that Laravel answers errors as JSON.
- One request is on its way at a time for an environment.
- After the Metrics tab closes, no request for `prometheus` leaves.
- It trusts the certificates this computer trusts, so a local site with its own certificate
  authority works as it does in the browser.

## Time limits

| Limit | Value |
|---|---|
| Wait for one endpoint | 10 s. Then the answer is "no answer in 10 s" |
| Body of one `prometheus` answer | 16 MiB after decompression. A longer body gives no metrics |

## Credentials

Each request carries the credentials of the environment.

| Kind in Settings | Header sent |
|---|---|
| None | No header |
| Bearer token | `Authorization: Bearer <token>` |
| HTTP basic | `Authorization: Basic <base64 of user:password>` |
| Custom header | `<header name>: <header value>` |

The secret comes from the credential store of the operating system.
See [Files and folders](/mortise/reference/files-and-folders/#credential-store).

## Answers

| Status | How Mortise reads it |
|---|---|
| 200 | The answer. For `prometheus`, an empty body means metrics are off in the app |
| 503 | Also an answer: the app is down or out of service, and says so in the body. For `prometheus`, Mortise keeps the first line of the body, at most 200 characters |
| 401, 403 | Not authorized |
| Any other, such as 404, 500 or a redirect | Unexpected, with the status, and for a redirect the address it sends to |
| No answer | A refused connection, a timeout, a host that was not found, or a certificate that is not trusted on this computer |

"No answer" and "Not authorized" are Mortise's words for the connection. They are not statuses of the app.

When `info`, `mappings` or `scheduledtasks` is not exposed, Mortise says which, and shows the line to
add to `config/actuator.php`:

```php
'exposure' => [
    'include' => ['health', 'info', 'mappings', 'scheduledtasks'],
],
```

For metrics, `prometheus` must be in that list too.

## Related pages

- [How to connect an actuator](/mortise/how-to/connect-actuator/)
- [Package versions](/mortise/reference/package-versions/)
- [About the runtime per module](/mortise/explanation/runtime-per-module/)
