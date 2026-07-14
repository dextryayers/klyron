<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{{ name }}</title>
    @vite(['resources/css/app.css', 'resources/js/app.js'])
</head>
<body class="antialiased">
    <div class="flex min-h-screen flex-col items-center justify-center bg-gray-100">
        <div class="text-center">
            <h1 class="text-4xl font-bold text-gray-900 sm:text-6xl">{{ name }}</h1>
            <p class="mt-4 text-lg text-gray-600">{{ description }}</p>
            <div class="mt-6" x-data="{ count: 0 }">
                <button
                    class="rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500"
                    x-on:click="count++"
                >
                    count is <span x-text="count"></span>
                </button>
            </div>
        </div>
    </div>
</body>
</html>
