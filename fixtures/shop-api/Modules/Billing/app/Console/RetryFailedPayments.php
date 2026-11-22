<?php

namespace Modules\Billing\Console;

use Illuminate\Console\Command;

class RetryFailedPayments extends Command
{
    protected $signature = 'billing:retry-failed-payments';

    public function handle(): int
    {
        return self::SUCCESS;
    }
}
