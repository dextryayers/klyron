# `klyron run`

Execute JavaScript, TypeScript, PHP, or any supported language file directly from the command line.

## Usage

```bash
klyron run [options] <file> [-- <args>...]
```

## Description

The `run` command executes a single file using the configured or auto-detected engine. Klyron determines the engine based on the file extension: `.js`/`.ts` use V8 (or Boa/QuickJS), `.php` uses the embedded PHP engine via the Laravel adapter layer.

## Options

| Option | Description |
|---|---|
| `--engine <engine>` | Force a specific engine: `v8`, `boa`, `quickjs`, `jsc` |
| `--watch` | Watch mode: re-run the file on changes |
| `--allow-net` | Grant network access to the script |
| `--allow-read` | Grant filesystem read access |
| `--allow-write` | Grant filesystem write access |
| `--allow-env` | Grant environment variable access |
| `--allow-all` | Grant all permissions (use with caution) |
| `--eval <code>` | Evaluate inline code instead of a file |
| `--import-map <path>` | Use an import map for module resolution |

## Examples

### Run a TypeScript file

```bash
klyron run hello.ts
```

### Run with permissions

```bash
klyron run --allow-net --allow-read server.ts
```

### Run PHP

```bash
klyron run artisan.php migrate
```

### Watch mode

```bash
klyron run --watch src/main.ts
```

### Use a specific engine

```bash
klyron run --engine quickjs script.js
```

### Evaluate inline code

```bash
klyron run --eval 'console.log("hello from klyron")'
```

### Pass arguments to the script

```bash
klyron run cli.ts -- --port 8080 --verbose
```

## Permissions Model

By default, scripts run in a sandbox with no I/O permissions. You must explicitly grant access using the `--allow-*` flags. This follows Deno's permission model and prevents untrusted scripts from accessing sensitive resources.

## Engine Selection

Klyron supports four JavaScript engines:

- **V8** (default) -- Full Node.js/Deno compatibility, fastest execution
- **Boa** -- Rust-native engine, ideal for embedding and custom environments
- **QuickJS** -- Small footprint, fast startup, good for CLI tools
- **JSC** -- JavaScriptCore, Safari-compatible

Use `--engine` to override the default for a specific run.
