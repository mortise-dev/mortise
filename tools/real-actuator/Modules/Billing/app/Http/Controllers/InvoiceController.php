<?php

namespace Modules\Billing\Http\Controllers;

use App\Http\Controllers\Controller;
use Modules\Billing\Jobs\ChargeInvoice;

class InvoiceController extends Controller
{
    public function show(int $invoice): array
    {
        return ['invoice' => $invoice];
    }

    public function charge(int $invoice): array
    {
        ChargeInvoice::dispatch($invoice);

        return ['queued' => $invoice];
    }
}
