# Klyron LiveReload Plugin

Injects a Server-Sent Events (SSE) live-reload script into HTML files during the `onAfterBuild` hook.

## Installation

```bash
klyron plugin install ./plugins/klyron-plugin-livereload
```

## Behavior

- Scans the build output for `.html` and `.htm` files
- Injects a small `<script>` tag before `</body>` or `</html>`
- The script connects to `/__klyron_livereload` SSE endpoint
- On `reload` event, the page refreshes automatically
