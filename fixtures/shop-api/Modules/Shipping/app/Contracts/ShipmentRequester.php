<?php

namespace Modules\Shipping\Contracts;

interface ShipmentRequester
{
    public function request(int $orderId): void;
}
