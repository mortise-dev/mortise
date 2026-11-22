<?php

namespace Modules\Checkout\Services;

use Modules\Shipping\Contracts\RateCalculator;

class ShippingQuote
{
    public function __construct(private RateCalculator $rates)
    {
    }
}
