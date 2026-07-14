# Security Policy

## Supported Versions

The following versions of Klyron are currently supported with security updates:

| Version | Supported |
| ------- | --------- |
| 0.x     | ✅ Active |

We recommend always using the latest release to ensure you have the most up-to-date security fixes.

## Reporting a Vulnerability

We take security vulnerabilities seriously. Please report vulnerabilities responsibly.

### Process

1. **Do not** open a public GitHub issue for security vulnerabilities
2. Send a detailed report to **security@klyron.dev**
3. Include the following information:
   - Type of vulnerability
   - Full description and impact
   - Steps to reproduce
   - Affected versions
   - Any potential mitigations you've identified

### Response Timeline

- **24 hours**: Initial acknowledgment of your report
- **72 hours**: Preliminary assessment and remediation plan
- **7 days**: Patch release or detailed mitigation timeline for complex issues

### Disclosure Policy

- We will coordinate disclosure with you
- We aim to release fixes before public disclosure
- We will credit researchers who report valid vulnerabilities (with their consent)

## Security Contact

- **Email**: security@klyron.dev
- **PGP Key**: Available on request

## Best Practices

When using Klyron in production:

- Run the latest stable version
- Enable sandboxing (`KLYRON_SANDBOX=strict` or `maximum`) in untrusted environments
- Set appropriate memory limits with `KLYRON_ENGINE_MEMORY_LIMIT`
- Disable telemetry in sensitive environments: `KLYRON_TELEMETRY=false`
- Use a dedicated non-root user for the Klyron process
- Regularly update dependencies
