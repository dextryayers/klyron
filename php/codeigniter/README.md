# Klyron CodeIgniter Compatibility

This guide covers running CodeIgniter applications with Klyron.

## Requirements

- PHP 8.1+
- CodeIgniter 4.x
- Klyron CLI

## Getting Started

```bash
# Create a new CodeIgniter project
klyron create my-app --template codeigniter

# Serve the application
klyron serve

# Run migrations
klyron exec php spark migrate
```

## Supported Features

- CodeIgniter's URL routing
- Database query builder
- Migrations and seeds
- Validation library
- Session handling
- Email library
- File uploads
- Caching (file-based)
- CLI via Spark

## Klyron Integration

- Serve with `klyron serve` instead of `php spark serve`
- Environment variables from `.env` are loaded automatically
- Klyron's file watcher provides hot reload during development
- Deploy with `klyron deploy`

## Known Limitations

- CodeIgniter's `composer.json` autoloading works as expected
- Some third-party packages may need PHP extensions
- Use Klyron's process manager for queue workers
