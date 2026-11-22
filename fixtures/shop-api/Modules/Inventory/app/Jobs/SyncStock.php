<?php

namespace Modules\Inventory\Jobs;

use Modules\Catalog\Contracts\ProductLookup;

class SyncStock
{
    public function handle(ProductLookup $products): void
    {
        $products->find(1);
    }
}
