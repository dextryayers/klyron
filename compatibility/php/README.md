# PHP Interoperability Guide

This guide covers running PHP alongside Klyron, calling PHP scripts from Klyron, and sharing data between the two runtimes.

## Running PHP Scripts

### From Klyron (TypeScript)
```typescript
import { exec } from './process';

const result = await exec('php', ['-r', 'echo json_encode(["hello" => "world"]);']);
console.log(result.stdout); // {"hello":"world"}
```

### From Klyron (CLI)
```bash
klyron exec php artisan migrate
```

## Data Exchange

### JSON-based IPC
PHP and Klyron communicate best through JSON:
```php
<?php
$data = json_decode(file_get_contents('php://stdin'), true);
echo json_encode(['result' => $data['value'] * 2]);
```

```typescript
const result = await execShell(`echo '${JSON.stringify({ value: 21 })}' | php script.php`);
const doubled = JSON.parse(result.stdout);
```

## Laravel Integration

- Run `php artisan` commands via `klyron exec`
- Use Klyron's built-in HTTP server to serve Laravel with `php artisan serve`
- Build frontend assets with Klyron's Vite integration
- Klyron can read `.env` files and `config/*.php` for environment configuration

## Symfony Integration

- Symfony Console commands execute via `klyron exec php bin/console <command>`
- Klyron's caching layer can replace Symfony's file-based cache
- Environment variable mapping between `.env` and Klyron's config

## Shared Environment

- Environment variables set in Klyron are passed to PHP processes
- File system operations are shared (both runtimes access the same filesystem)
- For long-running PHP processes, use Klyron's process management
