<?php

namespace Modules\Checkout\Events;

class OrderShipped
{
    public function __construct(public readonly int $orderId)
    {
    }
}
