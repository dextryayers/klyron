<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Klyron + Laravel</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css">
</head>
<body>
    <main class="container">
        <h1>Hello, {{ $name }}!</h1>
        <p>This page is served by <strong>{{ $runtime }}</strong> using <strong>Laravel</strong> and <strong>Blade</strong>.</p>
        <p>Server time: {{ $time }}</p>
    </main>
</body>
</html>
