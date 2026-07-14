use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct LaravelAdapter;

fn version_composer(version: &str) -> (&'static str, &'static str, &'static str) {
    match version {
        "9" => ("^9.52", "^8.0", "^9.0"),
        "10" => ("^10.48", "^8.1", "^10.0"),
        "11" => ("^11.31", "^8.2", "^11.0"),
        "12" => ("^12.0", "^8.2", "^12.0"),
        "13" => ("^13.0", "^8.3", "^13.0"),
        _ => ("^11.31", "^8.2", "^11.0"),
    }
}

fn version_sanctum(version: &str) -> &'static str {
    match version { "9" => "^3.0", "10" => "^3.0", _ => "^4.0" }
}

fn version_livewire(version: &str) -> &'static str {
    match version { "9" => "^2.0", "10" => "^3.0", _ => "^3.5" }
}

fn version_collision(version: &str) -> &'static str {
    match version { "9" => "^6.0", "10" => "^7.0", "11" => "^8.0", _ => "^9.0" }
}

fn simple_bootstrap(version: &str) -> bool {
    matches!(version, "11" | "12" | "13")
}

fn is_lts(version: &str) -> bool {
    matches!(version, "9" | "11")
}

pub fn detect_laravel_version(dir: &Path) -> Option<&'static str> {
    let composer_path = dir.join("composer.json");
    let content = std::fs::read_to_string(composer_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let require = json.get("require")?;
    let framework = require.get("laravel/framework")?.as_str()?;
    if framework.contains("^9") || framework.contains("~9") {
        Some("9")
    } else if framework.contains("^10") || framework.contains("~10") {
        Some("10")
    } else if framework.contains("^11") || framework.contains("~11") {
        Some("11")
    } else if framework.contains("^12") || framework.contains("~12") {
        Some("12")
    } else if framework.contains("^13") || framework.contains("~13") {
        Some("13")
    } else {
        None
    }
}

