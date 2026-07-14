<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;

class HelloController extends Controller
{
    public function __invoke(Request $request)
    {
        $name = $request->query('name', 'Klyron');

        return view('hello', [
            'name' => $name,
            'runtime' => 'Klyron',
            'time' => now()->toDateTimeString(),
        ]);
    }
}
