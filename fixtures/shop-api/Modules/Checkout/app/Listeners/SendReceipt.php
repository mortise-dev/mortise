<?php

namespace Modules\Checkout\Listeners;

use Modules\Billing\Events\InvoicePaid;

class SendReceipt
{
    public function handle(InvoicePaid $event): void
    {
    }
}
