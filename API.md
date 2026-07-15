# Klyron API Reference

## CLI Commands

```
klyron <command> [options] [arguments]
```

### Core Commands

| Command | Description |
|---------|-------------|
| `run <file>` | Execute a JS/TS file |
| `eval <code>` | Evaluate inline JavaScript/TypeScript |
| `repl` | Start interactive REPL |
| `dev [dir]` | Start development server with HMR |
| `build [dir]` | Build project for production |
| `watch <file>` | Watch file for changes and re-execute |

### Package Management

| Command | Description |
|---------|-------------|
| `install [packages]` | Install dependencies + generate klyron.lock |
| `add <package>` | Add dependency |
| `remove <package>` | Remove dependency |
| `update [packages]` | Update dependencies |
| `outdated` | List outdated packages |
| `link [dir]` | Symlink a package |
| `unlink <package>` | Remove symlink |
| `audit` | Scan for vulnerabilities |
| `dedupe` | Deduplicate dependency tree |
| `bundle [package]` | Bundle dependencies |
| `pack` | Create .tgz package |
| `publish` | Publish to registry |
| `dist-tag <cmd> <pkg> <tag>` | Manage distribution tags |

### Lockfile

| Command | Description |
|---------|-------------|
| `lock verify` | Verify klyron.lock integrity |
| `lock migrate` | Migrate from package-lock.json/yarn.lock/pnpm-lock.yaml |
| `lock diff` | Show lockfile changes |
| `lock update` | Regenerate lockfile |
| `--frozen-lockfile` | Fail if lockfile is stale |

### Framework & Scaffold

| Command | Description |
|---------|-------------|
| `create <framework> [project]` | Scaffold new project |
| `workspace list` | List workspace members |
| `workspace add <member> <dep>` | Add dependency to workspace member |
| `orm <cmd>` | ORM operations (generate, migrate, seed) |
| `db <cmd>` | Database operations |

### Development

| Command | Description |
|---------|-------------|
| `format [files]` | Format code |
| `lint [files]` | Lint code |
| `test [files]` | Run tests |
| `coverage` | Generate coverage report |
| `typecheck` | Type-check TypeScript files |

### System

| Command | Description |
|---------|-------------|
| `doctor` | System health check |
| `info` | Display project/system info |
| `config get/set/list` | Manage configuration |
| `cache <cmd>` | Cache management |
| `completions [shell]` | Generate shell completions |
| `upgrade` | Upgrade Klyron |
| `telemetry <cmd>` | Manage telemetry |
| `--version` | Show version |
| `--help` | Show help |

### Common Flags

| Flag | Description |
|------|-------------|
| `--engine <name>` | JS engine: v8, boa, quickjs, jsc, auto |
| `--ts` | Enable TypeScript |
| `--jsx` | Enable JSX |
| `--watch` | Watch mode |
| `--json` | JSON output |
| `--color` | Force color output |
| `--no-color` | Disable color output |
| `--verbose` | Verbose output |
| `--quiet` | Quiet mode |
| `--frozen-lockfile` | Fail if lockfile stale |

## Runtime API

### Klyron.* Globals

```js
Klyron.version          // Runtime version string
Klyron.engine           // Current engine name
Klyron.platform         // OS platform
Klyron.arch             // CPU architecture
Klyron.pid              // Process ID
Klyron.ppid             // Parent process ID
Klyron.cwd()            // Current working directory
Klyron.env              // Environment variables
Klyron.argv             // Command-line arguments
Klyron.execPath         // Path to Klyron binary
Klyron.mainModule       // Entry module
Klyron.builtinModules   // List of built-in modules
Klyron.isPolyfill(name) // Check if module is a polyfill
Klyron.require(id)      // CommonJS require
Klyron.copy(obj)        // Deep copy object
Klyron.readBuffer(path) // Read file as Buffer
Klyron.writeBuffer(path, buf) // Write Buffer to file
```

