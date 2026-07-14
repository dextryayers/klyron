use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct LaravelAdapter;

#[async_trait]
impl FrameworkAdapter for LaravelAdapter {
    fn name(&self) -> &'static str { "laravel" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("artisan").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["11.0", "12.0"] }
    fn default_version(&self) -> &'static str { "11.0" }
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
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("app/Http/Controllers"))?;
        std::fs::create_dir_all(project_dir.join("app/Http/Livewire"))?;
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
        std::fs::create_dir_all(project_dir.join("resources/views/livewire"))?;
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
            klyron_template::TemplateEngine::render(r#"{
  "name": "app/{{ name }}",
  "type": "project",
  "require": {
    "php": "^8.2",
    "laravel/framework": "^11.0",
    "laravel/sanctum": "^4.0",
    "laravel/telescope": "^5.0",
    "spatie/laravel-permission": "^6.0",
    "livewire/livewire": "^3.0"
  },
  "require-dev": {
    "laravel/sail": "^1.0",
    "fakerphp/faker": "^1.23",
    "mockery/mockery": "^1.6",
    "nunomaduro/collision": "^8.0",
    "phpunit/phpunit": "^11.0",
    "laravel/pint": "^1.0"
  },
  "autoload": {
    "psr-4": { "App\\": "app/", "Database\\Factories\\": "database/factories/", "Database\\Seeders\\": "database/seeders/" }
  },
  "scripts": {
    "post-autoload-dump": ["Illuminate\\Foundation\\ComposerScripts::postAutoloadDump", "@php artisan package:discover --ansi"],
    "post-update-cmd": ["@php artisan vendor:publish --tag=laravel-assets --ansi --force"],
    "post-root-package-install": ["@php -r \"file_exists('.env') || copy('.env.example', '.env');\""],
    "post-create-project-cmd": ["@php artisan key:generate --ansi"]
  },
  "extra": { "laravel": { "dont-discover": [] } }
}"#, vars))?;

        std::fs::write(project_dir.join("package.json"),
            r#"{
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "format": "prettier --write ."
  },
  "devDependencies": {
    "vite": "^6.0.0",
    "laravel-vite-plugin": "^1.0.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0",
    "prettier": "^3.4.0"
  }
}"#)?;

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

        std::fs::write(project_dir.join("bootstrap/app.php"),
            r#"<?php
return Illuminate\Foundation\Application::configure(basePath: dirname(__DIR__))
    ->withRouting(web: __DIR__.'/../routes/web.php', api: __DIR__.'/../routes/api.php')
    ->withMiddleware(function ($middleware) {})
    ->withExceptions(function ($exceptions) {})->create();
"#)?;

        std::fs::write(project_dir.join("config/app.php"),
            r#"<?php return ['name' => env('APP_NAME', 'Laravel'), 'env' => env('APP_ENV', 'production'), 'debug' => (bool) env('APP_DEBUG', false), 'url' => env('APP_URL', 'http://localhost'), 'timezone' => 'UTC', 'locale' => 'en']; "#)?;

        std::fs::write(project_dir.join("config/cache.php"),
            r#"<?php return ['default' => env('CACHE_STORE', 'file'), 'stores' => ['file' => ['driver' => 'file', 'path' => storage_path('framework/cache/data')]]]; "#)?;

        std::fs::write(project_dir.join("config/database.php"),
            r#"<?php return ['default' => env('DB_CONNECTION', 'mysql'), 'connections' => ['mysql' => ['driver' => 'mysql', 'host' => env('DB_HOST', '127.0.0.1'), 'port' => env('DB_PORT', '3306'), 'database' => env('DB_DATABASE', 'laravel'), 'username' => env('DB_USERNAME', 'root'), 'password' => env('DB_PASSWORD', '')]]]; "#)?;

        std::fs::write(project_dir.join("config/sanctum.php"), r#"<?php return ['stateful' => explode(',', env('SANCTUM_STATEFUL_DOMAINS', 'localhost,localhost:3000'))]; "#)?;

        std::fs::write(project_dir.join("config/permission.php"),
            r#"<?php return ['models' => ['permission' => Spatie\Permission\Models\Permission::class, 'role' => Spatie\Permission\Models\Role::class]]; "#)?;

        std::fs::write(project_dir.join("config/telescope.php"),
            r#"<?php return ['enabled' => env('TELESCOPE_ENABLED', true)]; "#)?;

        std::fs::write(project_dir.join("app/Http/Controllers/Controller.php"),
            r#"<?php namespace App\Http\Controllers; abstract class Controller extends \Illuminate\Routing\Controller {}"#)?;

        std::fs::write(project_dir.join("app/Http/Controllers/AuthController.php"),
            r#"<?php namespace App\Http\Controllers; use Illuminate\Http\Request; class AuthController extends Controller { public function login(Request $req) {} public function register(Request $req) {} }"#)?;

        std::fs::write(project_dir.join("app/Http/Controllers/DashboardController.php"),
            r#"<?php namespace App\Http\Controllers; class DashboardController extends Controller { public function index() { return view('dashboard'); } }"#)?;

        std::fs::write(project_dir.join("app/Http/Controllers/PostController.php"),
            r#"<?php namespace App\Http\Controllers; use App\Models\Post; use Illuminate\Http\Request; class PostController extends Controller { public function index() { return Post::all(); } public function store(Request $r) { return Post::create($r->all()); } }"#)?;

        std::fs::write(project_dir.join("app/Http/Livewire/Counter.php"),
            r#"<?php namespace App\Http\Livewire; use Livewire\Component; class Counter extends Component { public $count = 0; public function increment() { $this->count++; } public function render() { return view('livewire.counter'); } }"#)?;

        std::fs::write(project_dir.join("app/Models/User.php"),
            r#"<?php namespace App\Models; use Illuminate\Foundation\Auth\User as Authenticatable; use Laravel\Sanctum\HasApiTokens; use Spatie\Permission\Traits\HasRoles; class User extends Authenticatable { use HasApiTokens, HasRoles; protected $fillable = ['name', 'email', 'password']; protected $hidden = ['password', 'remember_token']; }"#)?;

        std::fs::write(project_dir.join("app/Models/Post.php"),
            r#"<?php namespace App\Models; use Illuminate\Database\Eloquent\Model; class Post extends Model { protected $fillable = ['title', 'content', 'user_id']; public function user() { return $this->belongsTo(User::class); } }"#)?;

        std::fs::write(project_dir.join("app/Providers/AppServiceProvider.php"),
            r#"<?php namespace App\Providers; use Illuminate\Support\ServiceProvider; class AppServiceProvider extends ServiceProvider { public function register() {} public function boot() {} }"#)?;

        std::fs::write(project_dir.join("app/Providers/TelescopeServiceProvider.php"),
            r#"<?php namespace App\Providers; use Laravel\Telescope\TelescopeServiceProvider as Base; class TelescopeServiceProvider extends Base { public function register() { $this->hideSensitiveRequestDetails(); parent::register(); } }"#)?;

        std::fs::write(project_dir.join("app/Http/Middleware/AdminMiddleware.php"),
            r#"<?php namespace App\Http\Middleware; use Closure; use Illuminate\Http\Request; class AdminMiddleware { public function handle(Request $request, Closure $next) { if (!$request->user()?->hasRole('admin')) abort(403); return $next($request); } }"#)?;

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

        std::fs::write(project_dir.join("resources/views/dashboard.blade.php"),
            r#"@extends('layouts.app') @section('content')<h1>Dashboard</h1>@endsection"#)?;

        std::fs::write(project_dir.join("resources/views/auth/login.blade.php"),
            r#"@extends('layouts.app') @section('content')<form method="POST" action="/login">@csrf<input type="email" name="email" placeholder="Email"><input type="password" name="password" placeholder="Password"><button type="submit">Login</button></form>@endsection"#)?;

        std::fs::write(project_dir.join("resources/views/livewire/counter.blade.php"),
            r#"<div><h1>{{ $count }}</h1><button wire:click="increment">+</button></div>"#)?;

        std::fs::write(project_dir.join("resources/css/app.css"), r#"@tailwind base; @tailwind components; @tailwind utilities;"#)?;

        std::fs::write(project_dir.join("routes/web.php"), r#"<?php use Illuminate\Support\Facades\Route; Route::get('/', function () { return view('welcome'); });"#)?;

        std::fs::write(project_dir.join("routes/api.php"), r#"<?php use Illuminate\Support\Facades\Route; Route::get('/user', function (Request $request) { return $request->user(); })->middleware('auth:sanctum');"#)?;

        std::fs::write(project_dir.join("public/index.php"),
            r#"<?php use Illuminate\Http\Request; define('LARAVEL_START', microtime(true)); require __DIR__.'/../vendor/autoload.php'; $app = require_once __DIR__.'/../bootstrap/app.php'; $kernel = $app->make(Illuminate\Contracts\Http\Kernel::class); $response = $kernel->handle($request = Request::capture()); $response->send(); $kernel->terminate($request, $response);"#)?;

        std::fs::write(project_dir.join("public/.htaccess"),
            r#"<IfModule mod_rewrite.c><IfModule mod_negotiation.c>Options -MultiViews -Indexes</IfModule>RewriteEngine On RewriteCond %{REQUEST_FILENAME} !-d RewriteCond %{REQUEST_FILENAME} !-f RewriteRule ^ index.php [L]</IfModule>"#)?;

        std::fs::write(project_dir.join("storage/logs/.gitkeep"), "")?;

        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render(r#"# {{ name }}

Laravel application

## Getting Started

composer install
npm install
php artisan serve
"#, vars))?;

        Ok(())
    }
}
