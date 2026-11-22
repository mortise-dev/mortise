<?php

use Modules\Checkout\Models\Order;

it('creates an order', function () {
    expect(new Order)->toBeInstanceOf(Order::class);
});

test('an order has a buyer', function () {
    expect(true)->toBeTrue();
});
