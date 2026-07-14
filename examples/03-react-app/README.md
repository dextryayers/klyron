# 03 — React SSR App

A React server-side rendering example using Klyron's runtime. Components are rendered to HTML on the server using `ReactDOMServer.renderToString`, producing a fully populated page before it reaches the client.

## Files

| File           | Description                       |
|----------------|-----------------------------------|
| `server.js`    | React SSR server (Klyron runtime) |
| `package.json` | Project metadata                  |
| `src/App.jsx`  | Simple React component            |
| `src/index.html` | HTML template                   |

## Run

```bash
klyron run server.js
```

Then visit `http://localhost:3000` in your browser.

## How it works

1. Klyron's `serve()` API receives an HTTP request.
2. The React component is rendered to an HTML string via `ReactDOMServer.renderToString()`.
3. The rendered string is injected into an HTML template and returned as the response.
