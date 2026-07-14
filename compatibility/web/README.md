# Web API Compatibility Guide

Klyron implements standard Web Platform APIs for maximum portability. This guide documents supported APIs.

## Supported Web APIs

### Fetch API
```typescript
const response = await fetch('https://api.example.com/data');
const data = await response.json();
```
- `fetch()` - Full support with Request/Response
- `Request`, `Response`, `Headers` - Full support
- `AbortController`, `AbortSignal` - Full support
- `FormData` - Supported
- `URL`, `URLSearchParams` - Full support

### Web Crypto API
```typescript
const key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, ['encrypt', 'decrypt']);
```
- `crypto.getRandomValues()` - ✅
- `crypto.randomUUID()` - ✅
- `crypto.subtle.digest()` - SHA-1, SHA-256, SHA-384, SHA-512
- `crypto.subtle.encrypt()` / `decrypt()` - AES-CBC, AES-GCM
- `crypto.subtle.generateKey()` - ✅
- `crypto.subtle.importKey()` / `exportKey()` - ✅
- `crypto.subtle.sign()` / `verify()` - HMAC, ECDSA

### Console API
- `console.log`, `console.info`, `console.warn`, `console.error`
- `console.debug`, `console.trace`, `console.table`
- `console.time`, `console.timeEnd`, `console.timeLog`
- `console.group`, `console.groupEnd`, `console.groupCollapsed`
- `console.count`, `console.countReset`

### Timers
- `setTimeout` / `clearTimeout` - ✅
- `setInterval` / `clearInterval` - ✅
- `setImmediate` / `clearImmediate` - ✅
- `queueMicrotask` - ✅

### Streams API
- `ReadableStream` - ✅
- `WritableStream` - ✅
- `TransformStream` - ✅
- `TextEncoder` / `TextDecoder` - ✅

### Encoding API
- `TextEncoder` - ✅ (UTF-8)
- `TextDecoder` - ✅ (UTF-8, UTF-16, Latin1)
- `atob` / `btoa` - ✅

### WebSocket API
- `WebSocket` - ✅
- `WebSocketPair` - ❌ (Cloudflare-specific)

### Web Workers
- `Worker` - ✅ (same-origin only)
- `MessageChannel`, `MessagePort` - ✅

### Event Target
- `EventTarget` - ✅
- `Event` - ✅
- `CustomEvent` - ✅
- `addEventListener`, `removeEventListener`, `dispatchEvent` - ✅

## Not Supported

- DOM APIs (document, window, HTMLElement)
- Service Workers (use Klyron's built-in HTTP)
- WebGL / Canvas
- WebRTC
- WebUSB / WebBluetooth / WebNFC
- History / Navigation APIs
- IndexedDB (use Klyron KV or SQLite instead)
- CacheStorage (use Klyron Cache)
- `localStorage` / `sessionStorage` (use Klyron KV)
