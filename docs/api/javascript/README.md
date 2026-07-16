# JavaScript API Reference

Klyron exposes a set of built-in modules and globals available at runtime.

## Built-in Modules

### `klyron:test`

Test framework module with `test`, `expect`, and `describe` functions.

```typescript
import { test, expect, describe } from "klyron:test";

describe("Math", () => {
  test("addition", () => {
    expect(1 + 1).toBe(2);
  });
});
```

### `klyron:fs`

Filesystem operations (subject to `--allow-read`/`--allow-write` permissions).

```typescript
import { readTextFile, writeTextFile } from "klyron:fs";

const content = await readTextFile("/path/to/file.txt");
await writeTextFile("/path/to/output.txt", "hello");
```

### `klyron:env`

Environment variable access (subject to `--allow-env` permission).

```typescript
import { getEnv, setEnv } from "klyron:env";

const mode = getEnv("NODE_ENV");
setEnv("MY_VAR", "value");
```

### `klyron:http`

HTTP client and server utilities.

```typescript
import { serve } from "klyron:http";

serve((req) => {
  return new Response("Hello from Klyron!", {
    headers: { "content-type": "text/plain" },
  });
});
```

### `klyron:permissions`

Query and request permissions at runtime.

```typescript
import { requestPermission, Permission } from "klyron:permissions";

const granted = await requestPermission(Permission.Net, "api.example.com");
if (granted) {
  // Make network request
}
```

## Globals

### `console`

Standard console API:

```typescript
console.log("info");
console.error("error");
console.warn("warning");
console.table([{ a: 1, b: 2 }]);
console.time("timer");
console.timeEnd("timer");
```

### `fetch`

WHATWG Fetch API:

```typescript
const response = await fetch("https://api.example.com/data");
const data = await response.json();
```

### Timers

```typescript
setTimeout(() => {}, 1000);
setInterval(() => {}, 100);
setImmediate(() => {});
clearTimeout(id);
clearInterval(id);
clearImmediate(id);
```

### `URL` and `URLSearchParams`

```typescript
const url = new URL("https://example.com/path?q=search");
const params = new URLSearchParams(url.search);
```

### `WebSocket`

```typescript
const ws = new WebSocket("ws://localhost:8080");
ws.onmessage = (event) => console.log(event.data);
```

## Runtime Configuration

### `Klyron` global

The `Klyron` global provides access to runtime information:

```typescript
console.log(Klyron.version);     // "0.1.0"
console.log(Klyron.engine);      // "v8"
console.log(Klyron.permissions); // Permission info
console.log(Klyron.args);        // CLI arguments passed to script
```

## TypeScript Types

Type definitions for Klyron APIs are available as a npm package:

```bash
klyron install --save-dev @klyron/types
```

Then reference them in `tsconfig.json`:

```json
{
  "compilerOptions": {
    "types": ["@klyron/types"]
  }
}
```

## Compatibility APIs

When running in compat mode (`--compat node`), these Node.js built-in modules are available:

- `fs`, `path`, `os`, `crypto`, `buffer`, `stream`, `child_process`, `process`
- `require()` function (CommonJS)

For more details on compatibility, see [Migration from npm](../guides/migration-from-npm.md).
