<?php

namespace Modules\Checkout\Listeners;

use Modules\Checkout\Events\OrderPaid;
use Modules\Shipping\Contracts\ShipmentRequester;

class RequestShipment
{
    public function __construct(private ShipmentRequester $shipments)
    {
    }

    public function handle(OrderPaid $event): void
    {
        $this->shipments->request($event->orderId);
    }
}
