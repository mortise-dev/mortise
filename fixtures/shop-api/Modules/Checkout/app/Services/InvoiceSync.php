<?php

namespace Modules\Checkout\Services;

use Modules\Billing\Models\Invoice;

class InvoiceSync
{
    public function sync(int $orderId): Invoice
    {
        return Invoice::firstOrCreate(['number' => 'ORD-'.$orderId]);
    }
}
