<?php

use Illuminate\Support\Facades\Route;
use Modules\Blog\Http\Controllers\PostController;

Route::get('blog/posts/{post}', [PostController::class, 'show']);
Route::get('blog/posts/{post}/publish', [PostController::class, 'publish']);
Route::get('blog/boom', [PostController::class, 'boom']);
