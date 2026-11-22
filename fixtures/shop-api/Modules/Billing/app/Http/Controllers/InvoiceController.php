<?php

namespace Modules\Billing\Http\Controllers;

use Modules\Billing\Models\Invoice;

class InvoiceController
{
    public function show(Invoice $invoice): Invoice
    {
        return $invoice;
    }
}
