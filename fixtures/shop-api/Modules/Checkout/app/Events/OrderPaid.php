<?php

namespace Modules\Checkout\Events;

class OrderPaid
{
    public function __construct(public readonly int $orderId)
    {
    }
}
