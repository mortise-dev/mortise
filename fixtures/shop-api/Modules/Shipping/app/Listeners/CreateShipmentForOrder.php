<?php

namespace Modules\Shipping\Listeners;

use Modules\Checkout\Events\OrderPaid;

class CreateShipmentForOrder
{
    public function handle(OrderPaid $event): void
    {
    }
}
