# Klyron Minify Plugin

Minifies JavaScript and CSS files during the `onAfterBuild` hook.

## Installation

```bash
klyron plugin install ./plugins/klyron-plugin-minify
```

## Behavior

- Scans the build output directory for `.js`, `.mjs`, `.css` files
- Strips comments, whitespace, and unnecessary characters
- Reports bytes saved after minification
- Skips `node_modules` and hidden directories
