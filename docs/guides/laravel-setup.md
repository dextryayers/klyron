# Laravel Setup

Klyron provides first-class support for Laravel projects with automatic Vite integration, Inertia scaffolding, and artisan command forwarding.

## Creating a Laravel Project

### Basic Laravel + Blade

```bash
klyron create laravel my-app
cd my-app
klyron dev
```

This scaffolds a standard Laravel application with Klyron's Vite integration for compiling Blade assets (CSS, JS).

### Laravel + Inertia + React

```bash
klyron create laravel-react my-app
cd my-app
klyron dev
```

This scaffolds a Laravel application with Inertia.js and React configured out of the box:

- `resources/js/` -- React components with TypeScript
- `routes/web.php` -- Inertia-compatible route definitions
- `app/Http/Controllers/` -- Example Inertia controllers
- Automatic HMR for both frontend (React) and backend (PHP)

### Laravel + Inertia + Vue

```bash
klyron create laravel-vue my-app
```

Same as above but with Vue 3 as the frontend framework.

## Inertia.js Integration

Klyron automatically detects Inertia projects and configures the dev server to:

1. Compile React/Vue components with HMR
2. Proxy API requests to the Laravel Artisan server
3. Handle SSR (Server-Side Rendering) for Inertia pages

The dev proxy is configured so that:

- Frontend assets (`/assets/*`) are served by Klyron's HMR server
- API requests (`/api/*`, `/login`, `/register`, etc.) are forwarded to Artisan on port 8000
- WebSocket connections for HMR are handled by Klyron

## Artisan Commands

Run Artisan commands directly through Klyron:

```bash
klyron run artisan.php migrate
klyron run artisan.php make:controller UserController
klyron run artisan.php make:model User -m
klyron run artisan.php queue:work
```

Klyron automatically sets environment variables like `APP_ENV`, `APP_DEBUG`, and `DB_*` from your `.env` file when running artisan commands.

## Blade Compilation

Klyron compiles Blade templates with Vite's asset pipeline:

```blade
{{-- resources/views/app.blade.php --}}
<!DOCTYPE html>
<html>
<head>
    @vite('resources/js/app.tsx')
</head>
<body>
    @inertia
</body>
</html>
```

The `@vite` directive resolves to the correct asset URL in both development (pointing to HMR server) and production (pointing to built files in `public/build/`).

## Configuration

Laravel-specific configuration in `klyron.json`:

```jsonc
{
  "laravel": {
    "artisan": "artisan",
    "publicDir": "public",
    "buildDir": "build",
    "hotReload": true,
    "ssr": {
      "enabled": true,
      "entry": "resources/js/ssr.tsx"
    },
    "server": {
      "port": 8000,
      "host": "127.0.0.1"
    }
  }
}
```

## Deployment

For production deployment of Laravel projects:

```bash
klyron build               # Compile frontend assets
klyron run artisan.php config:cache   # Cache configuration
klyron run artisan.php route:cache    # Cache routes
klyron run artisan.php view:cache     # Cache views
```

The built assets go to `public/build/` and are served by Laravel's built-in asset handling.
