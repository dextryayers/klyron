# Klyron Environment Validator Plugin

Validates required environment variables before the `onBeforeServe` hook.

## Installation

```bash
klyron plugin install ./plugins/klyron-plugin-env
```

## Behavior

- Checks that `NODE_ENV`, `PORT`, `HOST` are set and non-empty
- Warns if `DATABASE_URL`, `API_KEY` are missing
- Fails the serve if required variables are missing
