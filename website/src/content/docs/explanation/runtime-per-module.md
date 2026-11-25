---
title: The runtime of each module
description: How Mortise turns the health and metrics of a running app into a status and figures per module, and why it keeps them in memory only.
sidebar:
  order: 4
---

Files tell Mortise how an app is built. They do not tell it how the app behaves. Is the storage of the
Billing module reachable? How many requests does Checkout serve, and how slowly? Those answers live in
a running copy of the app. This page explains how Mortise reads them and ties them to modules.

## Why a running app, and why the actuator

Mortise reads a running app through `nwidart/laravel-actuator`, over HTTP. It asks an environment you
add in Settings, such as `local` or `staging`. This needs no PHP on your computer and writes nothing in
the app. It is optional: without an environment, every other screen works as before.

[What Mortise changes, and what it never does](/mortise/explanation/safety/)
lists the endpoints Mortise asks for, and why `env` is not one of them.

What the running app reports is kept apart from what Mortise works out from files. A status from the
app is Runtime. A broken rule is a Finding. The two never mix, because they answer different questions:
one is about the app as it runs now, the other about the code as it is written.

## Health indicators, by name

A Laravel app reports its health as components: the database, the cache, the queue. Since version 13.4,
`nwidart/laravel-modules` lets each module add its own indicators, and names each one
`module:{name}:{key}`. The name is the module name in lower case. So `module:billing:storage` is the
`storage` indicator of Billing.

The name is the whole contract. Mortise reads it to place each indicator in its module, without any
other configuration. An indicator whose name matches no module of the app stays with the app's own
components.

The package also adds a component named `modules`. It lists each module and gives an error for one that
is turned on but missing on disk. A module turned off registers no indicator.

## The worst state wins

A module can have several indicators. The index shows one mark per module, so Mortise needs one status.
It takes the worst, in the order Spring uses: Down, then Out of service, then Up, then Unknown. One Down
indicator makes the module Down, even if the others are Up. A module the `modules` component reports
missing is Down.

A module with no indicator of its own gets no mark and no runtime box. Mortise does not invent an Up for
it. "No indicator" and "healthy" are different facts.

On the Summary of All modules, the runtime box of the app names each module that is not Up. So a Down
app always says what is down.

Whether an environment has module indicators is read from what it answers, not from `composer.lock`.
Your local copy and staging may run different versions, and each is judged by its own answer.

## Metrics, and series without a module label

The Metrics tab reads the `prometheus` endpoint of the actuator, which `nwidart/laravel-metrics`
fills. It shows, per module: requests per minute, the time under which 95% of requests finish, the share
of errors, jobs per minute and the share of time spent in the database.

A metric series can carry a label named `module`. When it does, that label wins: the series belongs to
that module, whatever its case. The value `none`, or a name the app does not have, counts as
"Not in a module".

Most series today have no such label. Mortise then places them from facts it has:

- **A request** is placed by its route. Mortise matches the route template of the series with the
  routes the `mappings` endpoint lists, and each route belongs to the module of its controller. When the
  same path is in two modules, or the request had no route (a 404), it counts as "Not in a module".
- **A job** is placed by the namespace of its class.
- **Database time** has no route and no class to go by. It needs the `module` label, so its column reads
  "unknown" until the package adds that label.

Placing runs each time the figures are worked out, on the routes of now. A read from before a change of
routes is placed by the new routes. "Not in a module" is a real row: a closure route or a package route
belongs to no module, and the table says so rather than hiding it.

Two figures are less exact than they look:

- **95% under** is a bucket bound, such as "100 ms", not an exact time. The app records a histogram
  with buckets, not each request. The figure is the smallest bucket bound under which 95% of the
  requests fall.
- **Errors** is the share of requests answered with a 5xx status. A 4xx answer, such as a failed
  validation, does not count.

## Why reads stay in memory

Mortise reads `prometheus` every 15 seconds while the Metrics tab is open and the window is visible.
It keeps the reads in memory for the range you choose: 15 minutes, 1 hour or 24 hours. It never writes
them: not to disk, not to `mortise.json`, not to its settings.

There are three reasons.

- **Mortise is not a monitoring system.** It shows what the app does now, beside the code. Long-term
  history, limits and alerts are the job of other tools.
- **Nothing left behind.** Metrics on disk would grow, need a place, and outlive the session. Figures
  from a staging or production app would sit on a laptop. Kept in memory, they end when Mortise closes.
- **Bounded memory.** Each read holds running totals, so older reads can be thinned without losing a
  request. Mortise keeps every read of the last hour and one per 5 minutes before that. Memory stays flat
  after 24 hours.

The cost is that a restart starts from empty, and so does a switch to another environment: the figures
of two environments are not mixed. Switching the range keeps the reads.

Polling also stops when you are not looking. When the window is minimised or hidden, no frame is drawn,
and the worker that asks the app pauses until the window shows again.

[How to read metrics](/mortise/how-to/read-metrics/) and [How to connect the actuator](/mortise/how-to/connect-actuator/)
cover the setup.
