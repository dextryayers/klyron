# Deploy Configuration Templates

This directory contains deploy configuration templates for various platforms.

## Templates

| File | Platform | Description |
|------|----------|-------------|
| `vercel.json` | Vercel | Vercel deployment configuration |
| `netlify.toml` | Netlify | Netlify deployment configuration |
| `railway.json` | Railway | Railway deployment configuration |
| `fly.toml` | Fly.io | Fly.io deployment configuration |
| `docker-compose.yml` | Docker | Docker Compose with PostgreSQL, Redis |

## Usage

```bash
# Apply a template to your project
cp registry/deploy-templates/vercel.json ./vercel.json

# Or use the klyron CLI
klyron deploy init --platform vercel
```

## Customization

Edit the template files to match your project's specific requirements:
- Update build commands for your framework
- Set the correct port numbers
- Add environment variables
- Configure health check endpoints
