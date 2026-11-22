<?php

namespace Modules\Billing\Contracts;

interface InvoiceNumbering
{
    public function next(): string;
}
