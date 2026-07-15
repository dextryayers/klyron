# Klyron TypeScript Plugin

Compiles TypeScript (.ts, .tsx) files to JavaScript during the `onBeforeBuild` hook.

## Installation

```bash
klyron plugin install ./plugins/klyron-plugin-typescript
```

## Behavior

- Scans the build directory for `.ts` and `.tsx` files
- Strips TypeScript-specific syntax (interfaces, types, annotations)
- Outputs `.js` files alongside original `.ts` files
- Skips `node_modules` and hidden directories
