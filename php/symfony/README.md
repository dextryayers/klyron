# Klyron Symfony Compatibility

This guide covers running Symfony applications with Klyron.

## Requirements

- PHP 8.2+
- Symfony 6.x / 7.x
- Klyron CLI

## Getting Started

```bash
# Create a new Symfony project
klyron create my-app --template symfony

# Serve the application
klyron serve

# Run Symfony Console commands
klyron exec php bin/console cache:clear
```

## Supported Features

- Symfony routing and controllers
- Doctrine ORM / DBAL
- Twig templating
- Symfony Messenger
- Mailer component
- Serializer
- Validator
- Security bundle
- Symfony Console commands
- Web Profiler toolbar (dev mode)

## Klyron Integration

- Replace `symfony server:start` with `klyron serve`
- Environment variables map directly to Symfony's `.env`
- Klyron's cache layer can back Symfony's cache pools
- Process management for Messenger workers
- Deploy with `klyron deploy` or `klyron deploy --docker`

## Console Commands

```bash
klyron exec php bin/console doctrine:migrations:migrate
klyron exec php bin/console messenger:consume async
klyron exec php bin/console cache:warmup
```

## Known Limitations

- Symfony's built-in server is replaced by Klyron's server
- Some low-level PHP extensions must be available
- Long-running commands run through Klyron's process manager
