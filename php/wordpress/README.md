# Klyron WordPress Compatibility

This guide covers running WordPress with Klyron.

## Requirements

- PHP 8.1+
- WordPress 6.x
- Klyron CLI
- MySQL / MariaDB

## Getting Started

```bash
# Create a new WordPress project
klyron create my-blog --template wordpress

# Serve the application
klyron serve

# The WordPress installer will be available at http://localhost:3000
```

## Supported Features

- WordPress core functionality
- Theme and plugin management
- WP-CLI integration
- REST API
- Block editor (Gutenberg)
- Custom post types and taxonomies
- User management and roles
- Comments and moderation
- Media library

## Klyron Integration

- Serve with `klyron serve` for development
- WP-CLI commands via `klyron exec wp <command>`
- `.env` file for database configuration
- Automatic URL rewriting (pretty permalinks)
- File permissions handled automatically

## WP-CLI Commands

```bash
klyron exec wp plugin install woocommerce
klyron exec wp theme activate twentytwentyfour
klyron exec wp db export backup.sql
klyron exec wp search-replace 'http://old' 'https://new'
```

## Known Limitations

- Some plugins requiring Apache mod_rewrite may need configuration
- Email delivery requires SMTP plugin configuration
- Object cache (Redis/Memcached) requires additional setup
- Keep PHP memory limit sufficient for WordPress (~128MB minimum)