#[async_trait]
impl FrameworkAdapter for LaravelAdapter {
    fn name(&self) -> &'static str { "laravel" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("artisan").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["9", "10", "11", "12", "13"] }
    fn default_version(&self) -> &'static str { "11" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Polyglot }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("php");
        cmd.args(["artisan", "serve"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("php").args(["artisan", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("php").args(["./vendor/bin/phpunit"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("./vendor/bin/pint").current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("./vendor/bin/pint").current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("./vendor/bin/pint").arg("--test").current_dir(dir).status().await?;
        }
        Ok(())
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let version = options.version.as_deref().unwrap_or("11");
        let (fw_dep, php_req, phpunit_ver) = version_composer(version);
        let sanctum_dep = version_sanctum(version);
        let livewire_dep = version_livewire(version);
        let collision_dep = version_collision(version);
        let stack = options.template_vars.get("stack").map(|s| s.as_str()).unwrap_or("blade");

        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("app/Http/Controllers"))?;
        std::fs::create_dir_all(project_dir.join("app/Http/Middleware"))?;
        std::fs::create_dir_all(project_dir.join("app/Models"))?;
        std::fs::create_dir_all(project_dir.join("app/Providers"))?;
        std::fs::create_dir_all(project_dir.join("bootstrap"))?;
        std::fs::create_dir_all(project_dir.join("config"))?;
        std::fs::create_dir_all(project_dir.join("database/migrations"))?;
        std::fs::create_dir_all(project_dir.join("database/seeders"))?;
        std::fs::create_dir_all(project_dir.join("database/factories"))?;
        std::fs::create_dir_all(project_dir.join("resources/views/layouts"))?;
        std::fs::create_dir_all(project_dir.join("resources/views/auth"))?;
        std::fs::create_dir_all(project_dir.join("resources/css"))?;
        std::fs::create_dir_all(project_dir.join("routes"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;
        std::fs::create_dir_all(project_dir.join("storage/logs"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("artisan"), r#"#!/usr/bin/env php
<?php
define('LARAVEL_START', microtime(true));
require __DIR__.'/vendor/autoload.php';
$app = require_once __DIR__.'/bootstrap/app.php';
$kernel = $app->make(Illuminate\Contracts\Console\Kernel::class);
$status = $kernel->handle($input = Symfony\Component\Console\Input\ArgvInput::new(), new Symfony\Component\Console\Output\ConsoleOutput());
$kernel->terminate($input, $status);
exit($status);
"#)?;

        std::fs::write(project_dir.join("composer.json"),
            klyron_template::TemplateEngine::render(
                &format!(r#"{{
  "name": "app/{{ name }}",
  "type": "project",
  "require": {{
    "php": "{}",
    "laravel/framework": "{}",
    "laravel/sanctum": "{}",
    "spatie/laravel-permission": "^6.0",
    "livewire/livewire": "{}"
  }},
  "require-dev": {{
    "laravel/sail": "^1.0",
    "fakerphp/faker": "^1.23",
    "mockery/mockery": "^1.6",
    "nunomaduro/collision": "{}",
    "phpunit/phpunit": "{}",
    "laravel/pint": "^1.0"
  }},
  "autoload": {{
    "psr-4": {{ "App\\": "app/", "Database\\Factories\\": "database/factories/", "Database\\Seeders\\": "database/seeders/" }}
  }},
  "scripts": {{
    "post-autoload-dump": ["Illuminate\\Foundation\\ComposerScripts::postAutoloadDump", "@php artisan package:discover --ansi"],
    "post-update-cmd": ["@php artisan vendor:publish --tag=laravel-assets --ansi --force"],
    "post-root-package-install": ["@php -r \"file_exists('.env') || copy('.env.example', '.env');\""],
    "post-create-project-cmd": ["@php artisan key:generate --ansi"]
  }},
  "extra": {{ "laravel": {{ "dont-discover": [] }} }}
}}"#, php_req, fw_dep, sanctum_dep, livewire_dep, collision_dep, phpunit_ver),
            vars))?;

        std::fs::write(project_dir.join(".env.example"),
            r#"APP_NAME=Laravel
APP_ENV=local
APP_DEBUG=true
APP_URL=http://localhost
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=laravel
DB_USERNAME=root
DB_PASSWORD=
"#)?;

        std::fs::write(project_dir.join(".gitignore"),
            "/node_modules\n/vendor\n.env\n.DS_Store\n/storage/*.log\n")?;

        if simple_bootstrap(version) {
            std::fs::write(project_dir.join("bootstrap/app.php"),
                r#"<?php
return Illuminate\Foundation\Application::configure(basePath: dirname(__DIR__))
    ->withRouting(web: __DIR__.'/../routes/web.php', api: __DIR__.'/../routes/api.php')
    ->withMiddleware(function ($middleware) {})
    ->withExceptions(function ($exceptions) {})->create();
"#)?;
        } else {
            std::fs::write(project_dir.join("bootstrap/app.php"),
                r#"<?php
$app = new Illuminate\Foundation\Application($_ENV['APP_BASE_PATH'] ?? dirname(__DIR__));
$app->singleton(Illuminate\Contracts\Http\Kernel::class, App\Http\Kernel::class);
$app->singleton(Illuminate\Contracts\Console\Kernel::class, App\Console\Kernel::class);
$app->singleton(Illuminate\Contracts\Debug\ExceptionHandler::class, App\Exceptions\Handler::class);
return $app;
"#)?;
        }

        std::fs::write(project_dir.join("config/app.php"),
            r#"<?php return ['name' => env('APP_NAME', 'Laravel'), 'env' => env('APP_ENV', 'production'), 'debug' => (bool) env('APP_DEBUG', false), 'url' => env('APP_URL', 'http://localhost'), 'timezone' => 'UTC', 'locale' => 'en']; "#)?;

        std::fs::write(project_dir.join("config/cache.php"),
            r#"<?php return ['default' => env('CACHE_STORE', 'file'), 'stores' => ['file' => ['driver' => 'file', 'path' => storage_path('framework/cache/data')]]]; "#)?;

        std::fs::write(project_dir.join("config/database.php"),
            r#"<?php return ['default' => env('DB_CONNECTION', 'mysql'), 'connections' => ['mysql' => ['driver' => 'mysql', 'host' => env('DB_HOST', '127.0.0.1'), 'port' => env('DB_PORT', '3306'), 'database' => env('DB_DATABASE', 'laravel'), 'username' => env('DB_USERNAME', 'root'), 'password' => env('DB_PASSWORD', '')]]]; "#)?;

        std::fs::write(project_dir.join("config/sanctum.php"), r#"<?php return ['stateful' => explode(',', env('SANCTUM_STATEFUL_DOMAINS', 'localhost,localhost:3000'))]; "#)?;

        std::fs::write(project_dir.join("config/permission.php"),
            r#"<?php return ['models' => ['permission' => Spatie\Permission\Models\Permission::class, 'role' => Spatie\Permission\Models\Role::class]]; "#)?;

        std::fs::write(project_dir.join("app/Http/Controllers/Controller.php"),
            r#"<?php namespace App\Http\Controllers; abstract class Controller extends \Illuminate\Routing\Controller {}"#)?;

        std::fs::write(project_dir.join("app/Http/Controllers/AuthController.php"),
            r#"<?php namespace App\Http\Controllers; use Illuminate\Http\Request; class AuthController extends Controller { public function login(Request $req) {} public function register(Request $req) {} }"#)?;

        std::fs::write(project_dir.join("app/Models/User.php"),
            r#"<?php namespace App\Models; use Illuminate\Foundation\Auth\User as Authenticatable; use Laravel\Sanctum\HasApiTokens; use Spatie\Permission\Traits\HasRoles; class User extends Authenticatable { use HasApiTokens, HasRoles; protected $fillable = ['name', 'email', 'password']; protected $hidden = ['password', 'remember_token']; }"#)?;

        std::fs::write(project_dir.join("app/Providers/AppServiceProvider.php"),
            r#"<?php namespace App\Providers; use Illuminate\Support\ServiceProvider; class AppServiceProvider extends ServiceProvider { public function register() {} public function boot() {} }"#)?;

        std::fs::write(project_dir.join("database/migrations/2014_10_12_000000_create_users_table.php"),
            r#"<?php use Illuminate\Database\Migrations\Migration; use Illuminate\Database\Schema\Blueprint; use Illuminate\Support\Facades\Schema; return new class extends Migration { public function up() { Schema::create('users', function (Blueprint $t) { $t->id(); $t->string('name'); $t->string('email')->unique(); $t->timestamp('email_verified_at')->nullable(); $t->string('password'); $t->rememberToken(); $t->timestamps(); }); } public function down() { Schema::dropIfExists('users'); } };"#)?;

        std::fs::write(project_dir.join("database/migrations/2014_10_12_100000_create_password_resets_table.php"),
            r#"<?php use Illuminate\Database\Migrations\Migration; use Illuminate\Database\Schema\Blueprint; use Illuminate\Support\Facades\Schema; return new class extends Migration { public function up() { Schema::create('password_reset_tokens', function (Blueprint $t) { $t->string('email')->primary(); $t->string('token'); $t->timestamp('created_at')->nullable(); }); } public function down() { Schema::dropIfExists('password_reset_tokens'); } };"#)?;

        std::fs::write(project_dir.join("database/migrations/2024_01_01_000001_create_posts_table.php"),
            r#"<?php use Illuminate\Database\Migrations\Migration; use Illuminate\Database\Schema\Blueprint; use Illuminate\Support\Facades\Schema; return new class extends Migration { public function up() { Schema::create('posts', function (Blueprint $t) { $t->id(); $t->string('title'); $t->text('content'); $t->foreignId('user_id')->constrained()->cascadeOnDelete(); $t->timestamps(); }); } public function down() { Schema::dropIfExists('posts'); } };"#)?;

        std::fs::write(project_dir.join("database/seeders/DatabaseSeeder.php"),
            r#"<?php namespace Database\Seeders; use Illuminate\Database\Seeder; class DatabaseSeeder extends Seeder { public function run() { $this->call([UserSeeder::class]); } }"#)?;

        std::fs::write(project_dir.join("database/seeders/UserSeeder.php"),
            r#"<?php namespace Database\Seeders; use App\Models\User; use Illuminate\Database\Seeder; class UserSeeder extends Seeder { public function run() { User::factory(10)->create(); } }"#)?;

        std::fs::write(project_dir.join("database/factories/UserFactory.php"),
            r#"<?php namespace Database\Factories; use Illuminate\Database\Eloquent\Factories\Factory; use Illuminate\Support\Facades\Hash; use Illuminate\Support\Str; class UserFactory extends Factory { protected $model = \App\Models\User::class; public function definition() { return ['name' => fake()->name(), 'email' => fake()->unique()->safeEmail(), 'email_verified_at' => now(), 'password' => Hash::make('password'), 'remember_token' => Str::random(10)]; } public function unverified() { return $this->state(fn (array $a) => ['email_verified_at' => null]); } }"#)?;

        std::fs::write(project_dir.join("database/factories/PostFactory.php"),
            r#"<?php namespace Database\Factories; use Illuminate\Database\Eloquent\Factories\Factory; class PostFactory extends Factory { protected $model = \App\Models\Post::class; public function definition() { return ['title' => fake()->sentence(), 'content' => fake()->paragraph(), 'user_id' => \App\Models\User::factory()]; } }"#)?;

        std::fs::write(project_dir.join("resources/views/layouts/app.blade.php"),
            klyron_template::TemplateEngine::render(r#"<!doctype html>
<html>
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>{{ name }}</title>@vite('resources/css/app.css')</head>
<body><div id="app">@yield('content')</div></body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("resources/views/welcome.blade.php"),
            klyron_template::TemplateEngine::render(r#"@extends('layouts.app') @section('content')<h1>Welcome to {{ name }}</h1>@endsection"#, vars))?;

        std::fs::write(project_dir.join("resources/css/app.css"), r#"@tailwind base; @tailwind components; @tailwind utilities;"#)?;

        std::fs::write(project_dir.join("routes/web.php"), r#"<?php use Illuminate\Support\Facades\Route; Route::get('/', function () { return view('welcome'); });"#)?;

        std::fs::write(project_dir.join("routes/api.php"), r#"<?php use Illuminate\Support\Facades\Route; Route::get('/user', function (Request $request) { return $request->user(); })->middleware('auth:sanctum');"#)?;

        std::fs::write(project_dir.join("public/index.php"),
            r#"<?php use Illuminate\Http\Request; define('LARAVEL_START', microtime(true)); require __DIR__.'/../vendor/autoload.php'; $app = require_once __DIR__.'/../bootstrap/app.php'; $kernel = $app->make(Illuminate\Contracts\Http\Kernel::class); $response = $kernel->handle($request = Request::capture()); $response->send(); $kernel->terminate($request, $response);"#)?;

        std::fs::write(project_dir.join("public/.htaccess"),
            r#"<IfModule mod_rewrite.c><IfModule mod_negotiation.c>Options -MultiViews -Indexes</IfModule>RewriteEngine On RewriteCond %{REQUEST_FILENAME} !-d RewriteCond %{REQUEST_FILENAME} !-f RewriteRule ^ index.php [L]</IfModule>"#)?;

        std::fs::write(project_dir.join("storage/logs/.gitkeep"), "")?;

        std::fs::write(project_dir.join("vite.config.js"),
            r#"import { defineConfig } from 'vite'
import laravel from 'laravel-vite-plugin'

export default defineConfig({
  plugins: [laravel({ input: ['resources/css/app.css'], refresh: true })],
})"#)?;

        std::fs::write(project_dir.join("tailwind.config.js"),
            r#"export default { content: ['./resources/**/*.blade.php', './resources/**/*.js'], theme: { extend: {} }, plugins: [] }"#)?;

        std::fs::write(project_dir.join("postcss.config.js"),
            r#"export default { plugins: { tailwindcss: {}, autoprefixer: {} } }"#)?;

        std::fs::write(project_dir.join("phpunit.xml"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<phpunit xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="vendor/phpunit/phpunit/phpunit.xsd" bootstrap="vendor/autoload.php" colors="true">
  <testsuites><testsuite name="Unit"><directory>tests/Unit</directory></testsuite><testsuite name="Feature"><directory>tests/Feature</directory></testsuite></testsuites>
  <php><env name="APP_ENV" value="testing"/><env name="BCRYPT_ROUNDS" value="4"/><env name="CACHE_DRIVER" value="array"/><env name="DB_CONNECTION" value="sqlite"/><env name="DB_DATABASE" value=":memory:"/></php>
</phpunit>"#)?;

        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render(r#"# {{ name }}

Laravel {{ version }} application

## Getting Started

composer install
npm install
php artisan serve
"#, &{let mut v = vars.clone(); v.insert("version".to_string(), version.to_string()); v}))?;

        match stack {
            "blade" => {}
            "react" | "vue" | "inertia-react" | "inertia-vue" | "livewire" | "next" | "astro" | "api" => {
                self.scaffold_stack(&project_dir, stack, vars)?;
            }
            _ => anyhow::bail!("Unknown Laravel stack: {stack}"),
        }

        println!("Laravel {} app created: {} (stack: {})", version, project_dir.display(), stack);
        Ok(())
    }
}

impl LaravelAdapter {
    fn scaffold_stack(&self, project_dir: &Path, stack: &str, _vars: &HashMap<String, String>) -> Result<()> {
        match stack {
            "react" => {
                std::fs::create_dir_all(project_dir.join("resources/js"))?;
                for (path, content) in [
                    ("resources/js/app.jsx", "import React from 'react'\nimport { createRoot } from 'react-dom/client'\nimport App from './App'\n\ncreateRoot(document.getElementById('app')).render(<App />)\n"),
                    ("resources/js/App.jsx", "import React from 'react'\n\nexport default function App() {\n  return <h1>Laravel + React</h1>\n}\n"),
                    ("package.json", r#"{"private":true,"scripts":{"dev":"vite","build":"vite build"},"dependencies":{"react":"^18.3","react-dom":"^18.3"},"devDependencies":{"@vitejs/plugin-react":"^4.3","vite":"^5.4","laravel-vite-plugin":"^1.0"}}"#),
                ] {
                    std::fs::write(project_dir.join(path), content)?;
                }
            }
            "vue" => {
                std::fs::create_dir_all(project_dir.join("resources/js"))?;
                for (path, content) in [
                    ("resources/js/app.js", "import { createApp } from 'vue'\nimport App from './App.vue'\n\ncreateApp(App).mount('#app')\n"),
                    ("resources/js/App.vue", "<template>\n  <h1>Laravel + Vue</h1>\n</template>\n\n<script setup>\n</script>\n"),
                    ("package.json", r#"{"private":true,"scripts":{"dev":"vite","build":"vite build"},"dependencies":{"vue":"^3.5"},"devDependencies":{"@vitejs/plugin-vue":"^5.1","vite":"^5.4","laravel-vite-plugin":"^1.0"}}"#),
                ] {
                    std::fs::write(project_dir.join(path), content)?;
                }
            }
            "inertia-react" => {
                std::fs::create_dir_all(project_dir.join("resources/js"))?;
                std::fs::write(project_dir.join("resources/js/app.jsx"),
                    "import { createInertiaApp } from '@inertiajs/react'\nimport { createRoot } from 'react-dom/client'\n\ncreateInertiaApp({\n  resolve: name => import(`./Pages/${name}.jsx`),\n  setup({ el, App, props }) { createRoot(el).render(<App {...props} />) },\n})\n")?;
            }
            "inertia-vue" => {
                std::fs::create_dir_all(project_dir.join("resources/js"))?;
                std::fs::write(project_dir.join("resources/js/app.js"),
                    "import { createInertiaApp } from '@inertiajs/vue3'\nimport { createApp } from 'vue'\n\ncreateInertiaApp({\n  resolve: name => import(`./Pages/${name}.vue`),\n  setup({ el, App, props }) { createApp(App, props).mount(el) },\n})\n")?;
            }
            "livewire" => {
                std::fs::create_dir_all(project_dir.join("app/Http/Livewire"))?;
                std::fs::create_dir_all(project_dir.join("resources/views/livewire"))?;
                std::fs::write(project_dir.join("app/Http/Livewire/Counter.php"),
                    r#"<?php namespace App\Http\Livewire; use Livewire\Component; class Counter extends Component { public $count = 0; public function increment() { $this->count++; } public function render() { return view('livewire.counter'); } }"#)?;
                std::fs::write(project_dir.join("resources/views/livewire/counter.blade.php"),
                    r#"<div><h1>{{ $count }}</h1><button wire:click="increment">+</button></div>"#)?;
                std::fs::write(project_dir.join("resources/views/layouts/app.blade.php"),
                    "<!DOCTYPE html>\n<html>\n<head><title>{{ config('app.name') }}</title>@livewireStyles</head>\n<body>\n  {{ $slot }}\n  @livewireScripts\n</body>\n</html>\n")?;
            }
            "next" => {
                std::fs::create_dir_all(project_dir.join("next-app"))?;
                std::fs::write(project_dir.join("next-app/package.json"),
                    r#"{"private":true,"scripts":{"dev":"next dev","build":"next build"},"dependencies":{"next":"^15.0","react":"^19.0","react-dom":"^19.0"}}"#)?;
            }
            "astro" => {
                std::fs::create_dir_all(project_dir.join("astro-app"))?;
            }
            "api" => {}
            _ => anyhow::bail!("Unknown Laravel stack: {stack}"),
        }
        Ok(())
    }
}
