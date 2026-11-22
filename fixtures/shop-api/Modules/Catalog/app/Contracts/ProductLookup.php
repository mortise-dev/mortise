<?php

namespace Modules\Catalog\Contracts;

interface ProductLookup
{
    public function find(int $id): ?array;
}
