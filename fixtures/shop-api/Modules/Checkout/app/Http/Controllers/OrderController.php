<?php

namespace Modules\Checkout\Http\Controllers;

use Modules\Billing\Models\Invoice;
use Modules\Catalog\Contracts\ProductLookup;

class OrderController
{
    public function show(int $order, ProductLookup $products): array
    {
        return [
            'invoice' => Invoice::query()->where('number', 'ORD-'.$order)->first(),
            'product' => $products->find($order),
        ];
    }
}
