<?php

namespace Modules\Checkout\Listeners;

use Modules\Billing\Events\InvoicePaid;

class MarkOrderPaid
{
    public function handle(InvoicePaid $event): void
    {
    }
}
