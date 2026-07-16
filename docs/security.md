# Security Policy

Klyron takes security seriously. This document outlines our security policies, threat model, and vulnerability reporting process.

## Supported Versions

| Version | Supported |
|---|---|
| 0.x (latest) | Active development, security fixes applied |
| < 0.1.0 | Not supported |

## Threat Model

Klyron's threat model considers the following attacker scenarios:

### Untrusted Code Execution

An attacker provides malicious JavaScript, TypeScript, or PHP code that is executed by a user.

**Mitigations:**
- Permission-based sandbox with explicit `--allow-*` flags
- No I/O access by default (files, network, env, subprocesses)
- WASM sandbox for plugins with separate permission model
- Resource limits (memory, CPU time) for runaway scripts

### Supply Chain Attacks

An attacker publishes a malicious package to the registry or compromises an existing package.

**Mitigations:**
- Package signing with GPG signatures
- TUF (The Update Framework) integration for metadata security
- SRI (Subresource Integrity) hashes in lockfile
- Content-addressable storage prevents hash collision attacks
- Lockfile verification with `--frozen-lockfile`
- Dependency audit command (`klyron audit`)

### Registry Compromise

An attacker compromises the Klyron registry infrastructure.

**Mitigations:**
- TUF metadata signing prevents tampered package listings
- Package signatures verified on install (optional, enabled by default)
- Rollback attack prevention via TUF timestamp metadata
- Multi-signer threshold for critical metadata updates

### Network Eavesdropping

An attacker intercepts traffic between Klyron and the registry.

**Mitigations:**
- All registry communication over HTTPS/TLS
- Certificate pinning for registry.klyron.dev
- Package integrity verified via SRI hashes after download

## Vulnerability Reporting

### Reporting Process

If you discover a security vulnerability, **do not** open a public GitHub issue. Instead, report it privately:

1. **Email**: security@klyron.dev
2. **PGP Key**: Available at https://klyron.dev/security/pgp.asc (Key ID: 0xSECURITY)

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact
- Any suggested mitigations (optional)

### Response Timeline

| Timeframe | Action |
|---|---|
| 24 hours | Acknowledgment of receipt |
| 72 hours | Initial triage and severity assessment |
| 7 days | Fix development plan communicated |
| 30 days | Public disclosure (coordinated) |

We aim to release patches within 7 days for critical vulnerabilities and 30 days for moderate severity issues.

## Sandbox Details

Klyron's sandbox is built on the principle of capability-based security.

### Permission Flags

| Flag | Grants | Risk |
|---|---|---|
| `--allow-net` | TCP/UDP network access | High |
| `--allow-read` | Read any file on the filesystem | High |
| `--allow-write` | Write any file on the filesystem | Critical |
| `--allow-env` | Read all environment variables | Medium |
| `--allow-run` | Spawn subprocesses | Critical |
| `--allow-ffi` | Call native code via FFI | Critical |
| `--allow-all` | All of the above | Critical |

### Granular Permissions

Network and filesystem permissions can be scoped:

```bash
# Allow network access to specific hosts
klyron run --allow-net=api.example.com,registry.klyron.dev server.ts

# Allow read access to specific paths
klyron run --allow-read=/app/src,/app/config build.ts

# Allow write access to specific paths
klyron run --allow-write=/app/dist build.ts
```

### Sandbox Implementation

- **Linux**: seccomp BPF for syscall filtering, namespaces for process isolation
- **macOS**: sandbox_init() with custom profile, seatbelt sandbox
- **Windows**: AppContainer isolation, restricted tokens

## Supply Chain Security

### TUF (The Update Framework)

Klyron's registry uses TUF to secure the package distribution pipeline:

- **Root metadata**: Signed by offline root keys, defines trusted keys
- **Targets metadata**: Lists all available packages with their hashes
- **Snapshot metadata**: Lists versions of all metadata files
- **Timestamp metadata**: Ensures freshness, prevents rollback attacks

### SRI (Subresource Integrity)

All packages in the lockfile include SRI hashes:

```json
{
  "lodash@4.17.21": {
    "integrity": "sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg==",
    "signature": "-----BEGIN PGP SIGNATURE-----..."
  }
}
```

### Package Signing

Maintainers can sign packages with GPG:

```bash
klyron publish --sign
klyron install --verify-signatures
```

Signatures are verified against the registry's key server or a custom keyring.

## Security Audits

Klyron undergoes regular security audits:

| Date | Auditor | Scope | Report |
|---|---|---|---|
| TBD | TBD | Initial audit | Pending |

See [SECURITY_AUDIT.md](./SECURITY_AUDIT.md) for details.

## Best Practices for Users

1. **Never run untrusted code with `--allow-all`**
2. **Always use `--frozen-lockfile` in CI**
3. **Enable signature verification** for production installs
4. **Pin your Klyron version** in CI/CD configurations
5. **Run `klyron audit`** regularly on your projects
6. **Review dependencies** for known vulnerabilities before major upgrades
7. **Use scoped permissions** instead of broad `--allow-*` flags

## Reporting Non-security Bugs

For non-security bugs, please open a standard GitHub issue at https://github.com/anomalyco/klyron/issues.
