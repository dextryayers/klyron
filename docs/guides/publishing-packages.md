# Publishing Packages

Klyron provides a built-in publishing workflow with versioning, package signing, and registry integration.

## Publishing Workflow

### Prepare your package

Ensure your `klyron.json` includes publishing metadata:

```jsonc
{
  "name": "@my-org/my-package",
  "version": "1.0.0",
  "publish": {
    "registry": "https://registry.klyron.dev",
    "access": "public",
    "files": ["dist", "src", "README.md", "LICENSE.md"],
    "sign": true
  }
}
```

### Build for publish

```bash
klyron build --format esm --format cjs --out-dir dist
```

### Publish

```bash
klyron publish
```

This command:

1. Validates the package manifest
2. Runs the pre-publish script (if configured)
3. Builds the package (if not already built)
4. Signs the package tarball with your GPG key
5. Uploads to the registry
6. Creates a git tag for the version

### Dry run

```bash
klyron publish --dry-run
```

This simulates publishing without actually uploading.

## Versioning

### Manual version bump

```bash
klyron publish patch    # 1.0.0 -> 1.0.1
klyron publish minor    # 1.0.0 -> 1.1.0
klyron publish major    # 1.0.0 -> 2.0.0
klyron publish 2.0.0-rc.1  # Specific version
```

### Version lifecycle

The `publish` command follows semantic versioning conventions:

- **Patch** (`1.0.0` -> `1.0.1`): Bug fixes
- **Minor** (`1.0.0` -> `1.1.0`): New features, backward-compatible
- **Major** (`1.0.0` -> `2.0.0`): Breaking changes

### Pre-release versions

```bash
klyron publish prerelease --preid alpha   # 1.0.0 -> 1.0.1-alpha.0
klyron publish prerelease --preid beta    # 1.0.0 -> 1.0.1-beta.0
klyron publish prerelease --preid rc      # 1.0.0 -> 1.0.1-rc.0
```

## Package Signing

Klyron supports GPG-based package signing for supply chain security:

### Configure signing

```bash
klyron publish --sign
klyron publish --sign-key 0xABC123DEF456
```

The signing key can also be set in `klyron.json`:

```jsonc
{
  "publish": {
    "sign": true,
    "signKey": "0xABC123DEF456"
  }
}
```

### Verification

Users can verify package signatures on install:

```bash
klyron install --verify-signatures
```

## Registry

### Default registry

The default Klyron registry is `https://registry.klyron.dev`. Packages are published as compressed tarballs with a metadata JSON file.

### Self-hosted registry

```bash
klyron publish --registry https://registry.my-org.com
```

Or configure in `klyron.json`:

```jsonc
{
  "publish": {
    "registry": "https://registry.my-org.com"
  }
}
```

### Authentication

```bash
klyron login --registry https://registry.my-org.com
klyron logout
```

Tokens are stored in `~/.config/klyron/auth.json` or can be set via the `KLYRON_TOKEN` environment variable.

## Pre-publish and Post-publish Hooks

```jsonc
{
  "scripts": {
    "prepublish": "npm run build && npm test",
    "postpublish": "npm run deploy-docs"
  }
}
```

Hooks run in the project root directory before and after publishing.

## Tags and Distributions

Publish with distribution tags:

```bash
klyron publish --tag latest
klyron publish --tag next
klyron publish --tag experimental
```

Tags allow users to install specific release channels:

```bash
klyron install @my-org/my-package@next
```

## Deprecating and Unpublishing

### Deprecate a version

```bash
klyron deprecate @my-org/my-package@1.0.0 "Use 2.0.0 instead"
```

### Unpublish a version

```bash
klyron unpublish @my-org/my-package@1.0.0
```

Unpublishing is restricted within 72 hours of publish. After that, versions are immutable.
