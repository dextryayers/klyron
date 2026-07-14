# Klyron Laravel 11.x Stubs

Compatibility stubs for running Laravel 11.x applications with Klyron.

## Requirements

- PHP 8.2+
- Laravel Framework 11.x
- Klyron CLI

## Usage

```bash
klyron artisan serve
klyron artisan migrate
```

## What's New in v11

- Slimmed application skeleton
- No more `Http/Kernel.php` (bootstrap/app.php configuration)
- SQLite by default
- Health routing endpoint
- Graceful encryption key rotation
- `once()` helper
- Improved `when()` and `unless()` methods

## Klyron Integration

- Streamlined Artisan command execution
- Automatic `.env` loading
- Built-in dev server with hot reload
- Database migrations via `klyron artisan migrate`
