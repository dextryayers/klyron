# 06 — Full-Stack App

A full-stack single-page application with Klyron serving both an API and a static frontend. The SPA communicates with the backend via `fetch()`.

## Files

| File               | Description                          |
|--------------------|--------------------------------------|
| `package.json`     | Project metadata                     |
| `server.js`        | Klyron HTTP server with API routes   |
| `client/index.html`| Single-page application frontend     |
| `client/app.js`    | Frontend JavaScript                  |

## Run

```bash
klyron run server.js
```

Then visit `http://localhost:3000`.

## API Endpoints

| Method | Path         | Description          |
|--------|-------------|----------------------|
| GET    | `/api/tasks` | List all tasks       |
| POST   | `/api/tasks` | Create a new task    |
| GET    | `/api/health` | Health check        |
