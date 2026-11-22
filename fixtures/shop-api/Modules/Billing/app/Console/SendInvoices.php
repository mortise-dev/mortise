<?php

namespace Modules\Billing\Console;

use Illuminate\Console\Attributes\Signature;
use Illuminate\Console\Command;

#[Signature('billing:send-invoices')]
class SendInvoices extends Command
{
    public function handle(): int
    {
        return self::SUCCESS;
    }
}
