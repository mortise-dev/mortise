<?php

namespace Modules\Users\Contracts;

interface UserLookup
{
    public function find(int $id): ?array;
}