### Web APIs

```js
fetch(url, options)              // HTTP fetch
Request, Response, Headers       // Fetch API
WebSocket(url, protocols)        // WebSocket client
EventSource(url)                 // Server-Sent Events
URL, URLSearchParams             // URL APIs
URLPattern(pattern)              // URL pattern matching
TextEncoder, TextDecoder         // Text encoding
Blob, File                       // Binary data
FormData                         // Form data
structuredClone(value)           // Deep clone
Event, EventTarget, CustomEvent  // Event system
AbortController, AbortSignal     // Abort API
performance.now()                // High-res timestamp
console.log/warn/error/info      // Console API
setTimeout, setInterval, setImmediate, queueMicrotask
crypto (getRandomValues, subtle) // Web Crypto
BroadcastChannel                 // Tab communication
MessageChannel, MessagePort      // Message passing
CacheStorage, Cache              // Cache API (basic)
```

### Node.js Compat Modules

All 32 modules available via:
```js
const fs = require('fs')         // bare specifier
const fs = require('node:fs')    // node: prefix
import fs from 'node:fs'         // ESM
```

## SDK APIs

### JavaScript SDK (`sdk/js`)
```js
import { KlyronRuntime } from '@klyron/sdk'
// All node:* polyfills as named exports
// Klyron.builtinModules, Klyron.isPolyfill(), Klyron.require()
```

### TypeScript SDK (`sdk/ts`)
```ts
import { KlyronRuntime } from '@klyron/sdk'
// Full type declarations for all node:* and Web APIs
```

### Rust SDK (`sdk/rust`)
```rust
use klyron_sdk::KlyronRuntime;
// Async runtime with tokio
// Full HttpClient, KlyronCrypto, KlyronFS, KlyronEnv, KlyronProcess
```

### C++ SDK (`sdk/cpp`)
```cpp
#include <klyron.hpp>
klyron::KlyronRuntime rt;
// Cross-platform (WIN32/MACOS/LINUX)
```

### PHP SDK (`sdk/php`)
```php
use Klyron\KlyronRuntime;
$rt = new KlyronRuntime();
```

### Python SDK (`sdk/python`)
```python
from klyron import KlyronRuntime
rt = KlyronRuntime()
```

### Go SDK (`sdk/go`)
```go
import "github.com/klyron/sdk-go"
rt := klyron.NewRuntime()
```

### Ruby SDK (`sdk/ruby`)
```ruby
require 'klyron'
rt = Klyron::Runtime.new
```

### Zig SDK (`sdk/zig`)
```zig
const klyron = @import("klyron");
var rt = klyron.KlyronRuntime.init();
```

## Plugin API

```rust
pub trait PluginTrait: Send + Sync {
    fn name(&self) -> &str;
    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn start(&mut self) -> Result<(), PluginError>;
    fn stop(&mut self) -> Result<(), PluginError>;
    fn handle_request(&self, req: &Request) -> Result<Response, PluginError>;
    fn handle_event(&self, event: &PluginEvent) -> Result<(), PluginError>;
}
```

### Plugin Hooks

- PRE_INSTALL, POST_INSTALL
- PRE_BUILD, POST_BUILD
- PRE_DEV, POST_DEV
- PRE_REQUEST, POST_REQUEST

### Plugin Permissions

- FS_READ, FS_WRITE
- NET_CONNECT, NET_LISTEN
- ENV_READ, ENV_WRITE
- PROCESS_SPAWN
- ALL

## Configuration (klyron.json)

```json
{
  "compilerOptions": { "jsx": "react-jsx", "strict": true },
  "engine": "auto",
  "port": 3000,
  "plugins": [
    { "name": "my-plugin", "version": "1.0.0", "enabled": true, "permissions": ["fs:read"] }
  ],
  "frameworks": ["next", "prisma"],
  "deploy": { "target": "vercel", "region": "iad1" },
  "telemetry": false
}
```
