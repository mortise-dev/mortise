<?php

namespace Modules\Notifications\Listeners;

use Modules\Checkout\Events\OrderPaid;

class NotifyOrderPaid
{
    public function handle(OrderPaid $event): void
    {
    }
}
