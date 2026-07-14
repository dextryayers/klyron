# 02 — HTTP Server

A basic HTTP server example demonstrating Klyron's unified `Klyron.serve()` API. The same API works across JavaScript and TypeScript without requiring external dependencies like Express or Koa.

## Files

| File          | Description              |
|--------------|--------------------------|
| `server.js`  | HTTP server (JavaScript) |
| `server.ts`  | HTTP server (TypeScript) |
| `index.html` | Landing page             |
| `package.json` | Project metadata       |

## Run

```bash
klyron run server.js
```

Then visit `http://localhost:3000` in your browser.

## API

```javascript
Klyron.serve({ port: 3000 }, (req) => {
  return new Response("Hello", {
    headers: { "Content-Type": "text/plain" },
  });
});
```
