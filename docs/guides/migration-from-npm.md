# Migration from npm

This guide covers migrating a project from npm (or pnpm, yarn) to Klyron.

## Key Differences

| Aspect | npm | Klyron |
|---|---|---|
| Engine | Node.js (V8 only) | V8, Boa, QuickJS, JSC |
| Package manager | npm CLI | Built-in klyron install |
| Lockfile | package-lock.json | klyron.lock |
| Store | node_modules (copies) | Content-addressable store (symlinks) |
| Dev server | Separate (Vite, webpack) | Built-in klyron dev |
| Build tool | Separate (Vite, esbuild) | Built-in klyron build |
| Test runner | Separate (Vitest, Jest) | Built-in klyron test |
| Config files | Multiple (tsconfig, vite.config, jest.config) | Single klyron.json |
| Permissions | Full system access | Sandbox with --allow-* flags |

## Migration Steps

### 1. Install Klyron

```bash
cargo install klyron
```

### 2. Create klyron.json

Generate a starting configuration:

```bash
klyron init
```

This creates a `klyron.json` by analyzing your existing `package.json`, `tsconfig.json`, and framework configuration files.

### 3. Migrate dependencies

```bash
# Remove existing node_modules
rm -rf node_modules package-lock.json

# Install with klyron
klyron install
```

Klyron reads `package.json` and installs all dependencies into its content-addressable store. Existing `package.json` files remain compatible.

### 4. Migrate npm scripts

Replace npm run commands:

| npm | Klyron |
|---|---|
| `npm run dev` | `klyron dev` |
| `npm run build` | `klyron build` |
| `npm test` | `klyron test` |
| `npm run <script>` | `klyron run <script>` or keep `npm run <script>` |
| `npx <pkg>` | `klyron run --eval '...'` or `klyron install -g <pkg>` |

Klyron can run your existing npm scripts defined in `package.json`:

```bash
klyron run build   # runs "npm run build" equivalent
```

### 5. Migrate config files

Klyron consolidates many config files into `klyron.json`:

```bash
klyron migrate
```

This command reads your existing configuration and produces a `klyron.json` with equivalent settings.

### 6. Update CI/CD

```yaml
# GitHub Actions example
- name: Install Klyron
  run: cargo install klyron

- name: Install dependencies
  run: klyron install --frozen-lockfile

- name: Build
  run: klyron build

- name: Test
  run: klyron test --coverage
```

## Compat Mode

Klyron provides a compatibility mode for projects that aren't fully migrated:

```jsonc
{
  "compat": {
    "nodeBuiltins": true,
    "require": true,
    "commonjs": true,
    "npmScripts": true
  }
}
```

### What Compat Mode Provides

- **Node.js built-in modules**: `fs`, `path`, `http`, etc. are polyfilled
- **CommonJS require**: `require()` works alongside ESM `import`
- **npm scripts**: `npm run <script>` works via delegation
- **node_modules resolution**: Falls back to npm-style resolution if needed

### Running Node.js-specific code

For packages that depend on Node.js APIs not available in Klyron's sandbox, use the Node.js compatibility adapter:

```bash
klyron run --compat node script.js
```

## Potential Issues

### Native modules

Packages with native Node.js addons (`.node` files) may not work in Klyron. Use pure JS alternatives or run those scripts with the Node.js compat adapter.

### Global installs

Packages installed globally with npm need to be re-installed with Klyron:

```bash
klyron install --global typescript
klyron install --global eslint
```

### Permissions

Scripts that previously had full system access will be sandboxed by default. Add `--allow-*` flags as needed:

```bash
klyron run --allow-net --allow-read --allow-write server.js
```

### Environment variables

Klyron reads `.env` files automatically. If you use environment-specific `.env.production` files, they must be explicitly loaded:

```bash
klyron run --env-file .env.production script.js
```

## Rollback

If you need to revert to npm, keep your `package.json` and `package-lock.json` files. Klyron does not modify them. Simply delete `klyron.json` and `klyron.lock`, then run `npm install` to restore the npm-managed state.
