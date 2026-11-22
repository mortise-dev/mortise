<?php

namespace Modules\Checkout\Models;

use Illuminate\Database\Eloquent\Model;
use Modules\Users\Contracts\UserLookup;

class Order extends Model
{
    public function buyer(UserLookup $users): ?array
    {
        return $users->find($this->user_id);
    }
}
