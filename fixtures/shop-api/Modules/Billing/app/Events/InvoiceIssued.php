<?php

namespace Modules\Billing\Events;

class InvoiceIssued
{
    public function __construct(public readonly int $invoiceId)
    {
    }
}
