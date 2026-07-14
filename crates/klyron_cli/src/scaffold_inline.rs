use std::path::Path;

pub(crate) fn scaffold_gatsby(_name: &str, _dir: &Path) -> anyhow::Result<()> {
    eprintln!("→ Creating Gatsby project...");
    crate::run_cmd("npx", &["--yes", "gatsby@latest", "new", _name], _dir)?;
    println!("✅ Gatsby app scaffolded at {}", _dir.join(_name).display());
    Ok(())
}

pub(crate) fn scaffold_laravel(name: &str, dir: &Path) -> anyhow::Result<()> {
    let project_dir = dir.join(name);
    if project_dir.exists() { anyhow::bail!("Directory exists: {}", project_dir.display()); }

    for d in &[
        "app/Http/Controllers", "app/Http/Middleware", "app/Http/Livewire",
        "app/Models", "app/Providers", "bootstrap", "config",
        "database/migrations", "database/factories", "database/seeders",
        "public", "resources/views/layouts", "resources/views/livewire",
        "resources/views/auth", "resources/views/posts", "resources/css",
        "routes", "storage/logs", "lang",
    ] {
        std::fs::create_dir_all(project_dir.join(d))?;
    }

    crate::write_files(&project_dir, vec![
        ("composer.json", &format!(r#"{{
    "name": "app/{}",
    "type": "project",
    "require": {{
        "php": "^8.2",
        "laravel/framework": "^11.0",
        "livewire/livewire": "^3.5",
        "laravel/sanctum": "^4.0",
        "spatie/laravel-permission": "^6.0"
    }},
    "require-dev": {{
        "fakerphp/faker": "^1.23",
        "phpunit/phpunit": "^11.0"
    }},
    "autoload": {{
        "psr-4": {{
            "App\\": "app/",
            "Database\\Factories\\": "database/factories/",
            "Database\\Seeders\\": "database/seeders/"
        }}
    }},
    "scripts": {{
        "post-autoload-dump": [
            "Illuminate\\Foundation\\ComposerScripts::postAutoloadDump",
            "@php artisan package:discover --ansi"
        ],
        "post-root-package-install": [
            "@php -r \"file_exists('.env') || copy('.env.example', '.env');\""
        ],
        "post-create-project-cmd": [
            "@php artisan key:generate --ansi",
            "@php artisan storage:link --ansi"
        ]
    }}
}}"#, name)),
        (".env.example", "APP_NAME=Laravel\nAPP_ENV=local\nAPP_KEY=\nAPP_DEBUG=true\nAPP_URL=http://localhost\nLOG_CHANNEL=stack\nLOG_LEVEL=debug\nDB_CONNECTION=pgsql\nDB_HOST=127.0.0.1\nDB_PORT=5432\nDB_DATABASE=laravel\nDB_USERNAME=postgres\nDB_PASSWORD=secret\n"),
        (".gitignore", "/vendor\n/node_modules\n.env\n.phpunit.result.cache\nstorage/logs/*\n!storage/logs/.gitkeep\n"),
        ("artisan", "#!/usr/bin/env php\n<?php\nrequire __DIR__.'/vendor/autoload.php';\n$app = require __DIR__.'/bootstrap/app.php';\n$kernel = $app->make(Illuminate\\Contracts\\Console\\Kernel::class);\n$status = $kernel->handle($input = new Symfony\\Component\\Console\\Input\\ArgvInput);\n$kernel->terminate($input, $kernel->getOutput());\nexit($status);\n"),
        ("bootstrap/app.php", "<?php\n$app = new Illuminate\\Foundation\\Application($_ENV['APP_BASE_PATH'] ?? dirname(__DIR__));\n$app->singleton(Illuminate\\Contracts\\Http\\Kernel::class, App\\Http\\Kernel::class);\n$app->singleton(Illuminate\\Contracts\\Console\\Kernel::class, App\\Console\\Kernel::class);\n$app->singleton(Illuminate\\Contracts\\Debug\\ExceptionHandler::class, App\\Exceptions\\Handler::class);\nreturn $app;\n"),
        ("config/app.php", "<?php\nreturn ['name' => env('APP_NAME', 'Laravel'), 'env' => env('APP_ENV', 'production'), 'debug' => (bool) env('APP_DEBUG', false), 'url' => env('APP_URL', 'http://localhost'), 'timezone' => 'UTC', 'locale' => 'en', 'fallback_locale' => 'en', 'faker_locale' => 'en_US', 'cipher' => 'AES-256-CBC', 'key' => env('APP_KEY'), 'previous_keys' => [], 'maintenance' => ['driver' => 'cache'], 'providers' => [Illuminate\\Auth\\AuthServiceProvider::class, Illuminate\\Broadcasting\\BroadcastServiceProvider::class, Illuminate\\Bus\\BusServiceProvider::class, Illuminate\\Cache\\CacheServiceProvider::class, Illuminate\\Foundation\\Providers\\ConsoleSupportServiceProvider::class, Illuminate\\Cookie\\CookieServiceProvider::class, Illuminate\\Database\\DatabaseServiceProvider::class, Illuminate\\Encryption\\EncryptionServiceProvider::class, Illuminate\\Filesystem\\FilesystemServiceProvider::class, Illuminate\\Foundation\\Providers\\FoundationServiceProvider::class, Illuminate\\Hashing\\HashServiceProvider::class, Illuminate\\Mail\\MailServiceProvider::class, Illuminate\\Notifications\\NotificationServiceProvider::class, Illuminate\\Pagination\\PaginationServiceProvider::class, Illuminate\\Pipeline\\PipelineServiceProvider::class, Illuminate\\Queue\\QueueServiceProvider::class, Illuminate\\Redis\\RedisServiceProvider::class, Illuminate\\Auth\\Passwords\\PasswordResetServiceProvider::class, Illuminate\\Session\\SessionServiceProvider::class, Illuminate\\Translation\\TranslationServiceProvider::class, Illuminate\\Validation\\ValidationServiceProvider::class, Illuminate\\View\\ViewServiceProvider::class, App\\Providers\\AppServiceProvider::class], 'aliases' => Illuminate\\Support\\Facades\\Facade::defaultAliases()->toArray()];\n"),
        ("config/database.php", "<?php\nreturn ['default' => env('DB_CONNECTION', 'pgsql'), 'connections' => ['pgsql' => ['driver' => 'pgsql', 'url' => env('DATABASE_URL'), 'host' => env('DB_HOST', '127.0.0.1'), 'port' => env('DB_PORT', '5432'), 'database' => env('DB_DATABASE', 'laravel'), 'username' => env('DB_USERNAME', 'postgres'), 'password' => env('DB_PASSWORD', ''), 'charset' => 'utf8', 'prefix' => '', 'prefix_indexes' => true, 'search_path' => 'public', 'sslmode' => 'prefer'], 'sqlite' => ['driver' => 'sqlite', 'url' => env('DATABASE_URL'), 'database' => env('DB_DATABASE', database_path('database.sqlite')), 'prefix' => '', 'foreign_key_constraints' => env('DB_FOREIGN_KEYS', true)]], 'migrations' => 'migrations'];\n"),
        ("config/cache.php", "<?php\nreturn ['default' => env('CACHE_DRIVER', 'file'), 'stores' => ['file' => ['driver' => 'file', 'path' => storage_path('framework/cache/data')]], 'prefix' => env('CACHE_PREFIX', 'laravel_cache')];\n"),
        ("config/session.php", "<?php\nreturn ['driver' => env('SESSION_DRIVER', 'file'), 'lifetime' => env('SESSION_LIFETIME', 120), 'expire_on_close' => false, 'encrypt' => false, 'files' => storage_path('framework/sessions'), 'connection' => null, 'table' => 'sessions', 'store' => null, 'lottery' => [2, 100], 'cookie' => env('SESSION_COOKIE', 'laravel_session'), 'path' => '/', 'domain' => env('SESSION_DOMAIN', null), 'secure' => env('SESSION_SECURE_COOKIE', false), 'http_only' => true, 'same_site' => 'lax'];\n"),
        ("config/filesystems.php", "<?php\nreturn ['default' => env('FILESYSTEM_DISK', 'local'), 'disks' => ['local' => ['driver' => 'local', 'root' => storage_path('app')], 'public' => ['driver' => 'local', 'root' => storage_path('app/public'), 'url' => env('APP_URL').'/storage', 'visibility' => 'public']], 'links' => [public_path('storage') => storage_path('app/public')]];\n"),
        ("config/logging.php", "<?php\nreturn ['default' => env('LOG_CHANNEL', 'stack'), 'channels' => ['stack' => ['driver' => 'stack', 'channels' => ['single']], 'single' => ['driver' => 'single', 'path' => storage_path('logs/laravel.log'), 'level' => env('LOG_LEVEL', 'debug')]]];\n"),
        ("config/mail.php", "<?php\nreturn ['default' => env('MAIL_MAILER', 'smtp'), 'mailers' => ['smtp' => ['transport' => 'smtp', 'host' => env('MAIL_HOST', 'smtp.mailgun.org'), 'port' => env('MAIL_PORT', 587), 'encryption' => env('MAIL_ENCRYPTION', 'tls'), 'username' => env('MAIL_USERNAME'), 'password' => env('MAIL_PASSWORD')]], 'from' => ['address' => env('MAIL_FROM_ADDRESS', 'hello@example.com'), 'name' => env('MAIL_FROM_NAME', 'Example')]];\n"),
        ("config/services.php", "<?php\nreturn ['mailgun' => ['domain' => env('MAILGUN_DOMAIN'), 'secret' => env('MAILGUN_SECRET')], 'postmark' => ['token' => env('POSTMARK_TOKEN')], 'ses' => ['key' => env('AWS_ACCESS_KEY_ID'), 'secret' => env('AWS_SECRET_ACCESS_KEY'), 'region' => env('AWS_DEFAULT_REGION', 'us-east-1')]];\n"),
        ("config/cors.php", "<?php\nreturn ['paths' => ['api/*', 'sanctum/csrf-cookie'], 'allowed_methods' => ['*'], 'allowed_origins' => [env('FRONTEND_URL', 'http://localhost:3000')], 'allowed_headers' => ['*'], 'exposed_headers' => [], 'max_age' => 0, 'supports_credentials' => true];\n"),
        ("config/sanctum.php", "<?php\nreturn ['stateful' => explode(',', env('SANCTUM_STATEFUL_DOMAINS', 'localhost,localhost:3000')), 'guard' => ['web'], 'expiration' => null, 'middleware' => ['authenticate_session' => Laravel\\Sanctum\\Http\\Middleware\\AuthenticateSession::class, 'encrypt_cookies' => App\\Http\\Middleware\\EncryptCookies::class, 'verify_csrf_token' => App\\Http\\Middleware\\VerifyCsrfToken::class]];\n"),
        ("routes/web.php", "<?php\nuse Illuminate\\Support\\Facades\\Route;\nRoute::get('/', function () { return view('welcome'); });\n"),
        ("routes/api.php", "<?php\nuse Illuminate\\Support\\Facades\\Route;\nRoute::middleware('auth:sanctum')->get('/user', function (Request $request) { return $request->user(); });\n"),
        ("routes/console.php", "<?php\nuse Illuminate\\Support\\Facades\\Schedule;\nSchedule::command('inspire')->hourly();\n"),
        ("resources/views/welcome.blade.php", "<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>Laravel</title></head>\n<body class=\"antialiased\"><div class=\"relative flex items-top justify-center min-h-screen\"><h1>Laravel</h1></div></body>\n</html>\n"),
        ("resources/views/layouts/app.blade.php", "<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>@yield('title', config('app.name'))</title>@livewireStyles @vite('resources/css/app.css')</head>\n<body><nav>@include('layouts.nav')</nav><main>@yield('content')</main>@livewireScripts @vite('resources/js/app.js')</body>\n</html>\n"),
        ("resources/css/app.css", "@tailwind base;\n@tailwind components;\n@tailwind utilities;\n"),
        ("app/Http/Controllers/Controller.php", "<?php\nnamespace App\\Http\\Controllers;\nabstract class Controller extends \\Illuminate\\Routing\\Controller {}\n"),
        ("app/Models/User.php", "<?php\nnamespace App\\Models;\nuse Illuminate\\Database\\Eloquent\\Factories\\HasFactory;\nuse Illuminate\\Foundation\\Auth\\User as Authenticatable;\nuse Laravel\\Sanctum\\HasApiTokens;\nclass User extends Authenticatable { use HasApiTokens, HasFactory; }\n"),
        ("database/migrations/0001_01_01_000000_create_users_table.php", "<?php\nuse Illuminate\\Database\\Migrations\\Migration;\nuse Illuminate\\Database\\Schema\\Blueprint;\nuse Illuminate\\Support\\Facades\\Schema;\nreturn new class extends Migration { public function up(): void { Schema::create('users', function (Blueprint $table) { $table->id(); $table->string('name'); $table->string('email')->unique(); $table->timestamp('email_verified_at')->nullable(); $table->string('password'); $table->rememberToken(); $table->timestamps(); }); } };\n"),
        ("public/index.php", "<?php\nuse Illuminate\\Foundation\\Application;\nuse Illuminate\\Http\\Request;\ndefine('LARAVEL_START', microtime(true));\nrequire __DIR__.'/../vendor/autoload.php';\n$app = require __DIR__.'/../bootstrap/app.php';\n$kernel = $app->make(Illuminate\\Contracts\\Http\\Kernel::class);\n$response = $kernel->handle($request = Request::capture())->send();\n$kernel->terminate($request, $response);\n"),
        ("phpunit.xml", "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<phpunit xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:noNamespaceSchemaLocation=\"vendor/phpunit/phpunit/phpunit.xsd\" bootstrap=\"vendor/autoload.php\">\n<testsuites><testsuite name=\"Unit\"><directory>tests/Unit</directory></testsuite><testsuite name=\"Feature\"><directory>tests/Feature</directory></testsuite></testsuites></phpunit>\n"),
        ("storage/logs/.gitkeep", ""),
    ])?;

    println!("✅ Laravel app at {}", project_dir.display());
    println!("\n  cd {}", project_dir.display());
    println!("  composer install");
    println!("  cp .env.example .env && php artisan key:generate");
    println!("  npm install && npm run dev\n");
    Ok(())
}

