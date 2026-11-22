<?php

namespace Modules\Checkout\Services;

use Modules\Billing\Contracts\PaymentGateway;

class PaymentService
{
    public function __construct(private PaymentGateway $gateway)
    {
    }

    public function pay(int $amount): bool
    {
        return $this->gateway->charge($amount);
    }
}
