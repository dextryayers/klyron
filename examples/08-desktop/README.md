# 08 — Desktop App

A desktop application example using Klyron as the backend with an HTML/CSS frontend rendered in a webview. Demonstrates how Klyron can be used for local desktop-style applications.

## Files

| File               | Description                        |
|--------------------|------------------------------------|
| `package.json`     | Project metadata                   |
| `main.js`          | Desktop entry point (Klyron server)|
| `public/index.html`| Main window HTML                   |
| `src/renderer.js`  | Frontend JavaScript for the UI     |

## Run

```bash
klyron run main.js
```

Then open `http://localhost:3000` in your browser (or point a webview/Electron shell at it).

## Notes

- In production, this would be wrapped with Electron, Tauri, or a system webview.
- Klyron handles the backend logic; the frontend is a standard web UI.
