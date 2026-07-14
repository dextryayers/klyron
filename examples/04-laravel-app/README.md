# 04 — Laravel with Klyron

A Laravel application example that uses Klyron's PHP engine to serve routes and Blade templates. Klyron's polyglot runtime allows running PHP applications alongside other languages.

## Files

| File                                    | Description                     |
|-----------------------------------------|---------------------------------|
| `artisan`                               | Laravel artisan wrapper script  |
| `composer.json`                         | Composer project metadata       |
| `app/Http/Controllers/HelloController.php` | Sample controller           |
| `routes/web.php`                        | Web route definitions           |
| `resources/views/hello.blade.php`       | Blade template                  |

## Run

```bash
klyron run artisan serve
```

Then visit `http://localhost:8000`.

## Notes

- This is a minimal Laravel-style setup; a full Laravel app would include the framework bootstrap, config, and vendor directory.
- Klyron executes PHP files directly through its built-in PHP engine.
