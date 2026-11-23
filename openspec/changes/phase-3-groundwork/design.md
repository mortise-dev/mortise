# Design

## Context

See `proposal.md` for the reasons. The actuator client is `runtime/actuator.rs` (one request per
endpoint, JSON only) and `runtime/live.rs` (the worker thread and its paces). `testing.rs` holds
`StubActuator`, a local server that answers what a test tells it to. No OpenSpec capability covered the
client before this change. The plan (`docs/plan/phase-3.md`, section 1) marks two facts as read in the
package docs and not seen on a running app: the form of `http_route` against the `uri` of `mappings`,
and what `prometheus` answers.

## Goals / Non-Goals

**Goals:**

- The rule of decision 1 reads the same in the four places that state it.
- The capability `actuator-client` describes the client as built. Each scenario matches code that
  exists today, except the one path `prometheus` that the rule allows and milestone 20 asks for.
- Real answers settle the placing questions before milestone 20 designs `metrics/place.rs`.

**Non-Goals:**

- No code asks for `prometheus`. No parser, no test reads `fixtures/actuator/` yet.
- No change to what the client does.
- No release (decision 11).

## Decisions

- **One capability for the client.** `actuator-client` holds the paths, the redirect and status rules,
  the credentials, the timeout and the paces. Alternative: add the rule to `app-access`. Rejected:
  `app-access` is about the files of an app and the commands Mortise runs in it, not about HTTP.
- **The rule names `prometheus` now.** Milestones 19, 20 and 22 start from it. The rule is a limit, not
  a list of what the code does today, so naming a path no code asks for yet breaks nothing.
- **Recorded answers, not hand-written ones.** The answers come from a real app built outside the
  repository (`fixtures/actuator/README.md` gives each command). Only the answers enter the repository.
  Secrets, host names and local paths are scrubbed. Alternative: answers written from the package
  docs. Rejected: the docs are what section 1 of the plan could not confirm.
- **Text moves, it does not change.** The S7.4 block of the Metrics tab moves to `app/strings.rs`. The
  text of the Settings view, the table of shortcuts with it, moves to `settings/strings.rs`. The tab
  labels and "Open in editor" move to `mortise-app/src/strings.rs`, as more than one feature shows them.
  Key names on key caps stay where they are drawn. One sentence of S7.4 said the actuator has no
  metrics endpoint. `nwidart/laravel-actuator` has `prometheus` since 1.2.0, so that sentence is
  corrected and marked "not in the spec". Its image is made again.
- **Spec 12.2.** Edited in `docs/spec-build`, then `docs/spec` is rebuilt with its script. The HTML is
  never edited by hand. The script wrote to a folder outside the repository; it now writes to
  `docs/spec` beside it.
- **Line endings.** `fixtures/actuator/` is marked `-text` in `.gitattributes`, so a checkout on
  Windows keeps the bytes the app wrote.

## What the recorded answers show

The answers are in `fixtures/actuator/`, recorded from one app on macOS: laravel/framework 13.43.0,
laravel-modules 13.4.0, laravel-actuator 1.3.1, laravel-metrics 1.1.0, metrics-store 1.0.0, PHP 8.4.27
under `php artisan serve`, metrics in the file cache store. Modules Blog and Billing are on and have one
indicator each; Legacy is off. Each finding below was checked on the recorded files.

**Routes.** `prometheus-busy.txt` has 15 values of `http_route`. 14 equal `/` followed by the `uri` of
the matching route in `mappings.json`. That includes a prefix (`/api/v1/posts`) and optional parameters
(`/blog/archive/{year?}`, `/billing/invoices/{invoice}/lines/{line?}`); a request with and without the
optional part falls in the same series. The exceptions:

- The root route. `mappings` writes `"uri":"/"` and the metric writes `http_route="/"`. The rule that
  holds for all 15 is: `http_route` equals `/` followed by the `uri` with its leading slashes removed.
- A domain route. `portal` on `{account}.billing.test` is `http_route="/portal"`; the domain is dropped.
  Two routes with the same path on two domains share one series. `mappings` gives the domain, so
  milestone 20 can see the clash. Such a series counts as "Not in a module" when the two routes belong
  to different modules.
- A request that matches no route (404, 405) has no `http_route` label at all. It counts as "Not in a
  module".
- The `action` of the domain route starts with a backslash
  (`\Modules\Billing\Http\Controllers\InvoiceController@portal`); the others do not. `module_of`
  already trims it.

Requests to the actuator itself are not counted, so Mortise's own reads add no request series.

**Jobs.** The label `laravel_job_name` holds the full class name, escaped as the text format requires:
`laravel_job_name="Modules\\Blog\\Jobs\\PublishPost"`. Unescaped, it starts with the namespace of the
module (`Modules\Blog\`, from the module's `composer.json`), so `module_of` places it as it places a
route action. The label is on `messaging_process_duration_seconds`,
`messaging_client_sent_messages_total` and `laravel_db_queries`.

**No `module` label.** No series carries a `module` label. This confirms A2 in part and decision 2:
Mortise places series itself. `db_client_operation_duration_seconds` has `db_namespace`,
`db_operation_name` and `db_system_name` only: no route and no job. Database time per module cannot be
worked out from these answers, which confirms decision 3 ("unknown" until PKG-1). `laravel_db_queries`
counts queries per route or job, but not their time.

**The `Accept` header.** `prometheus` answers 200 with `text/plain; version=0.0.4; charset=utf-8` to
`Accept: application/json`, to `text/plain`, to `application/openmetrics-text` and with no `Accept`
header. The bodies differ only in values that change between reads. The client can keep its header.
The 503 answer is `text/plain; charset=utf-8` with one line, `The metrics store cannot be read.`
With metrics off the answer is 200 with an empty body.

**Size.** Quiet: 1,975 bytes, 25 lines, gauges only. Busy: 121,862 bytes, 697 lines, 16 families.
One route with one status takes about 31 lines, about 5.5 KB: 17 lines of the request histogram
(15 buckets, sum, count) and 14 of `laravel_db_queries`. Each further status of the route adds about
4 KB. One job takes about 6 KB. 60 modules with 10 routes each give 3.3 MB or more. Milestone 20 sets
the body limit with this figure.

**Other facts.**

- Bucket bounds of request and job time: 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1, 2.5,
  5, 7.5, 10, +Inf (seconds). The 95% figure of decision 5 reads one of these.
- A 5xx request carries `error_type` beside `http_response_status_code`.
- Each read of `prometheus` adds two `SELECT` operations to `db_client_operation_duration_seconds`
  with the database cache store. A read every 15 s shows up in the app's database figures.
- `health` names the module indicators `module:billing:gateway` and `module:blog:storage`. The
  `modules` component lists each module with `enabled`; the turned-off module Legacy registers no
  indicator and no route.
- `scheduledtasks` gives a command as `php artisan blog:digest` and a job as its class name.
- Values that reach the store after the response were not seen: `php artisan serve` runs one worker.
  PHP-FPM with several workers was not tried.

## Risks / Trade-offs

- [The packages move fast and the fixtures go stale] → The README gives the versions and the commands,
  so the answers can be made again. Milestone 23 adds a CI job that checks a live actuator.
- [The answers come from one app on one computer, with a cache store] → They show the formats. They do
  not show how two servers behind a load balancer add up. The plan leaves that to milestone 20.
