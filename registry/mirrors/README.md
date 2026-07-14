# Mirror Configuration

Klyron supports multiple registry mirrors for plugin and package distribution.

## Mirror Configuration File

Mirrors are configured in `~/.klyron/mirrors.toml`:

```toml
[default]
url = "https://registry.klyron.dev"
fallback = "https://registry.npmjs.org"

[custom]
url = "https://mirror.internal.company.com/klyron"
token = "${KLYRON_MIRROR_TOKEN}"
priority = 1

[mirrors."npm"]
url = "https://registry.npmjs.org"
type = "npm"

[mirrors."deno"]
url = "https://deno.land/x"
type = "deno"

[mirrors."jsr"]
url = "https://jsr.io"
type = "jsr"
```

## Configuration Fields

| Field | Type | Description |
|-------|------|-------------|
| `url` | string | Base URL of the mirror |
| `fallback` | string | Fallback mirror URL |
| `token` | string | Auth token (supports env var `${VAR}`) |
| `priority` | int | Lower number = higher priority (default: 10) |
| `type` | string | Mirror type: `klyron`, `npm`, `deno`, `jsr` |

## Mirror Selection

Mirrors are selected by priority. The highest priority (lowest number) is tried first.
If a mirror fails, the next priority mirror is tried as fallback.

## Environment Variables

- `KLYRON_REGISTRY` - Override default registry URL
- `KLYRON_MIRROR_TOKEN` - Auth token for authenticated mirrors
- `KLYRON_NPM_MIRROR` - Override npm mirror URL
