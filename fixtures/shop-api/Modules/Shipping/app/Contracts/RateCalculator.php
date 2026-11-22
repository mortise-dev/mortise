<?php

namespace Modules\Shipping\Contracts;

interface RateCalculator
{
    public function rate(int $weight): int;
}
