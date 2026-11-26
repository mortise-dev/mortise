# Actuator answers of a real app

These files are answers of a running Laravel app, recorded on 23 November 2026.
Tests of Mortise read them as data from milestone 20 of phase 3.
Mortise does not install or run them, and the app itself is not in this repository.

## Versions

From the `composer.lock` of the app:

| Package | Version |
|---|---|
| `laravel/framework` | v13.43.0 |
| `nwidart/laravel-modules` | v13.4.0 |
| `nwidart/laravel-actuator` | 1.3.1 |
| `nwidart/laravel-metrics` | 1.1.0 |
| `nwidart/metrics-store` | 1.0.0 |
| `open-telemetry/sdk` | 1.17.0 |
| `wikimedia/composer-merge-plugin` | v2.1.0 |

PHP is 8.4.27 on macOS (arm64).
The app ran under `php artisan serve` (SAPI `cli-server`, one worker), on 127.0.0.1.

## How the app was built

```
composer create-project laravel/laravel app '^13.0'
composer config --no-plugins allow-plugins.wikimedia/composer-merge-plugin true
composer require nwidart/laravel-modules:13.4.0 nwidart/laravel-actuator:1.3.1 'nwidart/laravel-metrics:^1.0' -W
php artisan vendor:publish --provider="Nwidart\Modules\LaravelModulesServiceProvider"
php artisan vendor:publish --tag=actuator-config
php artisan vendor:publish --tag=metrics-config
```

`composer.json` got `"merge-plugin": {"include": ["Modules/*/composer.json"]}` under `extra`.
Then:

```
php artisan module:make Blog Billing
php artisan module:make Legacy
php artisan module:make-health-indicator StorageHealthIndicator Blog
php artisan module:make-health-indicator GatewayHealthIndicator Billing
php artisan module:make-health-indicator LegacyHealthIndicator Legacy
php artisan module:make-job PublishPost Blog
php artisan module:make-job ChargeInvoice Billing
php artisan module:make-command DigestCommand Blog
php artisan module:make-controller PostController Blog
php artisan module:make-controller InvoiceController Billing
php artisan module:disable Legacy
```

What the modules hold:

- Blog. Web routes `blog`, `blog/posts/{post}`, `blog/archive/{year?}`, `blog/posts/{post}/publish`
  (dispatches `Modules\Blog\Jobs\PublishPost`) and `blog/boom` (throws a `RuntimeException`, so 500).
  The api file of the module gives `api/v1/posts`.
  `$healthIndicators` has `'storage' => StorageHealthIndicator::class`; it answers Up with `drafts: 3`.
  `configureSchedules()` runs `blog:digest` every minute.
- Billing. Web routes `billing/invoices/{invoice}`, `billing/invoices/{invoice}/lines/{line?}`,
  `billing/invoices/{invoice}/charge` (dispatches `Modules\Billing\Jobs\ChargeInvoice`),
  `billing/fail` (`abort(503)`) and `portal` on the domain `{account}.billing.test`.
  `$healthIndicators` has `'gateway' => GatewayHealthIndicator::class`; it answers Up with
  `provider: fake` and `latencyMs: 12`.
  `configureSchedules()` runs the job `ChargeInvoice` every hour.
- Legacy. Turned off. It lists an indicator `legacy` and a route `legacy`. Neither is registered.

The app adds `/` (closure), `/hello/{name}` (closure) and `/status` (`App\Http\Controllers\StatusController`).
Each job and some controllers run one small query on the SQLite database.

`config/actuator.php` exposes `health`, `info`, `mappings`, `scheduledtasks` and `prometheus`.
It does not expose `env` or `modules`. `health.show_details` is `always`. No middleware guards the routes.

`.env` keeps the defaults of the skeleton (SQLite, `QUEUE_CONNECTION=database`, `CACHE_STORE=database`,
`SESSION_DRIVER=database`) and adds `APP_NAME=ShopFixture`, `METRICS_ENABLED=true`, `METRICS_STORE=cache`
and `METRICS_CACHE_STORE=file`. So the metrics live in one entry of the file cache store.

## The files

Every file was recorded with the header Mortise sends:

```
curl -sS -H 'Accept: application/json' -D <headers> -o <file> http://127.0.0.1:8931/actuator/<path>
```

