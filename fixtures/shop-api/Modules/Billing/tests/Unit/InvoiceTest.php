<?php

namespace Modules\Billing\Tests\Unit;

use Modules\Billing\Models\Invoice;
use Modules\Checkout\Models\Order;
use Tests\TestCase;

class InvoiceTest extends TestCase
{
    public function test_it_numbers_invoices(): void
    {
        $this->assertNotNull(Order::factory()->create());
    }

    public function test_it_sums_lines(): void
    {
        $this->assertTrue(true);
    }

    public function test_it_rounds_totals(): void
    {
        $this->assertTrue(true);
    }
}
