<?php

namespace Modules\Notifications\Listeners;

use Modules\Billing\Events\InvoicePaid;

class NotifyInvoicePaid
{
    public function handle(InvoicePaid $event): void
    {
    }
}
