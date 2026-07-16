# Monorepo Setup

Klyron provides built-in workspace support for managing monorepos with multiple packages.

## Workspace Configuration

Define workspaces in `klyron.json` at the root of your monorepo:

```jsonc
{
  "workspaces": ["packages/*"],
  "install": {
    "hoist": true,
    "linkWorkspacePackages": true
  }
}
```

### Glob Patterns

Workspace paths support glob patterns:

```jsonc
{
  "workspaces": [
    "packages/*",
    "apps/*",
    "libs/*",
    "tools/*"
  ]
}
```

## `klyron workspace` Commands

### List workspace packages

```bash
klyron workspace list
```

Output:

```
packages/core
packages/utils
packages/react-hooks
apps/web
apps/docs
```

### Run a command in all packages

```bash
klyron workspace run build
klyron workspace run --parallel test
klyron workspace run --filter packages/utils lint
```

### Filtering

Workspace commands support filters:

```bash
# Run in specific packages
klyron workspace run --filter packages/core build
klyron workspace run --filter 'packages/*' test

# Run in changed packages since last commit
klyron workspace run --filter '[main]' test

# Exclude packages
klyron workspace run --filter '!apps/docs' build
```

### Dependency graph

Visualize the workspace dependency graph:

```bash
klyron workspace graph
```

This outputs a dependency graph showing which packages depend on each other, helping identify circular dependencies.

## Shared Dependencies

### Hoisting

By default, Klyron hoists shared dependencies to the root `node_modules` using a pnpm-compatible symlinking strategy. This means:

- Common dependencies are deduplicated and stored once
- Packages can access root-level dependencies
- The `.klyron-store` directory provides content-addressable storage

### Explicit shared dependencies

You can specify dependencies that should be shared across all workspace packages in the root `klyron.json`:

```jsonc
{
  "sharedDependencies": {
    "typescript": "^5.0.0",
    "eslint": "^8.0.0",
    "@types/react": "^18.0.0"
  }
}
```

### Package-level overrides

Individual packages can override shared dependency versions in their own `klyron.json`:

```jsonc
// packages/legacy/klyron.json
{
  "overrides": {
    "react": "^17.0.0"
  }
}
```

## Workspace Protocol

Workspace packages can reference each other using the `workspace:` protocol or by version:

```json
{
  "name": "@my-app/web",
  "dependencies": {
    "@my-app/core": "workspace:*",
    "@my-app/utils": "workspace:^1.0.0"
  }
}
```

The `workspace:*` alias always resolves to the local workspace version. When publishing, workspace references are replaced with actual version ranges.

## TypeScript Project References

For TypeScript monorepos, Klyron integrates with TypeScript project references:

```jsonc
// packages/core/tsconfig.json
{
  "compilerOptions": {
    "composite": true,
    "outDir": "./dist",
    "rootDir": "./src"
  }
}

// apps/web/tsconfig.json
{
  "references": [
    { "path": "../../packages/core" },
    { "path": "../../packages/utils" }
  ]
}
```

## Example Monorepo Structure

```
my-monorepo/
├── klyron.json              # Workspace root config
├── klyron.lock              # Lockfile
├── packages/
│   ├── core/                # Core library
│   │   ├── klyron.json
│   │   ├── src/
│   │   └── package.json
│   ├── utils/               # Utility library
│   │   ├── klyron.json
│   │   ├── src/
│   │   └── package.json
│   └── react-hooks/         # React hooks library
│       ├── klyron.json
│       ├── src/
│       └── package.json
├── apps/
│   ├── web/                 # Web application
│   │   ├── klyron.json
│   │   ├── src/
│   │   └── package.json
│   └── docs/                # Documentation site
│       ├── klyron.json
│       ├── src/
│       └── package.json
└── tsconfig.base.json       # Shared TypeScript config
```
