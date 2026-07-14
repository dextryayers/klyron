# 05 — Next.js with Klyron

A Next.js-style application demonstrating how Klyron can run Next.js pages and API routes. Uses file-based routing and React components with Klyron's HTTP server.

## Files

| File               | Description                       |
|--------------------|-----------------------------------|
| `package.json`     | Project metadata                  |
| `next.config.js`   | Next.js configuration             |
| `pages/index.jsx`  | Home page component               |
| `pages/api/hello.js` | API route handler               |

## Run

```bash
klyron run node_modules/next/dist/bin/next dev
```

Or with the custom Klyron adapter:

```bash
klyron run server.js
```

Then visit `http://localhost:3000`.
