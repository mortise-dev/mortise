<?php

use Illuminate\Support\Facades\Route;
use Modules\Billing\Http\Controllers\InvoiceController;

Route::get('billing/invoices/{invoice}', [InvoiceController::class, 'show']);
Route::get('billing/invoices/{invoice}/charge', [InvoiceController::class, 'charge']);
