<?php

namespace App\Providers;

use Illuminate\Support\ServiceProvider;

class AppServiceProvider extends ServiceProvider
{
    /**
     * Adds the cache store `metrics_broken`: a database store on a SQLite file that does not exist.
     * With METRICS_CACHE_STORE=metrics_broken, a read of the metrics throws and `prometheus` answers 503.
     */
    public function register(): void
    {
        config([
            'database.connections.metrics_broken' => [
                'driver' => 'sqlite',
                'database' => storage_path('missing/metrics.sqlite'),
                'prefix' => '',
            ],
            'cache.stores.metrics_broken' => [
                'driver' => 'database',
                'connection' => 'metrics_broken',
                'table' => 'cache',
            ],
        ]);
    }
}
