<?php

use Illuminate\Support\Facades\Route;

// A route of the app, in no module.
Route::get('/hello/{name}', fn (string $name) => ['hello' => $name]);
