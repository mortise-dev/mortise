<?php

namespace Modules\Checkout\Contracts;

interface OrderLookup
{
    public function find(int $id): ?array;
}
