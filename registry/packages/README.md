# Package Storage Format

Packages in the Klyron registry are stored and distributed in a standardized format.

## Storage Directory Structure

```
~/.klyron/
  packages/
    <scope>/
      <name>/
        <version>/
          package.tar.gz
          manifest.json
          checksums.sha256
    cache/
      <hash>.tar.gz
  plugins/
    <name>.wasm
    <name>.toml
```

## Package Tarball Layout

Each package is a `.tar.gz` archive:

```
package.tar.gz
  package/
    index.js              # Entry point
    package.json          # NPM-compatible metadata
    README.md             # Documentation
    LICENSE               # License file
    dist/                 # Built assets
      index.js
      index.d.ts
    node_modules/         # Bundled dependencies (optional)
```

## Manifest File

The `manifest.json` contains package metadata:

```json
{
  "name": "@scope/package",
  "version": "1.0.0",
  "description": "Package description",
  "main": "index.js",
  "types": "dist/index.d.ts",
  "license": "MIT",
  "dependencies": {
    "dep1": "^1.0.0",
    "dep2": "~2.3.0"
  },
  "klyron": {
    "engines": ["node", "deno"],
    "polyglot": true,
    "wasm": false
  }
}
```

## Checksums

Each package version includes a `checksums.sha256` file:

```
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  package.tar.gz
a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a  manifest.json
```

## Version Resolution

Klyron uses semver for version resolution:

- `x.y.z` - Exact version
- `^x.y.z` - Compatible with x.y.z
- `~x.y.z` - Approximately x.y.z
- `>=x.y.z` - Minimum version
- `*` - Any version
