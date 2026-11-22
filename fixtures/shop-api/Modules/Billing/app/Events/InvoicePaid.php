<?php

namespace Modules\Billing\Events;

class InvoicePaid
{
    public function __construct(public readonly int $invoiceId)
    {
    }
}
