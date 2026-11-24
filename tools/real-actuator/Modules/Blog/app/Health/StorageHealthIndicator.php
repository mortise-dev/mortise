<?php

namespace Modules\Blog\Health;

use Nwidart\Actuator\Health\Health;
use Nwidart\Actuator\Health\HealthIndicator;

class StorageHealthIndicator implements HealthIndicator
{
    public function health(): Health
    {
        return Health::up()->withDetail('drafts', 3);
    }
}