pub(crate) fn scaffold_laravel_stack(name: &str, dir: &Path, stack: &str) -> anyhow::Result<()> {
    scaffold_laravel(name, dir)?;
    let project_dir = dir.join(name);
    match stack {
        "react" => {
            crate::write_files(&project_dir, vec![
                ("resources/js/app.jsx", "import React from 'react'\nimport { createRoot } from 'react-dom/client'\nimport App from './App'\n\ncreateRoot(document.getElementById('app')).render(<App />)\n"),
                ("resources/js/App.jsx", "import React from 'react'\n\nexport default function App() {\n  return <h1>Laravel + React</h1>\n}\n"),
                ("package.json", r#"{"private":true,"scripts":{"dev":"vite","build":"vite build"},"dependencies":{"react":"^18.3","react-dom":"^18.3"},"devDependencies":{"@vitejs/plugin-react":"^4.3","vite":"^5.4","laravel-vite-plugin":"^1.0"}}"#),
            ])?;
            println!("  Stack: React SPA + Tailwind");
        }
        "vue" => {
            crate::write_files(&project_dir, vec![
                ("resources/js/app.js", "import { createApp } from 'vue'\nimport App from './App.vue'\n\ncreateApp(App).mount('#app')\n"),
                ("resources/js/App.vue", "<template>\n  <h1>Laravel + Vue</h1>\n</template>\n\n<script setup>\n</script>\n"),
                ("package.json", r#"{"private":true,"scripts":{"dev":"vite","build":"vite build"},"dependencies":{"vue":"^3.5"},"devDependencies":{"@vitejs/plugin-vue":"^5.1","vite":"^5.4","laravel-vite-plugin":"^1.0"}}"#),
            ])?;
            println!("  Stack: Vue SPA + Tailwind");
        }
        "inertia-react" => {
            crate::write_files(&project_dir, vec![
                ("resources/js/app.jsx", "import { createInertiaApp } from '@inertiajs/react'\nimport { createRoot } from 'react-dom/client'\n\ncreateInertiaApp({\n  resolve: name => import(`./Pages/${name}.jsx`),\n  setup({ el, App, props }) { createRoot(el).render(<App {...props} />) },\n})\n"),
            ])?;
            println!("  Stack: Laravel + Inertia + React");
        }
        "inertia-vue" => {
            crate::write_files(&project_dir, vec![
                ("resources/js/app.js", "import { createInertiaApp } from '@inertiajs/vue3'\nimport { createApp } from 'vue'\n\ncreateInertiaApp({\n  resolve: name => import(`./Pages/${name}.vue`),\n  setup({ el, App, props }) { createApp(App, props).mount(el) },\n})\n"),
            ])?;
            println!("  Stack: Laravel + Inertia + Vue");
        }
        "livewire" => {
            crate::write_files(&project_dir, vec![
                ("resources/views/layouts/app.blade.php", "<!DOCTYPE html>\n<html>\n<head><title>{{ config('app.name') }}</title>@livewireStyles</head>\n<body>\n  {{ $slot }}\n  @livewireScripts\n</body>\n</html>\n"),
            ])?;
            println!("  Stack: Laravel + Livewire 3 + Volt");
        }
        "next" => {
            eprintln!("  Setting up Laravel + Next.js BFF...");
            std::fs::create_dir_all(project_dir.join("next-app"))?;
            crate::write_files(&project_dir, vec![
                ("next-app/package.json", r#"{"private":true,"scripts":{"dev":"next dev","build":"next build"},"dependencies":{"next":"^15.0","react":"^19.0","react-dom":"^19.0"}}"#),
            ])?;
            println!("  Stack: Laravel + Next.js BFF + API");
        }
        "astro" => {
            eprintln!("  Setting up Laravel + Astro...");
            std::fs::create_dir_all(project_dir.join("astro-app"))?;
            println!("  Stack: Laravel + Astro + API");
        }
        "api" => {
            println!("  Stack: Laravel API-only + Sanctum");
        }
        _ => anyhow::bail!("Unknown Laravel stack: {stack}"),
    }
    println!("\n  cd {} && composer install && npm install && npm run dev", project_dir.display());
    Ok(())
}

pub(crate) fn scaffold_django(name: &str, dir: &Path) -> anyhow::Result<()> {
    let project_dir = dir.join(name);
    if project_dir.exists() { anyhow::bail!("Directory exists: {}", project_dir.display()); }

    for sub in &["config", "apps/core", "apps/api", "apps/users", "static", "templates"] {
        std::fs::create_dir_all(project_dir.join(sub))?;
    }

    crate::write_files(&project_dir, vec![
        ("requirements.txt", "django>=4.2,<5.1\ndjangorestframework>=3.15\ndjango-cors-headers>=4.4\npsycopg2-binary>=2.9\npython-decouple>=3.8\ngunicorn>=22.0\n"),
        ("manage.py", "#!/usr/bin/env python\nimport os\nimport sys\n\ndef main():\n    os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'config.settings')\n    from django.core.management import execute_from_command_line\n    execute_from_command_line(sys.argv)\n\nif __name__ == '__main__':\n    main()\n"),
        ("config/__init__.py", ""),
        ("config/settings.py", &format!(r#"import os
from decouple import config
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SECRET_KEY = config('SECRET_KEY', default='django-insecure-dev-key-change-me')
DEBUG = config('DEBUG', default=True, cast=bool)
ALLOWED_HOSTS = config('ALLOWED_HOSTS', default='localhost,127.0.0.1').split(',')

INSTALLED_APPS = [
    'django.contrib.admin', 'django.contrib.auth',
    'django.contrib.contenttypes', 'django.contrib.sessions',
    'django.contrib.messages', 'django.contrib.staticfiles',
    'rest_framework', 'corsheaders',
    'apps.core', 'apps.api', 'apps.users',
]

MIDDLEWARE = [
    'corsheaders.middleware.CorsMiddleware',
    'django.middleware.security.SecurityMiddleware',
    'django.contrib.sessions.middleware.SessionMiddleware',
    'django.middleware.common.CommonMiddleware',
    'django.middleware.csrf.CsrfViewMiddleware',
    'django.contrib.auth.middleware.AuthenticationMiddleware',
    'django.contrib.messages.middleware.MessageMiddleware',
    'django.middleware.clickjacking.XFrameOptionsMiddleware',
]

ROOT_URLCONF = 'config.urls'
TEMPLATES = [{{'BACKEND': 'django.template.backends.django.DjangoTemplates','DIRS': [BASE_DIR / 'templates'],'APP_DIRS': True,'OPTIONS': {{'context_processors': ['django.template.context_processors.debug','django.template.context_processors.request','django.contrib.auth.context_processors.auth','django.contrib.messages.context_processors.messages'],}}}}]
DATABASES = {{'default': {{'ENGINE': 'django.db.backends.{}','NAME': config('DB_NAME', default='{}'),'USER': config('DB_USER', default='{}'),'PASSWORD': config('DB_PASSWORD', default=''),'HOST': config('DB_HOST', default='localhost'),'PORT': config('DB_PORT', default='5432'),}}}}
AUTH_PASSWORD_VALIDATORS = [{{'NAME': 'django.contrib.auth.password_validation.UserAttributeSimilarityValidator'}},{{'NAME': 'django.contrib.auth.password_validation.MinimumLengthValidator'}},{{'NAME': 'django.contrib.auth.password_validation.CommonPasswordValidator'}},{{'NAME': 'django.contrib.auth.password_validation.NumericPasswordValidator'}}]
LANGUAGE_CODE = 'en-us'
TIME_ZONE = 'UTC'
USE_I18N = True
USE_TZ = True
STATIC_URL = 'static/'
STATICFILES_DIRS = [BASE_DIR / 'static']
DEFAULT_AUTO_FIELD = 'django.db.models.BigAutoField'
CORS_ALLOW_ALL_ORIGINS = True
"#, "postgresql", name, name)),
        ("config/urls.py", "from django.contrib import admin\nfrom django.urls import path, include\n\nurlpatterns = [\n    path('admin/', admin.site.urls),\n    path('api/', include('apps.api.urls')),\n]\n"),
        ("config/wsgi.py", "import os\nfrom django.core.wsgi import get_wsgi_application\nos.environ.setdefault('DJANGO_SETTINGS_MODULE', 'config.settings')\napplication = get_wsgi_application()\n"),
        ("apps/core/__init__.py", ""),
        ("apps/core/models.py", "from django.db import models\n\nclass TimeStampedModel(models.Model):\n    created_at = models.DateTimeField(auto_now_add=True)\n    updated_at = models.DateTimeField(auto_now=True)\n\n    class Meta:\n        abstract = True\n"),
        ("apps/api/__init__.py", ""),
        ("apps/api/urls.py", "from django.urls import path, include\n\nurlpatterns = [\n    path('health/', include('apps.api.health.urls')),\n]\n"),
        ("apps/api/health/urls.py", "from django.urls import path\nfrom . import views\n\nurlpatterns = [\n    path('', views.health_check, name='health-check'),\n]\n"),
        ("apps/api/health/views.py", "from rest_framework.decorators import api_view\nfrom rest_framework.response import Response\n\n@api_view(['GET'])\ndef health_check(request):\n    return Response({'status': 'ok', 'version': '1.0.0'})\n"),
        ("apps/users/__init__.py", ""),
        ("apps/users/models.py", "from django.contrib.auth.models import AbstractUser\nfrom django.db import models\n\nclass User(AbstractUser):\n    bio = models.TextField(blank=True)\n    avatar = models.URLField(blank=True)\n\n    class Meta:\n        db_table = 'users'\n"),
        (".env", "SECRET_KEY=django-insecure-dev-key\nDEBUG=True\nDB_NAME=myapp\nDB_USER=postgres\nDB_PASSWORD=\nDB_HOST=localhost\nDB_PORT=5432\n"),
        ("Dockerfile", "FROM python:3.12-slim\nWORKDIR /app\nCOPY requirements.txt .\nRUN pip install -r requirements.txt\nCOPY . .\nCMD [\"gunicorn\", \"config.wsgi:application\", \"--bind\", \"0.0.0.0:8000\"]\n"),
        (".gitignore", "*.pyc\n__pycache__/\n.env\nvenv/\n*.sqlite3\n"),
    ])?;

    println!("✅ Django app at {}", project_dir.display());
    println!("\n  cd {}", project_dir.display());
    println!("  python -m venv venv && source venv/bin/activate");
    println!("  pip install -r requirements.txt");
    println!("  python manage.py migrate\n");
    Ok(())
}

pub(crate) fn scaffold_rails(name: &str, dir: &Path) -> anyhow::Result<()> {
    let project_dir = dir.join(name);
    if project_dir.exists() { anyhow::bail!("Directory exists: {}", project_dir.display()); }

    for d in &[
        "app/controllers", "app/models", "app/views/layouts", "app/views/pages",
        "app/views/posts", "config", "config/environments", "config/initializers",
        "db/migrate", "db", "public",
    ] {
        std::fs::create_dir_all(project_dir.join(d))?;
    }

    crate::write_files(&project_dir, vec![
        ("Gemfile", &format!(r#"source 'https://rubygems.org'
ruby '~> 3.3'
gem 'rails', '~> 7.2'
gem 'pg', '~> 1.5'
gem 'puma', '~> 6.4'
gem 'jbuilder', '~> 2.12'
gem 'bootsnap', require: false
gem 'rack-cors'
gem 'bcrypt', '~> 3.1'

group :development, :test do
  gem 'debug', platforms: %i[mri mingw x64_mingw]
  gem 'rspec-rails', '~> 7.0'
  gem 'factory_bot_rails'
end

group :development do
  gem 'rubocop', require: false
  gem 'rubocop-rails', require: false
end
"#)),
        ("config/application.rb", &format!(r#"require_relative 'boot'
require 'rails/all'

Bundler.require(*Rails.groups)

module {}
  class Application < Rails::Application
    config.load_defaults 7.2
    config.autoload_lib(ignore: %w[assets tasks])
    config.api_only = true
  end
end
"#, name.split('_').map(|w| w.chars().next().unwrap().to_uppercase().to_string() + &w[1..]).collect::<Vec<_>>().join(""))),
        ("config/boot.rb", "ENV['BUNDLE_GEMFILE'] ||= File.expand_path('../Gemfile', __dir__)\nrequire 'bundler/setup' # Set up gems listed in the Gemfile.\nrequire 'bootsnap/setup' # Speed up boot time by caching expensive operations.\n"),
        ("config/environment.rb", "require_relative 'application'\nRails.application.initialize!\n"),
        ("config/database.yml", &format!("default: &default\n  adapter: postgresql\n  encoding: unicode\n  pool: 5\n\ndevelopment:\n  <<: *default\n  database: {}\n\ntest:\n  <<: *default\n  database: {}_test\n\nproduction:\n  <<: *default\n  database: {}_production\n  username: {}\n  password: {{}}\n", name, name, name, name)),
        ("config/routes.rb", "Rails.application.routes.draw do\n  root 'pages#home'\n  resources :posts\n  get 'health', to: 'health#show'\nend\n"),
        ("config/puma.rb", "max_threads_count = ENV.fetch('RAILS_MAX_THREADS') { 5 }\nmin_threads_count = ENV.fetch('RAILS_MIN_THREADS') { max_threads_count }\nthreads min_threads_count, max_threads_count\nworker_timeout 3600 if ENV.fetch('RAILS_ENV', 'development') == 'development'\nport ENV.fetch('PORT') { 3000 }\nenvironment ENV.fetch('RAILS_ENV') { 'development' }\npids_dir ENV.fetch('PIDFILE') { 'tmp/pids' }\nplugin :tmp_restart\n"),
        ("config/environments/development.rb", "require 'active_support/core_ext/integer/time'\nRails.application.configure do\n  config.enable_reloading = true\n  config.eager_load = false\n  config.consider_all_requests_local = true\n  config.server_timing = true\n  config.active_support.deprecation = :log\nend\n"),
        ("config/environments/production.rb", "require 'active_support/core_ext/integer/time'\nRails.application.configure do\n  config.enable_reloading = false\n  config.eager_load = true\n  config.consider_all_requests_local = false\n  config.force_ssl = true\n  config.assets.compile = false\n  config.active_support.deprecation = :notify\nend\n"),
        ("config/environments/test.rb", "require 'active_support/core_ext/integer/time'\nRails.application.configure do\n  config.enable_reloading = false\n  config.eager_load = false\n  config.consider_all_requests_local = true\n  config.cache_classes = true\nend\n"),
        ("config/initializers/cors.rb", "Rails.application.config.middleware.insert_before 0, Rack::Cors do\n  allow do\n    origins '*'\n    resource '*', headers: :any, methods: [:get, :post, :put, :patch, :delete, :options, :head]\n  end\nend\n"),
        ("app/controllers/application_controller.rb", "class ApplicationController < ActionController::API\nend\n"),
        ("app/controllers/pages_controller.rb", "class PagesController < ApplicationController\n  def home\n    render json: { message: 'Hello from Rails!' }\n  end\nend\n"),
        ("app/controllers/health_controller.rb", "class HealthController < ApplicationController\n  def show\n    render json: { status: 'ok', timestamp: Time.current.iso8601 }\n  end\nend\n"),
        ("app/controllers/posts_controller.rb", "class PostsController < ApplicationController\n  before_action :set_post, only: [:show, :update, :destroy]\n\n  def index\n    posts = Post.all\n    render json: posts\n  end\n\n  def show\n    render json: @post\n  end\n\n  def create\n    post = Post.new(post_params)\n    if post.save\n      render json: post, status: :created\n    else\n      render json: post.errors, status: :unprocessable_entity\n    end\n  end\n\n  private\n\n  def set_post\n    @post = Post.find(params[:id])\n  end\n\n  def post_params\n    params.require(:post).permit(:title, :content)\n  end\nend\n"),
        ("app/models/application_record.rb", "class ApplicationRecord < ActiveRecord::Base\n  primary_abstract_class\nend\n"),
        ("app/models/post.rb", "class Post < ApplicationRecord\n  validates :title, presence: true\n  validates :content, presence: true\nend\n"),
        ("db/migrate/20240101000001_create_posts.rb", "class CreatePosts < ActiveRecord::Migration[7.2]\n  def change\n    create_table :posts do |t|\n      t.string :title, null: false\n      t.text :content, null: false\n      t.timestamps\n    end\n  end\nend\n"),
        ("db/seeds.rb", "puts 'Seeding...'\nPost.create!(title: 'Hello World', content: 'Welcome to Rails!')\nPost.create!(title: 'Getting Started', content: 'This is your first Rails app.')\nputs 'Done!'\n"),
        ("Rakefile", "require_relative 'config/application'\nRails.application.load_tasks\n"),
        (".env.example", "DATABASE_URL=postgres://postgres:postgres@localhost:5432/myapp\nSECRET_KEY_BASE=\n"),
        ("public/robots.txt", "User-agent: *\nDisallow:\n"),
    ])?;

    println!("✅ Rails app at {}", project_dir.display());
    println!("\n  cd {} && bundle install && rails db:create db:migrate db:seed", project_dir.display());
    Ok(())
}

pub(crate) fn scaffold_fastapi(name: &str, dir: &Path) -> anyhow::Result<()> {
    let project_dir = dir.join(name);
    if project_dir.exists() { anyhow::bail!("Directory exists: {}", project_dir.display()); }
    std::fs::create_dir_all(project_dir.join("app"))?;
    crate::write_files(&project_dir, vec![
        ("requirements.txt", "fastapi\nuvicorn[standard]\nsqlalchemy\npydantic\nalembic\n"),
        ("app/__init__.py", ""),
        ("app/main.py", &r#"from fastapi import FastAPI

app = FastAPI(title="NAME")

@app.get("/")
async def root():
    return {"message": "Hello from FastAPI!"}

@app.get("/health")
async def health():
    return {"status": "ok"}
"#.replace("NAME", name)),
        ("app/models.py", "from pydantic import BaseModel\n\nclass Item(BaseModel):\n    id: int\n    name: str\n    description: str | None = None\n"),
        ("Dockerfile", "FROM python:3.11-slim\nWORKDIR /app\nCOPY requirements.txt .\nRUN pip install -r requirements.txt\nCOPY . .\nCMD [\"uvicorn\", \"app.main:app\", \"--host\", \"0.0.0.0\", \"--port\", \"8000\"]\n"),
        (".env", "DATABASE_URL=sqlite:///./test.db\n"),
    ])?;
    println!("✅ FastAPI app at {}", project_dir.display());
    Ok(())
}

pub(crate) fn scaffold_flask(name: &str, dir: &Path) -> anyhow::Result<()> {
    let project_dir = dir.join(name);
    if project_dir.exists() { anyhow::bail!("Directory exists: {}", project_dir.display()); }
    std::fs::create_dir_all(project_dir.join("app"))?;
    crate::write_files(&project_dir, vec![
        ("requirements.txt", "flask\nsqlalchemy\nflask-sqlalchemy\npython-dotenv\n"),
        ("app/__init__.py", "from flask import Flask\n\ndef create_app():\n    app = Flask(__name__)\n    app.config.from_object('config')\n    from . import routes\n    app.register_blueprint(routes.bp)\n    return app\n"),
        ("app/routes.py", "from flask import Blueprint, jsonify\nbp = Blueprint('main', __name__)\n\n@bp.route('/')\ndef index():\n    return jsonify({\"message\": \"Hello from Flask!\"})\n"),
        ("run.py", "from app import create_app\napp = create_app()\nif __name__ == '__main__':\n    app.run(debug=True)\n"),
        ("config.py", "import os\nSECRET_KEY = os.environ.get('SECRET_KEY', 'dev-key')\nSQLALCHEMY_DATABASE_URI = os.environ.get('DATABASE_URL', 'sqlite:///app.db')\n"),
        (".env", "FLASK_APP=run.py\nFLASK_ENV=development\n"),
    ])?;
    println!("✅ Flask app at {}", project_dir.display());
    Ok(())
}

pub(crate) fn scaffold_go_gin(name: &str, dir: &Path) -> anyhow::Result<()> {
    let project_dir = dir.join(name);
    if project_dir.exists() { anyhow::bail!("Directory exists: {}", project_dir.display()); }
    std::fs::create_dir_all(project_dir.join("handlers"))?;
    crate::write_files(&project_dir, vec![
        ("go.mod", &format!("module {}\n\ngo 1.21\n", name)),
        ("main.go", "package main\n\nimport (\n\t\"net/http\"\n\t\"github.com/gin-gonic/gin\"\n)\n\nfunc main() {\n\tr := gin.Default()\n\tr.GET(\"/\", func(c *gin.Context) {\n\t\tc.JSON(http.StatusOK, gin.H{\"message\": \"Hello from Gin!\"})\n\t})\n\tr.Run(\":8080\")\n}\n"),
        ("handlers/hello.go", "package handlers\n\nimport (\n\t\"net/http\"\n\t\"github.com/gin-gonic/gin\"\n)\n\nfunc HelloHandler(c *gin.Context) {\n\tc.JSON(http.StatusOK, gin.H{\"message\": \"Hello!\"})\n}\n"),
    ])?;
    println!("✅ Go Gin app at {}", project_dir.display());
    println!("  cd {} && go mod tidy && go run .", project_dir.display());
    Ok(())
}

pub(crate) fn scaffold_go_fiber(name: &str, dir: &Path) -> anyhow::Result<()> {
    let project_dir = dir.join(name);
    if project_dir.exists() { anyhow::bail!("Directory exists: {}", project_dir.display()); }
    std::fs::create_dir_all(project_dir.join("handlers"))?;
    crate::write_files(&project_dir, vec![
        ("go.mod", &format!("module {}\n\ngo 1.21\n", name)),
        ("main.go", "package main\n\nimport (\n\t\"github.com/gofiber/fiber/v2\"\n)\n\nfunc main() {\n\tapp := fiber.New()\n\tapp.Get(\"/\", func(c *fiber.Ctx) error {\n\t\treturn c.JSON(fiber.Map{\"message\": \"Hello from Fiber!\"})\n\t})\n\tapp.Listen(\":3000\")\n}\n"),
    ])?;
    println!("✅ Go Fiber app at {}", project_dir.display());
    println!("  cd {} && go mod tidy && go run .", project_dir.display());
    Ok(())
}

pub(crate) fn scaffold_go_echo(name: &str, dir: &Path) -> anyhow::Result<()> {
    let project_dir = dir.join(name);
    if project_dir.exists() { anyhow::bail!("Directory exists: {}", project_dir.display()); }
    std::fs::create_dir_all(project_dir.join("handlers"))?;
    crate::write_files(&project_dir, vec![
        ("go.mod", &format!("module {}\n\ngo 1.21\n", name)),
        ("main.go", "package main\n\nimport (\n\t\"net/http\"\n\t\"github.com/labstack/echo/v4\"\n)\n\nfunc main() {\n\te := echo.New()\n\te.GET(\"/\", func(c echo.Context) error {\n\t\treturn c.String(http.StatusOK, \"Hello from Echo!\")\n\t})\n\te.Logger.Fatal(e.Start(\":8080\"))\n}\n"),
    ])?;
    println!("✅ Go Echo app at {}", project_dir.display());
    println!("  cd {} && go mod tidy && go run .", project_dir.display());
    Ok(())
}

pub(crate) fn scaffold_rust_project(name: &str, dir: &Path, kind: &str) -> anyhow::Result<()> {
    let project_dir = dir.join(name);
    if project_dir.exists() { anyhow::bail!("Directory exists: {}", project_dir.display()); }

    use crate::engines::RsEngine;
    let mut engine = RsEngine::new()?;
    let output = engine.scaffold(kind)?;

    if output.exit_code != 0 {
        anyhow::bail!("Scaffold failed: {}", output.stderr);
    }

    if let Ok(result_val) = serde_json::from_str::<serde_json::Value>(&output.result) {
        if let Some(files) = result_val.get("files").and_then(|f| f.as_array()) {
            for file in files {
                let name = file["name"].as_str().unwrap_or("");
                let content = file["content"].as_str().unwrap_or("");
                if !name.is_empty() {
                    let path = project_dir.join(name);
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, content)?;
                }
            }
        }
    } else {
        for d in &["src"] { std::fs::create_dir_all(project_dir.join(d))?; }

        match kind {
            "actix-web" => {
                crate::write_files(&project_dir, vec![
                    ("Cargo.toml", r#"[package]
name = "actix-app"
version = "0.1.0"
edition = "2021"
[dependencies]
actix-web = "4"
"#),
                    ("src/main.rs", r#"use actix_web::{get, App, HttpResponse, HttpServer, Responder};
#[get("/")]
async fn hello() -> impl Responder { HttpResponse::Ok().body("Hello, world!") }
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello)).bind(("127.0.0.1", 8080))?.run().await
}"#),
                ])?;
            }
            "axum" => {
                crate::write_files(&project_dir, vec![
                    ("Cargo.toml", r#"[package]
name = "axum-app"
version = "0.1.0"
edition = "2021"
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
"#),
                    ("src/main.rs", r#"use axum::{Router, routing::get, response::Html};
async fn hello() -> Html<&'static str> { Html("<h1>Hello!</h1>") }
#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}"#),
                ])?;
            }
            "rocket" => {
                crate::write_files(&project_dir, vec![
                    ("Cargo.toml", r#"[package]
name = "rocket-app"
version = "0.1.0"
edition = "2021"
[dependencies]
rocket = "0.5"
"#),
                    ("src/main.rs", r#"#[macro_use] extern crate rocket;
#[get("/")]
fn index() -> &'static str { "Hello, world!" }
#[launch]
fn rocket() -> _ { rocket::build().mount("/", routes![index]) }
"#),
                ])?;
            }
            _ => anyhow::bail!("Unknown scaffold type: {kind}"),
        }
    }

    println!("✅ {} ({}) at {}", name, kind, project_dir.display());
    println!("\n  cd {}", project_dir.display());
    println!("  cargo run\n");
    Ok(())
}
