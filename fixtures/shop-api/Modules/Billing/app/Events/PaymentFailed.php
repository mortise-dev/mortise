<?php

namespace Modules\Billing\Events;

class PaymentFailed
{
    public function __construct(public readonly int $invoiceId)
    {
    }
}
