<?php

namespace Modules\Shipping\Services;

use Modules\Checkout\Contracts\OrderLookup;

class LabelPrinter
{
    public function __construct(private OrderLookup $orders)
    {
    }
}
