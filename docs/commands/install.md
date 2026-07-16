# `klyron install`

Install packages from a registry with fast, disk-efficient dependency management.

## Usage

```bash
klyron install [options] [package[@version] ...]
klyron install  # Install all dependencies from manifest
```

## Description

The `install` command resolves, downloads, and links dependencies. It uses a content-addressable store for deduplication across projects (similar to pnpm), resulting in faster installs and less disk usage. A lockfile (`klyron.lock`) is generated to ensure reproducible installs.

## Options

| Option | Description |
|---|---|
| `--save-prod` | Save to `dependencies` (default) |
| `--save-dev` | Save to `devDependencies` |
| `--save-optional` | Save to `optionalDependencies` |
| `--save-exact` | Save exact version (no ^ or ~) |
| `--global` | Install globally |
| `--frozen-lockfile` | Fail if lockfile needs update |
| `--no-lockfile` | Skip lockfile generation |
| `--registry <url>` | Override registry URL |
| `--force` | Force re-download of all packages |

## Examples

### Install all dependencies

```bash
klyron install
```

### Install a specific package

```bash
klyron install lodash
klyron install lodash@4.17.21
klyron install lodash@^4.0.0
```

### Install dev dependency

```bash
klyron install --save-dev typescript
klyron install -D vitest
```

### Global install

```bash
klyron install --global klyron-create-react
```

## Lockfile

The `klyron.lock` file locks dependency versions across environments. Commit it to version control. To update all dependencies to latest within range:

```bash
klyron install --force
```

To verify that the lockfile matches the manifest:

```bash
klyron install --frozen-lockfile
```

## Workspace Support

When run in a workspace root, `install` aggregates and installs dependencies for all workspace packages. Shared dependencies are hoisted to the root `node_modules`, while package-specific dependencies remain local.

```bash
# Install all workspace packages
klyron install

# Install in a specific workspace package
klyron install --filter @my-app/utils
```

## Registry Configuration

The default registry is `https://registry.klyron.dev`. Configure registries in `klyron.json`:

```jsonc
{
  "install": {
    "registry": "https://registry.klyron.dev",
    "registries": {
      "npm": "https://registry.npmjs.org",
      "github": "https://npm.pkg.github.com"
    },
    "scopeRegistries": {
      "@my-org": "https://registry.my-org.com"
    }
  }
}
```

Scoped packages (e.g., `@scope/package`) can be mapped to custom registries. Auth tokens are read from environment variables or `.npmrc`/`.klyronrc`.

## Package Resolution Order

1. Workspace packages (if using workspaces)
2. Registry (default or configured)
3. Global store cache (content-addressable)
4. Git repositories (for `git:` dependencies)
5. Local tarballs and paths
