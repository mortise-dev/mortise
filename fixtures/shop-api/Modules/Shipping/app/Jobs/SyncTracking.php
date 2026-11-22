<?php

namespace Modules\Shipping\Jobs;

use Modules\Checkout\Events\OrderShipped;

class SyncTracking
{
    public function handle(): void
    {
        event(new OrderShipped(1));
    }
}
