<?php

namespace Modules\Billing\Health;

use Nwidart\Actuator\Health\Health;
use Nwidart\Actuator\Health\HealthIndicator;

class GatewayHealthIndicator implements HealthIndicator
{
    public function health(): Health
    {
        return Health::up()->withDetail('provider', 'fake');
    }
}
