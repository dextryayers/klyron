# `klyron dev`

Start the development server with Hot Module Replacement (HMR), auto-reload, and framework auto-detection.

## Usage

```bash
klyron dev [options]
```

## Description

The `dev` command starts an HTTP development server that watches your source files and applies changes instantly via HMR. Klyron automatically detects your project's framework by inspecting the project structure, `package.json`, and configuration files.

## Options

| Option | Description |
|---|---|
| `--port <port>` | Port to listen on (default: 5173) |
| `--host <host>` | Host address (default: localhost) |
| `--hot` | Enable/disable HMR (default: true) |
| `--https` | Enable HTTPS with auto-generated cert |
| `--open` | Open browser on startup |
| `--base <path>` | Base URL path (default: /) |
| `--proxy <config>` | Proxy configuration as JSON string |
| `--no-cors` | Disable CORS headers |
| `--entry <file>` | Entry point (auto-detected if omitted) |

## Auto-detected Frameworks

Klyron detects and configures the dev server for these frameworks:

| Framework | Detection |
|---|---|
| React | `vite.config.ts` / `react` in dependencies |
| Vue | `vue` in dependencies |
| Svelte | `svelte` in dependencies |
| Laravel | `artisan` file in root |
| Laravel + Inertia | `laravel` + `inertiajs` in composer.json |
| Astro | `astro.config.*` |
| Solid | `solid-js` in dependencies |
| Lit | `lit` in dependencies |

## Examples

### Start dev server on default port

```bash
klyron dev
```

### Custom port and host

```bash
klyron dev --port 3000 --host 0.0.0.0
```

### Enable HTTPS

```bash
klyron dev --https
```

### Proxy API requests

```bash
klyron dev --proxy '{"api":{"target":"http://localhost:8080","rewrite":"^/api"}}'
```

## HMR Details

Klyron's HMR works by establishing a WebSocket connection between the dev server and the browser. When a source file changes:

1. The dev server re-compiles only the changed module
2. A hot update payload (JSON + JS patch) is sent via WebSocket
3. The runtime module system applies the update without a full reload
4. Framework-specific HMR handlers (React Fast Refresh, Vue's HMR, etc.) preserve component state

## Laravel Dev Server

When Klyron detects a Laravel project, it runs both the Vite dev server (for frontend assets) and the Artisan development server (for PHP backend). The proxy configuration is set up automatically so that API requests are forwarded to Artisan.
