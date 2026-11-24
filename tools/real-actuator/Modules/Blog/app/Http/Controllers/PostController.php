<?php

namespace Modules\Blog\Http\Controllers;

use App\Http\Controllers\Controller;
use Modules\Blog\Jobs\PublishPost;
use RuntimeException;

class PostController extends Controller
{
    public function show(int $post): array
    {
        return ['post' => $post];
    }

    public function publish(int $post): array
    {
        PublishPost::dispatch($post);

        return ['queued' => $post];
    }

    public function boom(): never
    {
        throw new RuntimeException('Boom.');
    }
}
