<?php

use App\Http\Controllers\HelloController;
use Illuminate\Support\Facades\Route;

Route::get('/', HelloController::class);

Route::get('/health', function () {
    return response()->json([
        'status' => 'ok',
        'runtime' => 'Klyron PHP',
    ]);
});
