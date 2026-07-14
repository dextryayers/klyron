# WordPress Project

Scaffolded by Klyron. To download WordPress core, use:

```bash
wp core download
```

Then configure the database:

```bash
wp core config --dbname=wordpress --dbuser=root --dbpass=
wp db create
wp core install --url=localhost --title="My Site" --admin_user=admin --admin_password=admin --admin_email=admin@example.com
```

For the dev server:

```bash
php -S localhost:8000
```
