<?php

namespace Modules\Billing\Contracts;

interface PaymentGateway
{
    public function charge(int $amount): bool;
}
