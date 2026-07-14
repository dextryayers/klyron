<?php

return [
    'default' => env('CACHE_STORE', 'file'),
    'stores' => [
        'apc' => ['driver' => 'apc'],
        'array' => ['driver' => 'array', 'serialize' => false],
        'file' => [
            'driver' => 'file',
            'path' => storage_path('framework/cache/data'),
        ],
        'memcached' => [
            'driver' => 'memcached',
            'persistent_id' => env('MEMCACHED_PERSISTENT_ID'),
            'sasl' => [env('MEMCACHED_USERNAME'), env('MEMCACHED_PASSWORD')],
            'options' => [],
            'servers' => [
                ['host' => env('MEMCACHED_HOST', '127.0.0.1'), 'port' => env('MEMCACHED_PORT', 11211), 'weight' => 100],
            ],
        ],
        'redis' => [
            'driver' => 'redis',
            'connection' => 'cache',
            'lock_connection' => 'default',
        ],
        'database' => [
            'driver' => 'database',
            'table' => env('DB_CACHE_TABLE', 'cache'),
            'connection' => null,
            'lock_connection' => null,
        ],
    ],
    'prefix' => env('CACHE_PREFIX', Str::slug(env('APP_NAME', '{{ name }}'), '_').'_cache_'),
];
