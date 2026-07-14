# Node.js Compatibility Guide

Klyron aims for broad Node.js API compatibility. This guide documents which Node.js core modules work in Klyron and any behavioral differences.

## Supported Node.js Core Modules

| Module | Status | Notes |
|--------|--------|-------|
| `assert` | ✅ Full | `strict` mode supported |
| `buffer` | ✅ Full | `Buffer`, `Blob` support |
| `child_process` | ✅ Full | `exec`, `spawn`, `fork`, `execFile` |
| `crypto` | ✅ Full | `createHash`, `createHmac`, `randomBytes`, `pbkdf2` |
| `dns` | ✅ Full | `lookup`, `resolve4`, `resolve6`, `resolveMx` |
| `events` | ✅ Full | `EventEmitter`, `on`, `once` |
| `fs` | ✅ Full | Sync and async APIs, `fs.promises` |
| `http` | ✅ Full | Server and client |
| `https` | ✅ Full | Server and client |
| `net` | ✅ Full | TCP and IPC |
| `os` | ✅ Full | `platform`, `arch`, `cpus`, `homedir`, `tmpdir` |
| `path` | ✅ Full | `join`, `resolve`, `basename`, `dirname`, `extname` |
| `process` | ✅ Full | `env`, `argv`, `cwd`, `exit`, `pid` |
| `querystring` | ✅ Full | `parse`, `stringify` |
| `stream` | ✅ Full | Readable, Writable, Transform, Duplex |
| `string_decoder` | ✅ Full | `StringDecoder` |
| `timers` | ✅ Full | `setTimeout`, `setInterval`, `setImmediate` |
| `url` | ✅ Full | `URL`, `URLSearchParams`, `parse`, `format` |
| `util` | ✅ Full | `promisify`, `callbackify`, `inspect`, `types` |
| `zlib` | ✅ Partial | `gzip`, `gunzip`, `deflate`, `inflate` |
| `cluster` | ⚠️ Partial | Limited multi-process support |
| `worker_threads` | ⚠️ Partial | Web Workers recommended instead |

## Key Differences

- **File System**: Klyron uses an async runtime; prefer `fs.promises` APIs for non-blocking operations
- **Child Process**: `child_process.spawn` works identically; `execFile` uses internal lookup
- **Crypto**: OpenSSL-backed, same API surface as Node.js
- **process.env**: Environment variables are read-only in strict sandbox mode
- **No native addons**: C/C++ native addons are not supported; use FFI instead
- **No `require.resolve`**: Use import maps or file paths

## Migration Tips

1. Replace `require('fs')` with `import * as fs from 'fs'` or use `fs.promises`
2. Use `fetch()` instead of `http.request()` for simpler HTTP calls
3. Klyron supports top-level `await` in all modules
4. Use `process.env.KLYRON_CACHE` for cache directory
