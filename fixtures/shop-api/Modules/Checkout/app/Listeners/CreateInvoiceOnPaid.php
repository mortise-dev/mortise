<?php

namespace Modules\Checkout\Listeners;

use Modules\Billing\Models\Invoice;
use Modules\Checkout\Events\OrderPaid;

class CreateInvoiceOnPaid
{
    public function handle(OrderPaid $event): void
    {
        Invoice::create(['number' => 'ORD-'.$event->orderId, 'total' => 0]);
    }
}
