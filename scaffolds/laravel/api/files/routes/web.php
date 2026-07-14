<?php use Illuminate\Support\Facades\Route; Route::get('/api/health', fn() => response()->json(['status' => 'ok', 'service' => '{{ name }}', 'version' => '{{ version }}']));
