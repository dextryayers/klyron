# Python Interoperability Guide

This guide covers running Python alongside Klyron, calling Python scripts, and sharing data between runtimes.

## Running Python Scripts

### From Klyron (TypeScript)
```typescript
import { exec } from './process';

const result = await exec('python3', ['-c', 'import json; print(json.dumps({"hello": "world"}))']);
console.log(result.stdout); // {"hello": "world"}
```

### With virtual environment
```typescript
await exec('./venv/bin/python', ['script.py']);
```

## Data Exchange

### JSON via stdin/stdout
```python
import sys, json
data = json.load(sys.stdin)
print(json.dumps({"result": data["x"] + data["y"]}))
```

```typescript
import { execShell } from './process';

const result = await execShell(`echo '${JSON.stringify({ x: 1, y: 2 })}' | python3 add.py`);
```

## Framework Integration

### Django
- Run `python manage.py` commands via `klyron exec`
- Klyron can serve Django's static files
- Environment variables from Klyron's `.env` are passed to Django

### FastAPI / Flask
- Klyron can proxy or manage Python web servers
- Use Klyron's process manager for production deployments
- Hot reload support with Klyron's dev server

## Shared Environment

- Environment variables pass from Klyron to Python processes
- Both runtimes share the same filesystem
- Use `klyron exec python3 -m pip install <package>` to manage dependencies
- Klyron's `.env` file format is compatible with `python-dotenv`