| File | Path | Status | Content-Type | Before it |
|---|---|---|---|---|
| `prometheus-quiet.txt` | `prometheus` | 200 | `text/plain; version=0.0.4; charset=utf-8` | `php artisan actuator:metrics:clear`, then the server started. No request before it |
| `discovery.json` | (the base path) | 200 | `application/json` | The quiet scrape |
| `health.json` | `health` | 200 | `application/json` | The same |
| `info.json` | `info` | 200 | `application/json` | The same |
| `mappings.json` | `mappings` | 200 | `application/json` | The same |
| `scheduledtasks.json` | `scheduledtasks` | 200 | `application/json` | The same |
| `prometheus-busy.txt` | `prometheus` | 200 | `text/plain; version=0.0.4; charset=utf-8` | The traffic below |
| `prometheus-empty.txt` | `prometheus` | 200 | `text/plain; version=0.0.4; charset=utf-8` | `METRICS_ENABLED=false`, server restarted. The body is empty |
| `prometheus-503.txt` | `prometheus` | 503 | `text/plain; charset=utf-8` | `METRICS_ENABLED=true`, `METRICS_CACHE_STORE=metrics_broken`, server restarted, one request to `/blog` |

The traffic before `prometheus-busy.txt`, all `GET` with `Accept: application/json` unless said:

- `/blog` 20 times, `/blog/posts/1` to `/blog/posts/15`.
- `/blog/archive` and `/blog/archive/2021` to `/blog/archive/2025`, 5 times each.
- `/blog/boom` 3 times (500). `/api/v1/posts` 5 times.
- `/billing/invoices/1` to `/billing/invoices/10`.
- `/billing/invoices/{1,2,3}/lines` and `/billing/invoices/{1,2,3}/lines/2`.
- `/billing/fail` 3 times (503).
- `/portal` 3 times with the header `Host: acme.billing.test`.
- `/` 5 times, `/hello/x1` to `/hello/x5`, `/status` 3 times.
- `/nope/1` to `/nope/3` (404). `POST /blog` 3 times (405).
- `/blog/posts/{1..4}/publish` and `/billing/invoices/{1..4}/charge`: 8 queued jobs.
- `/up` once.
- `php artisan queue:work --stop-when-empty` ran the 8 jobs.
- `php artisan schedule:run` ran `blog:digest` once.

The store `metrics_broken` of `prometheus-503.txt` is a `database` cache store on a SQLite connection
whose file does not exist. Reading it throws, and the endpoint answers 503.

## What was changed after the recording

- The origin `http://127.0.0.1:8931` is now `http://app.test`, in `discovery.json`.
- The folder of the app is now `/srv/app`: in the `diskSpace` path of `health.json`, and in the
  `db_namespace` label of `prometheus-busy.txt` (`/srv/app/database/database.sqlite`).
  The real folder was 159 characters long. `prometheus-busy.txt` had 130,922 bytes before; it has 121,862 now.
- Nothing else was changed. The JSON keeps the escaped slashes (`\/`) the app wrote.
- The app has no secret in these answers. `env` was not exposed and not asked for.

## The tagged scheduled tasks

`scheduledtasks-tagged.json` was recorded on 26 November 2026 from another app, to show the `module`
tag of PKG-2. Mortise tests read it in place of `scheduledtasks.json`.

| Package | Version |
|---|---|
| `laravel/framework` | v13.44.1 |
| `nwidart/laravel-modules` | branch `feat/actuator-task-module` of PR #2311, not released (installed as 13.5.99 from a path repository) |
| `nwidart/laravel-actuator` | branch `feat/scheduled-task-tags` of PR #40, not released (installed as 1.4.99 from a path repository) |

PHP is 8.4.27 on macOS (arm64). The app was made as above with `module:make Blog Billing`,
`module:make-command DigestCommand Blog` (signature `blog:digest`, in `$commands` of the Blog provider)
and `module:make-job ChargeInvoice Billing`. `scheduledtasks` was in `exposure.include`, and
`METRICS_ENABLED=false`.

- Blog `configureSchedules()`: `command('blog:digest')->everyMinute()`, `call(fn () => null)->hourly()`,
  `exec('echo sitemap')->daily()` and `call(fn () => null)->everyTenSeconds()->description('Refresh drafts')`.
- Billing `configureSchedules()`: `job(ChargeInvoice::class)->hourly()`.
- `routes/console.php`: `Schedule::call(fn () => null)->daily()->description('Prune app cache')` and
  `Schedule::command('blog:digest')->dailyAt('06:00')`. These two have no tag.

```
curl -sS -H 'Accept: application/json' -o scheduledtasks-tagged.json http://127.0.0.1:8941/actuator/scheduledtasks
```

Nothing was changed after the recording.
