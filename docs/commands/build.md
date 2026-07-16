# `klyron build`

Bundle your application for production with tree-shaking, minification, and multi-target output.

## Usage

```bash
klyron build [options] [entry]
```

## Description

The `build` command compiles your source files into optimized production bundles. It supports multiple output formats, code splitting, and asset handling. If no entry point is specified, Klyron auto-detects it from the project configuration (`index.html`, `src/main.ts`, `src/app.tsx`, etc.).

## Options

| Option | Description |
|---|---|
| `--minify` | Minify output (default: true) |
| `--sourcemap` | Generate sourcemaps (default: false) |
| `--target <targets>` | Output format(s): `esm`, `cjs`, `iife`, `wasm` |
| `--out-dir <dir>` | Output directory (default: dist) |
| `--entry <file>` | Entry point |
| `--format <format>` | Module format: `esm`, `cjs`, `iife` |
| `--splitting` | Enable code splitting |
| `--asset-naming <pattern>` | Asset file naming pattern |
| `--chunk-naming <pattern>` | Chunk file naming pattern |
| `--public-path <path>` | Public asset base path |
| `--clean` | Clean output directory before build |
| `--analyze` | Generate bundle analysis report |
| `--watch` | Rebuild on changes (watch mode) |
| `--profile` | Enable profiling |

## Examples

### Default production build

```bash
klyron build
```

### Build with sourcemaps

```bash
klyron build --sourcemap
```

### Library build (ESM + CJS)

```bash
klyron build --format esm --format cjs --out-dir lib
```

### Custom entry and output

```bash
klyron build src/index.ts --out-dir build --minify
```

### Code splitting

```bash
klyron build --splitting --chunk-naming 'chunks/[name]-[hash].js'
```

### Bundle analysis

```bash
klyron build --analyze
```

## Multi-target Output

Klyron can produce multiple output formats from a single entry point:

### ESM (ECMAScript Modules)

Modern browsers and Node.js ESM environments:

```bash
klyron build --format esm
```

### CJS (CommonJS)

Legacy Node.js environments:

```bash
klyron build --format cjs
```

### IIFE (Immediately Invoked Function Expression)

For direct `<script>` tag usage in browsers:

```bash
klyron build --format iife
```

### WASM

For WebAssembly targets:

```bash
klyron build --format wasm
```

## CSS and Asset Handling

Klyron processes CSS imports, extracts them to separate files, and handles asset references (images, fonts) with content hashing. Configure asset handling in `klyron.json`:

```jsonc
{
  "build": {
    "css": {
      "modules": true,
      "extract": true
    },
    "assets": {
      "inline": true,
      "limit": 8192
    }
  }
}
```

## Tree Shaking

Klyron performs dead-code elimination using static analysis of ES module imports. Only exported symbols that are actually imported and used are included in the bundle. Side-effect annotations (`/*#__PURE__*/`) are respected.
