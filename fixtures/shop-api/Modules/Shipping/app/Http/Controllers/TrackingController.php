<?php

namespace Modules\Shipping\Http\Controllers;

use Modules\Checkout\Contracts\OrderLookup;

class TrackingController
{
    public function show(int $order, OrderLookup $orders): ?array
    {
        return $orders->find($order);
    }
}
