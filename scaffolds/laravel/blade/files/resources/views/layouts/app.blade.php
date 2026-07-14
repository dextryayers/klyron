<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>@yield('title', config('app.name'))</title>
    @vite(['resources/css/app.css', 'resources/js/app.js'])
</head>
<body>
    <header class="border-b border-gray-200 bg-white">
        <div class="mx-auto max-w-7xl px-4 py-4">
            <a href="/" class="text-xl font-bold">{{ name }}</a>
        </div>
    </header>
    <main>
        @yield('content')
    </main>
</body>
</html>
