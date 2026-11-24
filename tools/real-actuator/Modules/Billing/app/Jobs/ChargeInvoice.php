<?php

namespace Modules\Billing\Jobs;

use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;

class ChargeInvoice implements ShouldQueue
{
    use Queueable;

    public function __construct(public int $invoice) {}

    public function handle(): void {}
}
