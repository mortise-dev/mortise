<?php

namespace Modules\Blog\Jobs;

use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;

class PublishPost implements ShouldQueue
{
    use Queueable;

    public function __construct(public int $post) {}

    public function handle(): void {}
}
