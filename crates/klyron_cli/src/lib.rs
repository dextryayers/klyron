//! Klyron - Universal Polyglot Runtime CLI
//!
//! `klyron_cli` is the command-line interface for the Klyron runtime.
//! It provides commands for running JavaScript/TypeScript, managing packages,
//! scaffolding projects, database management, testing, deployment, and more.
//!
//! ## Quick Start
//!
//! ```bash
//! klyron run app.ts
//! klyron create next my-app
//! klyron install react
//! klyron dev
//! ```
//!
//! ## Architecture
//!
//! The CLI dispatches to sub-commands via the [`Commands`] enum. Each command
//! module under [`commands`] implements the actual logic. Engine operations
//! use the [`klyron_engine`] crate for JavaScript execution.

pub mod engines;
pub mod commands;
pub mod scaffold_inline;
pub mod color;
pub mod splash;
pub mod anim;
pub mod signal;

pub(crate) use commands::helpers::*;
pub(crate) use scaffold_inline::*;
pub(crate) use color::{Color, style_success};

use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand, Args, CommandFactory};
use klyron_adapter::AdapterRegistry;
use klyron_adapter::adapters::register_all;
use klyron_core::Runtime;
use klyron_engine::{JsEngineKind, EngineRuntime, detect_best_engine, benchmark_all_engines, EnginePreWarmer, profile_all_engines};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tracing_subscriber::EnvFilter;

// ── Engine helpers ────────────────────────────────────────────────────────

pub fn resolve_engine(name: Option<&str>) -> anyhow::Result<Option<EngineRuntime>> {
    let name = match name {
        Some(n) => n,
        None => return Ok(None),
    };
    match name.to_lowercase().as_str() {
        "auto" => {
            let kind = detect_best_engine();
            log_info(format!("Detected best engine: {kind}"));
            Ok(Some(EngineRuntime::new(kind).map_err(|e| anyhow::anyhow!("Engine {kind}: {e}"))?))
        }
        "v8" => Ok(Some(EngineRuntime::new(JsEngineKind::V8).map_err(|e| anyhow::anyhow!("V8 engine: {e}"))?)),
        "boa" => Ok(Some(EngineRuntime::new(JsEngineKind::Boa).map_err(|e| anyhow::anyhow!("Boa engine: {e}"))?)),
        "quickjs" => Ok(Some(EngineRuntime::new(JsEngineKind::QuickJS).map_err(|e| anyhow::anyhow!("QuickJS engine: {e}"))?)),
        "jsc" => Ok(Some(EngineRuntime::new(JsEngineKind::JSC).map_err(|e| anyhow::anyhow!("JSC engine: {e}"))?)),
        other => anyhow::bail!("Unknown engine '{other}'. Available: v8, boa, quickjs, jsc, auto"),
    }
}

pub fn exec_with_engine(engine: &EngineRuntime, code: &str) -> anyhow::Result<String> {
    engine.eval(code).map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn exec_file_with_engine(engine: &EngineRuntime, path: &std::path::Path) -> anyhow::Result<String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", path.display()))?;
    let filename = path.to_str().unwrap_or("<file>");
    engine.execute_script(filename, &source).map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn engine_info_json() -> serde_json::Value {
    let results = benchmark_all_engines();
    let engines: Vec<serde_json::Value> = results.iter().map(|(kind, result)| {
        serde_json::json!({
            "name": kind.name(),
            "available": result.success,
            "eval_time_ns": result.eval_time.as_nanos(),
            "error": result.error,
        })
    }).collect();
    serde_json::json!({ "js_engines": engines, "best": detect_best_engine().name() })
}

// ── Verbosity / Output ───────────────────────────────────────────────────

static VERBOSITY: Lazy<Mutex<Verbosity>> = Lazy::new(|| Mutex::new(Verbosity::Normal));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

impl Verbosity {
    #[inline]
    pub fn is_quiet(&self) -> bool { *self == Verbosity::Quiet }
    #[inline]
    pub fn is_verbose(&self) -> bool { *self == Verbosity::Verbose }
}

pub fn set_verbosity(count: u8) {
    let mut v = VERBOSITY.lock();
    *v = match count {
        0 => Verbosity::Normal,
        1 => Verbosity::Verbose,
        _ => Verbosity::Quiet,
    };
}

#[inline]
pub fn log_info(msg: impl AsRef<str>) {
    if !VERBOSITY.lock().is_quiet() {
        eprintln!("{}", msg.as_ref());
    }
}

#[inline]
pub fn log_debug(msg: impl AsRef<str>) {
    if VERBOSITY.lock().is_verbose() {
        eprintln!("[debug] {}", msg.as_ref());
    }
}

// ── Extensions (freshly built each call; Extension is not Clone) ─────────

#[inline]
pub fn all_extensions() -> Vec<deno_core::Extension> {
    vec![
        klyron_ext_console::init(),
        klyron_ext_timers::init(),
        klyron_ext_fs::init(),
        klyron_ext_net::init(),
        klyron_ext_http::init(),
        klyron_ext_crypto::init(),
        klyron_ext_web::init(),
        klyron_ext_node::init(),
        klyron_ext_bun::init(),
        klyron_ext_klyron::init(),
        klyron_ext_html::init(),
        klyron_ext_ffi::init(),
        klyron_ext_ws::init(),
    ]
}

// ── Adapter Registry (lazy-loaded) ───────────────────────────────────────

static ADAPTER_REGISTRY: Lazy<Mutex<AdapterRegistry>> = Lazy::new(|| {
    let mut reg = AdapterRegistry::new();
    register_all(&mut reg);
    Mutex::new(reg)
});

// ── CLI Structure ─────────────────────────────────────────────────────────

const STYLES: clap::builder::Styles = clap::builder::Styles::styled()
    .header(clap::builder::styling::AnsiColor::Green.on_default().bold())
    .usage(clap::builder::styling::AnsiColor::Green.on_default().bold())
    .literal(clap::builder::styling::AnsiColor::Cyan.on_default().bold())
    .placeholder(clap::builder::styling::AnsiColor::White.on_default())
    .error(clap::builder::styling::AnsiColor::Red.on_default().bold())
    .valid(clap::builder::styling::AnsiColor::Cyan.on_default().bold())
    .invalid(clap::builder::styling::AnsiColor::Yellow.on_default().bold());

#[derive(Parser)]
#[command(name = "klyron", version = "", disable_version_flag = true, about = "Klyron - Universal Polyglot Runtime", long_about = None, styles = STYLES, subcommand_required = false)]
pub struct Cli {
    #[arg(short = 'v', long = "verbose", global = true, action = clap::ArgAction::Count, help = "Increase verbosity (use -q for quiet)")]
    pub verbose: u8,

    #[arg(short = 'q', long = "quiet", global = true, default_value_t = false, conflicts_with = "verbose")]
    pub quiet: bool,

    #[arg(short = 'V', long = "version", help = "Print version information")]
    pub show_version: bool,

    #[arg(long = "engine", global = true, help = "JavaScript engine to use (v8, boa, quickjs, jsc, auto)")]
    pub engine: Option<String>,

    #[arg(long = "engine-pool-size", global = true, default_value_t = 4, help = "Size of the concurrent engine pool")]
    pub engine_pool_size: usize,

    #[arg(long = "pre-warm", global = true, default_value_t = false, help = "Pre-warm engines at startup")]
    pub pre_warm: bool,

    #[arg(long = "json", global = true, help = "Output in JSON format")]
    pub json: bool,

    /// Package manager to use (npm, pnpm, yarn, bun, klyron). Defaults to klyron native.
    #[arg(long = "pm", global = true, help = "Package mode: npm, pnpm, yarn, bun, klyron")]
    pub pm: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(long)]
    pub list: bool,
    #[arg(long)]
    pub versions: bool,
    pub framework: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    pub name: Option<String>,
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
    #[arg(long)]
    pub external: bool,
    #[arg(long)]
    pub stack: Option<String>,
    /// Package manager to use (npm, pnpm, yarn, klyron). Defaults to klyron native.
    #[arg(long)]
    pub pm: Option<String>,
}

#[derive(Args)]
pub struct InfoArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Evaluate JavaScript/TypeScript code inline from the command line")]
    Eval { #[command(flatten)] args: commands::runtime::EvalArgs },
    #[command(about = "Execute a JavaScript, TypeScript, or any supported language file")]
    Run { #[command(flatten)] args: commands::runtime::RunArgs },
    #[command(about = "Start an interactive REPL session with live code evaluation")]
    Repl,
    #[command(about = "Start an interactive shell with support for all supported languages")]
    Shell,
    #[command(about = "Bundle JavaScript/TypeScript and dependencies into a single output file")]
    Bundle { entry: PathBuf, #[arg(long, default_value = "bundle.js")] output: PathBuf, #[arg(long)] minify: bool },
    #[command(about = "Compile and execute C source code natively")]
    Cc { source: String, #[arg(long)] args: Option<String>, #[arg(long)] watch: bool },
    #[command(about = "Compile and execute C++ source code natively")]
    Cxx { source: String, #[arg(long)] args: Option<String>, #[arg(long)] watch: bool },
    #[command(about = "Execute TypeScript files directly without prior compilation")]
    Ts { source: String, #[arg(long)] watch: bool },
    #[command(about = "Execute PHP scripts using the embedded PHP engine")]
    Php { source: String, #[arg(long)] watch: bool },
    #[command(about = "Execute Python scripts using the embedded Python engine")]
    Py { source: String, #[arg(long)] watch: bool },
    #[command(about = "Execute Ruby scripts using the embedded Ruby engine")]
    Rb { source: String, #[arg(long)] watch: bool },
    #[command(about = "Compile and run Go source code")]
    Go { source: String, #[arg(long)] watch: bool },
    #[command(about = "Compile and run Zig source code")]
    Zig { source: String, #[arg(long)] watch: bool },
    #[command(about = "Compile and run Rust source code with Cargo")]
    Rs { source: String, #[arg(long)] watch: bool },
    #[command(about = "Execute JavaScript files with the selected JS engine")]
    Js { source: String, #[arg(long)] watch: bool },
    #[command(about = "Run Laravel Artisan commands (migrate, make:model, route:list, etc.)")]
    Artisan { #[command(flatten)] args: commands::artisan::ArtisanArgs },
    #[command(about = "Run PHP Composer commands (install, require, update, etc.)")]
    Composer { #[command(flatten)] args: commands::artisan::ComposerArgs },
    #[command(about = "Render and preview Laravel Blade templates")]
    Blade { #[command(flatten)] args: commands::artisan::BladeArgs },
    #[command(about = "Run Laravel Tinker - an interactive PHP shell for Laravel")]
    Tinker { #[command(flatten)] args: commands::artisan::TinkerArgs },
    #[command(about = "Create a new project from a scaffold template with guided setup")]
    Create { #[command(flatten)] args: CreateArgs },
    #[command(about = "Scaffold a new Next.js application with App Router or Pages Router")]
    CreateNextApp { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new React application with Vite and TypeScript")]
    CreateReactApp { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Angular application with CLI")]
    CreateAngular { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Vue 3 application with Vite and TypeScript")]
    CreateVue { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Svelte application with Vite")]
    CreateSvelte { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Express.js backend application with TypeScript")]
    CreateExpress { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Fastify backend application with TypeScript")]
    CreateFastify { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new NestJS backend application with TypeScript")]
    CreateNest { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Nuxt 3 application with SSR support")]
    CreateNuxt { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Remix application with React Router")]
    CreateRemix { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Gatsby static site with React")]
    CreateGatsby { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Astro static site generator project")]
    CreateAstro { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new AdonisJS backend application")]
    CreateAdonis { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Laravel PHP application with Sail support")]
    CreateLaravel { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold Laravel with React frontend scaffolding (Inertia)")]
    CreateLaravelReact { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold Laravel with Vue frontend scaffolding (Inertia)")]
    CreateLaravelVue { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold Laravel with Inertia.js + React stack")]
    CreateLaravelInertiaReact { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold Laravel with Inertia.js + Vue stack")]
    CreateLaravelInertiaVue { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold Laravel with Livewire full-stack components")]
    CreateLaravelLivewire { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold Laravel with Next.js frontend (Breeze/Inertia)")]
    CreateLaravelNext { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold Laravel with Astro frontend")]
    CreateLaravelAstro { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold Laravel API-only application (no frontend)")]
    CreateLaravelApi { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Django Python web application")]
    CreateDjango { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Ruby on Rails application")]
    CreateRails { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Actix-web Rust backend application")]
    CreateActix { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Axum Rust backend application")]
    CreateAxum { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Rocket Rust web application")]
    CreateRocket { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new SolidJS application with Vite")]
    CreateSolid { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Qwik application with Vite")]
    CreateQwik { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Preact application with Vite")]
    CreatePreact { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Lit web components project")]
    CreateLit { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new FastAPI Python backend application")]
    CreateFastApi { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Flask Python web application")]
    CreateFlask { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Gin Go web framework application")]
    CreateGoGin { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Fiber Go web framework application")]
    CreateGoFiber { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Echo Go web framework application")]
    CreateGoEcho { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Tauri desktop application (Rust + frontend)")]
    CreateTauri { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Leptos Rust full-stack web application")]
    CreateLeptos { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Symfony PHP web application")]
    CreateSymfony { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new CodeIgniter PHP web application")]
    CreateCodeIgniter { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new WordPress project with custom theme")]
    CreateWordPress { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new SvelteKit full-stack application")]
    CreateSvelteKit { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Hono web framework application (JS/TS)")]
    CreateHono { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Koa.js backend application")]
    CreateKoa { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Scaffold a new Hapi.js backend application")]
    CreateHapi { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    #[command(about = "Start the development server with hot module replacement and file watching")]
    Dev { #[command(flatten)] args: commands::dev::DevArgs },
    #[command(about = "Build the project for production deployment")]
    Build { #[command(flatten)] args: commands::build::BuildArgs },
    #[command(about = "Run project tests with the configured test runner")]
    Test { #[command(flatten)] args: commands::test::TestArgs },
    #[command(about = "Lint project source code with ESLint or configured linter")]
    Lint { #[command(flatten)] args: commands::lint::LintArgs },
    #[command(about = "Format project source code with Prettier or configured formatter")]
    Format { #[command(flatten)] args: commands::format::FormatArgs },
    #[command(about = "Type-check the project with TypeScript or configured type checker")]
    Check { #[command(flatten)] args: commands::check::CheckArgs },
    #[command(about = "Run performance benchmarks for the project")]
    Bench { #[command(flatten)] args: commands::bench::BenchArgs },
    #[command(about = "Start the production server")]
    Start,
    #[command(about = "Run an npm script defined in package.json")]
    RunScript { #[command(flatten)] args: commands::scripts::RunScriptArgs },
    #[command(about = "Database management commands (migrate, seed, rollback, etc.)")]
    Db { #[command(flatten)] args: commands::db::DbArgs },
    #[command(about = "Run Prisma ORM commands (generate, migrate, push, studio, etc.)")]
    Prisma { #[command(subcommand)] action: commands::orm::PrismaAction },
    #[command(about = "Run Drizzle ORM commands (generate, migrate, push, etc.)")]
    Drizzle { #[command(subcommand)] action: commands::orm::DrizzleAction },
    #[command(about = "Run TypeORM commands (migration:run, schema:sync, etc.)")]
    Typeorm { #[command(subcommand)] action: commands::orm::TypeOrmAction },
    #[command(about = "Run MikroORM commands (schema:update, migration:create, etc.)")]
    MikroOrm { #[command(subcommand)] action: commands::orm::MikroOrmAction },
    #[command(about = "Run Sequelize ORM commands (db:migrate, db:seed, etc.)")]
    Sequelize { #[command(subcommand)] action: commands::orm::SequelizeAction },
    #[command(about = "Run Mongoose ODM commands (generate model, seed, etc.)")]
    Mongoose { #[command(subcommand)] action: commands::orm::MongooseAction },
    #[command(about = "Run Kysely query builder commands (migrate, seed, etc.)")]
    Kysely { #[command(subcommand)] action: commands::orm::KyselyAction },
    #[command(about = "Run Knex.js SQL query builder commands (migrate, seed, etc.)")]
    KnexCmd { #[command(subcommand)] action: commands::orm::KnexAction },
    #[command(about = "Unified ORM command namespace for all supported database tools")]
    Orm { #[command(subcommand)] action: commands::orm::OrmCommand },
    #[command(about = "Install project dependencies from package.json (supports npm/pnpm/klyron)")]
    Install {
        #[arg(long = "frozen-lockfile")]
        frozen_lockfile: bool,
    },
    #[command(about = "Add a new package dependency to the project")]
    Add { #[command(flatten)] args: commands::pm::AddArgs },
    #[command(about = "Remove a package dependency from the project")]
    Remove { #[command(flatten)] args: commands::pm::RemoveArgs },
    #[command(about = "Uninstall a package dependency (alias for remove)")]
    Uninstall { #[command(flatten)] args: commands::pm::RemoveArgs },
    #[command(about = "List outdated packages with available versions")]
    Outdated,
    #[command(about = "Update project dependencies to latest compatible versions")]
    Update,
    #[command(about = "Audit project dependencies for security vulnerabilities")]
    Audit,
    #[command(about = "Deduplicate duplicate dependencies in the lockfile")]
    Dedupe,
    #[command(about = "Manage the lockfile (generate, check, verify)")]
    Lock { #[command(flatten)] args: commands::pm::LockArgs },
    #[command(about = "Publish a package to the registry")]
    Publish { #[command(flatten)] args: commands::registry::PublishArgs },
    #[command(about = "Remove a published package from the registry")]
    Unpublish { #[command(flatten)] args: commands::registry::UnpublishArgs },
    #[command(about = "Log in to the package registry")]
    Login { #[command(flatten)] args: commands::registry::LoginArgs },
    #[command(about = "Log out from the package registry")]
    Logout { #[command(flatten)] args: commands::registry::LogoutArgs },
    #[command(about = "Display the currently logged-in user")]
    Whoami,
    #[command(about = "Search for packages in the registry")]
    Search { #[command(flatten)] args: commands::registry::SearchArgs },
    #[command(about = "Display detailed information about a package")]
    InfoCmd { #[command(flatten)] args: commands::registry::InfoArgs },
    #[command(about = "Create a tarball package from the current project")]
    Pack { #[arg(long)] output: Option<PathBuf> },
    #[command(about = "Link a local package for development")]
    Link { #[command(flatten)] args: commands::pm::LinkArgs },
    #[command(about = "Unlink a previously linked local package")]
    Unlink { package: String },
    #[command(about = "Manage distribution tags for packages")]
    DistTag { #[command(subcommand)] action: commands::pm::DistTagAction },
    #[command(about = "Show why a package is installed (dependency tree)")]
    Why { package: String },
    #[command(about = "Manage monorepo workspaces (add, remove, list, exec)")]
    Workspace { #[command(subcommand)] action: commands::workspace::WorkspaceAction },
    #[command(about = "Manage Klyron plugins (install, remove, list, create)")]
    Plugin { #[command(subcommand)] action: commands::plugin::PluginAction },
    #[command(about = "Manage the Klyron cache (clean, ls, info, verify)")]
    Cache { #[command(subcommand)] action: commands::cache::CacheAction },
    #[command(about = "Docker container management for project services")]
    Docker { #[command(subcommand)] action: commands::docker::DockerAction },
    #[command(about = "Build and manage N-API native addons for Node.js compatibility")]
    Napi { #[command(subcommand)] action: commands::napi::NapiAction },
    #[command(about = "Deploy the project to a configured deployment target")]
    Deploy { #[command(flatten)] args: commands::deploy::DeployArgs },
    #[command(about = "Check and manage Node.js compatibility of packages")]
    Compat { #[command(flatten)] args: commands::compat::CompatArgs },
    #[command(about = "AI-assisted development commands (generate, explain, refactor)")]
    Ai { #[command(flatten)] args: commands::ai::AiArgs },
    #[command(about = "Watch files and auto-restart the dev server on changes")]
    Watch { #[command(flatten)] args: commands::watch::WatchArgs },
    #[command(about = "Initialize a new klyron.json configuration in the current directory")]
    Init,
    #[command(about = "Upgrade Klyron to the latest version")]
    Upgrade,
    #[command(about = "Run system diagnostics to verify the Klyron setup")]
    Doctor,
    #[command(about = "Display detailed system, project, and environment information")]
    Info { #[command(flatten)] args: InfoArgs },
    #[command(about = "Display Klyron version information")]
    Version,
    #[command(about = "Clean build artifacts, cache, and temporary directories")]
    Clean { #[arg(long)] yes: bool },
    #[command(about = "Generate code coverage reports from test runs")]
    Coverage { #[command(flatten)] args: commands::coverage::CoverageArgs },
    #[command(about = "Manage telemetry settings (enable, disable, status)")]
    Telemetry { #[command(subcommand)] action: Option<commands::utils::TelemetryAction> },
    #[command(about = "Manage Klyron configuration (get, set, list, edit)")]
    Config { #[command(subcommand)] action: commands::config::ConfigAction },
    #[command(about = "Laravel framework utility commands (serve, tinker, model, etc.)")]
    Laravel { #[command(subcommand)] action: commands::laravel::LaravelCommand },
    #[command(about = "Start a static file or development HTTP server")]
    Serve { #[arg(long, default_value = "localhost")] host: String, #[arg(long, default_value_t = 3000)] port: u16, #[arg(long)] dir: Option<PathBuf>, #[arg(long)] watch: bool },
    #[command(about = "Generate shell completion scripts for bash, zsh, fish, and powershell")]
    Completions { shell: clap_complete::Shell },
    #[command(about = "Manage project templates from adapters directory")]
    Template { #[command(subcommand)] action: TemplateAction },
}

#[derive(Subcommand)]
pub enum TemplateAction {
    #[command(about = "List all available templates")]
    List { #[arg(short, long)] category: Option<String> },
    #[command(about = "Show detailed info about a template")]
    Show { name: String },
    #[command(about = "Create a new project from a template with interactive version picker")]
    Create { name: String, project_name: String, #[arg(short, long)] dir: Option<PathBuf> },
}

// ── Info handler ──────────────────────────────────────────────────────────

fn handle_info(json: bool) -> anyhow::Result<()> {
    let js_engines = engine_info_json();
    let mut all_engines: Vec<&str> = vec!["c", "cpp", "ts", "js", "py", "rb", "php", "go", "zig", "rs"];
    // Add available JS runtimes
    if let Some(engines) = js_engines["js_engines"].as_array() {
        for e in engines {
            if e["available"].as_bool().unwrap_or(false) {
                all_engines.push(e["name"].as_str().unwrap_or("?"));
            }
        }
    }

    let profiles = profile_all_engines(100);
    let engine_metrics: Vec<serde_json::Value> = profiles.iter().map(|p| {
        serde_json::json!({
            "engine": p.engine_kind.name(),
            "ops_per_sec": p.ops_per_sec,
            "avg_eval_time_ns": p.avg_eval_time_ns,
            "warmup_complete": p.warmup_complete,
        })
    }).collect();

    let info = serde_json::json!({
        "name": "klyron",
        "version": env!("CARGO_PKG_VERSION"),
        "engines": all_engines,
        "js_engines": js_engines["js_engines"],
        "best_engine": js_engines["best"],
        "adapters": {
            "count": ADAPTER_REGISTRY.lock().names().len(),
        },
        "engine_metrics": engine_metrics,
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("Klyron v{}", info["version"]);
        let engines = info["engines"].as_array().map(|arr| {
            arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")
        }).unwrap_or_default();
        println!("Engines: {engines}");
        println!("Best JS engine: {}", info["best_engine"].as_str().unwrap_or("unknown"));
        if let Some(js_arr) = info["js_engines"].as_array() {
            for e in js_arr {
                let name = e["name"].as_str().unwrap_or("?");
                let avail = if e["available"].as_bool().unwrap_or(false) { "available" } else { "unavailable" };
                println!("  JS/{name}: {avail}");
            }
        }
        println!("Adapters: {} registered", info["adapters"]["count"]);
        println!("Engine Performance:");
        for m in &engine_metrics {
            let name = m["engine"].as_str().unwrap_or("?");
            let ops = m["ops_per_sec"].as_f64().unwrap_or(0.0);
            let avg_ns = m["avg_eval_time_ns"].as_f64().unwrap_or(0.0);
            println!("  {name:<8} {ops:>10.0} ops/s  avg {avg_ns:>8.0} ns");
        }
    }
    Ok(())
}

// ── Create handler ────────────────────────────────────────────────────────

fn list_frameworks() {
    let registry = ADAPTER_REGISTRY.lock();
    let mut names: Vec<&str> = registry.names().into_iter().collect();
    names.sort();
    let count = names.len();
    println!("Available frameworks ({count}):");
    for name in &names {
        let Some(adapter) = registry.get(name) else { continue };
        let versions = adapter.supported_versions();
        let default = adapter.default_version();
        println!("  {name:<12} versions: {versions:?} (default: {default})");
    }
    println!();
    println!("Additional frameworks available via --external:");
    println!("  express, nest, django, rails, sveltekit, remix, angular, symfony, codeigniter");
}

fn handle_create(args: CreateArgs) -> anyhow::Result<()> {
    if args.list {
        list_frameworks();
        commands::template::list_templates();
        return Ok(());
    }

    let framework = args.framework.as_deref().unwrap_or("");
    if framework.is_empty() {
        list_frameworks();
        println!();
        commands::template::list_templates();
        return Ok(());
    }

    let name = args.name.as_deref().unwrap_or("");

    // Check adapters/ directory first (filesystem templates)
    if commands::template::template_exists(framework) {
        if name.is_empty() {
            anyhow::bail!("Missing project name. Usage: klyron create <template> <project-name>");
        }
        return commands::template::create_template(framework, name, args.version.as_deref(), Some(&args.dir));
    }

    if args.versions {
        let registry = ADAPTER_REGISTRY.lock();
        if let Some(adapter) = registry.get(framework) {
            let versions = adapter.supported_versions();
            let default = adapter.default_version();
            println!("{framework} supported versions: {versions:?} (default: {default})");
        } else {
            println!("{framework} is not in the adapter registry. Use --external to scaffold via official CLI.");
        }
        return Ok(());
    }

    if name.is_empty() {
        anyhow::bail!("Missing project name. Usage: klyron create <framework> <name> [options]");
    }

    let scaffold_args = commands::scaffold::ScaffoldArgs {
        name: name.to_string(),
        dir: args.dir,
        version: args.version,
        external: args.external,
        stack: args.stack,
        pm: args.pm,
    };

    if args.external {
        return commands::scaffold::scaffold_via_external_cli(framework, &scaffold_args);
    }

    commands::scaffold::scaffold_via_adapter(&scaffold_args, framework)
}

// ── Run CLI ───────────────────────────────────────────────────────────────

pub fn run_cli() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    // Initialize signal handlers for graceful shutdown
    signal::SignalHandler::setup();

    let cli = Cli::parse();

    // Custom --version
    if cli.show_version {
        crate::splash::show_version();
        return Ok(());
    }

    // No subcommand → show splash screen
    if cli.command.is_none() {
        crate::splash::show_splash();
        return Ok(());
    }

    if cli.quiet {
        set_verbosity(2);
    } else {
        set_verbosity(cli.verbose);
    }

    log_debug("Running klyron command");

    let engine = resolve_engine(cli.engine.as_deref())?;

    if cli.pre_warm {
        let kind = engine.as_ref().map(|e| e.kind()).unwrap_or(JsEngineKind::Boa);
        let warmer = EnginePreWarmer::new(kind);
        warmer.start_background(cli.engine_pool_size);
        log_info(format!("Pre-warming {} engines (pool size: {})", kind, cli.engine_pool_size));
    }

    let cmd = match cli.command {
        Some(c) => c,
        None => {
            handle_info(false)?;
            return Ok(());
        }
    };

    let result = dispatch_command(cmd, engine, cli.json);

    if cli.json {
        match &result {
            Ok(()) => {
                println!("{}", serde_json::json!({"status": "ok", "message": "Command completed successfully"}));
            }
            Err(e) => {
                println!("{}", serde_json::json!({"status": "error", "message": e.to_string()}));
            }
        }
    }

    result
}

// ── Dispatch ──────────────────────────────────────────────────────────────

pub fn dispatch_command(cmd: Commands, engine: Option<EngineRuntime>, json_output: bool) -> anyhow::Result<()> {
    match cmd {
        Commands::Eval { args } => {
            if let Some(eng) = engine {
                if !args.typescript && !args.jsx {
                    let result = exec_with_engine(&eng, &args.code)?;
                    if !result.is_empty() { println!("{result}"); }
                    return Ok(());
                }
            }
            commands::runtime::eval_with_args(args)
        }
        Commands::Run { args } => {
            let dir = std::env::current_dir().unwrap_or_default();
            let file_path = if args.path.is_absolute() {
                args.path.clone()
            } else {
                dir.join(&args.path)
            };
            if file_path.is_file() {
                if let Some(eng) = engine {
                    let result = exec_file_with_engine(&eng, &args.path)?;
                    if !result.is_empty() { println!("{result}"); }
                    Ok(())
                } else {
                    commands::runtime::run_file(args, all_extensions())
                }
            } else {
                commands::scripts::run_script(commands::scripts::RunScriptArgs {
                    script: args.path.to_string_lossy().to_string(),
                    args: args.extra,
                })
            }
        }
        Commands::Repl => {
            let eng = engine;
            commands::runtime::repl_loop_ext(eng)
        }
        Commands::Shell => commands::runtime::shell_loop(),
        Commands::Bundle { entry, output, minify: _ } => {
            if let Some(eng) = engine {
                let result = exec_file_with_engine(&eng, &entry)?;
                std::fs::write(&output, result)
                    .map_err(|e| anyhow::anyhow!("Cannot write {}: {e}", output.display()))?;
                log_info(format!("Bundled {} -> {} (JS engine)", entry.display(), output.display()));
            } else {
                let source = std::fs::read_to_string(&entry)
                    .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", entry.display()))?;
                let runtime = Runtime::builder()
                    .enable_typescript(true)
                    .extensions(all_extensions())
                    .build()?;
                let js = runtime.execute_script(entry.to_str().unwrap_or("<entry>"), &source)?;
                std::fs::write(&output, js)
                    .map_err(|e| anyhow::anyhow!("Cannot write {}: {e}", output.display()))?;
                log_info(format!("Bundled {} -> {}", entry.display(), output.display()));
            }
            Ok(())
        }
        Commands::Cc { source, args, watch } => run_engine_watch("C", &source, watch, |watch_path| {
            let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut eng = engines::CEngine::new()?;
            eng.exec(&code, args.as_deref())
        }),
        Commands::Cxx { source, args, watch } => run_engine_watch("C++", &source, watch, |watch_path| {
            let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut eng = engines::CppEngine::new()?;
            eng.exec(&code, args.as_deref())
        }),
        Commands::Ts { source, watch } => run_engine_watch("TS", &source, watch, |watch_path| {
            let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut eng = engines::TsEngine::new()?;
            eng.exec(&code)
        }),
        Commands::Php { source, watch } => run_engine_watch("PHP", &source, watch, |watch_path| {
            let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut eng = engines::PhpEngine::new()?;
            eng.exec(&code)
        }),
        Commands::Py { source, watch } => run_engine_watch("Py", &source, watch, |watch_path| {
            let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut eng = engines::PyEngine::new()?;
            eng.exec(&code)
        }),
        Commands::Rb { source, watch } => run_engine_watch("Rb", &source, watch, |watch_path| {
            let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut eng = engines::RbEngine::new()?;
            eng.exec(&code)
        }),
        Commands::Go { source, watch } => run_engine_watch("Go", &source, watch, |watch_path| {
            let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut eng = engines::GoEngine::new()?;
            eng.exec(&code, None)
        }),
        Commands::Zig { source, watch } => run_engine_watch("Zig", &source, watch, |watch_path| {
            let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut eng = engines::ZigEngine::new()?;
            eng.exec(&code, None)
        }),
        Commands::Rs { source, watch } => run_engine_watch("Rs", &source, watch, |watch_path| {
            let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut eng = engines::RsEngine::new()?;
            eng.exec(&code, None)
        }),
        Commands::Js { source, watch } => {
            if engine.is_some() {
                let path = std::path::PathBuf::from(&source);
                let exec_fn = |p: &std::path::Path| -> anyhow::Result<String> {
                    let eng = resolve_engine(Some("auto"))?.unwrap();
                    let code = std::fs::read_to_string(p)?;
                    eng.execute_script(p.to_str().unwrap_or("<file>"), &code).map_err(|e| anyhow::anyhow!("{e}"))
                };
                let result = exec_fn(&path)?;
                println!("{result}");
                if watch {
                    let path_clone = path.clone();
                    crate::watch_file(&path, move || {
                        if let Ok(r) = exec_fn(&path_clone) { println!("{r}"); }
                    });
                }
                Ok(())
            } else {
                run_engine_watch("Js", &source, watch, |watch_path| {
                    let code = std::fs::read_to_string(watch_path).map_err(|e| anyhow::anyhow!("{e}"))?;
                    let mut eng = engines::JsEngine::new()?;
                    eng.exec(&code)
                })
            }
        }
        Commands::Artisan { args } => commands::artisan::run_artisan(&args.args, args.project.as_deref()),
        Commands::Composer { args } => commands::artisan::run_composer(&args.args, args.project.as_deref()),
        Commands::Blade { args } => commands::artisan::run_blade(&args.view, args.data.as_deref(), args.project.as_deref()),
        Commands::Tinker { args } => commands::artisan::run_tinker(args.project.as_deref()),
        Commands::Create { args } => handle_create(args),
        Commands::CreateNextApp { args } => commands::scaffold::scaffold_next(&args),
        Commands::CreateReactApp { args } => commands::scaffold::scaffold_react(&args),
        Commands::CreateAngular { args } => commands::scaffold::scaffold_angular(&args),
        Commands::CreateVue { args } => commands::scaffold::scaffold_vue(&args),
        Commands::CreateSvelte { args } => commands::scaffold::scaffold_svelte(&args),
        Commands::CreateExpress { args } => commands::scaffold::scaffold_express(&args),
        Commands::CreateFastify { args } => commands::scaffold::scaffold_fastify(&args),
        Commands::CreateNest { args } => commands::scaffold::scaffold_nest(&args),
        Commands::CreateNuxt { args } => commands::scaffold::scaffold_nuxt(&args),
        Commands::CreateRemix { args } => commands::scaffold::scaffold_remix(&args),
        Commands::CreateGatsby { args } => scaffold_gatsby(&args.name, &args.dir),
        Commands::CreateAstro { args } => commands::scaffold::scaffold_astro(&args),
        Commands::CreateAdonis { args } => commands::scaffold::scaffold_adonis(&args),
        Commands::CreateLaravel { args } => scaffold_laravel(&args.name, &args.dir),
        Commands::CreateLaravelReact { args } => scaffold_laravel_stack(&args.name, &args.dir, "react"),
        Commands::CreateLaravelVue { args } => scaffold_laravel_stack(&args.name, &args.dir, "vue"),
        Commands::CreateLaravelInertiaReact { args } => scaffold_laravel_stack(&args.name, &args.dir, "inertia-react"),
        Commands::CreateLaravelInertiaVue { args } => scaffold_laravel_stack(&args.name, &args.dir, "inertia-vue"),
        Commands::CreateLaravelLivewire { args } => scaffold_laravel_stack(&args.name, &args.dir, "livewire"),
        Commands::CreateLaravelNext { args } => scaffold_laravel_stack(&args.name, &args.dir, "next"),
        Commands::CreateLaravelAstro { args } => scaffold_laravel_stack(&args.name, &args.dir, "astro"),
        Commands::CreateLaravelApi { args } => scaffold_laravel_stack(&args.name, &args.dir, "api"),
        Commands::CreateDjango { args } => scaffold_django(&args.name, &args.dir),
        Commands::CreateRails { args } => scaffold_rails(&args.name, &args.dir),
        Commands::CreateActix { args } => scaffold_rust_project(&args.name, &args.dir, "actix-web"),
        Commands::CreateAxum { args } => scaffold_rust_project(&args.name, &args.dir, "axum"),
        Commands::CreateRocket { args } => scaffold_rust_project(&args.name, &args.dir, "rocket"),
        Commands::CreateSolid { args } => commands::scaffold::scaffold_solid(&args),
        Commands::CreateQwik { args } => commands::scaffold::scaffold_qwik(&args),
        Commands::CreatePreact { args } => commands::scaffold::scaffold_preact(&args),
        Commands::CreateLit { args } => commands::scaffold::scaffold_lit(&args),
        Commands::CreateFastApi { args } => scaffold_fastapi(&args.name, &args.dir),
        Commands::CreateFlask { args } => scaffold_flask(&args.name, &args.dir),
        Commands::CreateGoGin { args } => scaffold_go_gin(&args.name, &args.dir),
        Commands::CreateGoFiber { args } => scaffold_go_fiber(&args.name, &args.dir),
        Commands::CreateGoEcho { args } => scaffold_go_echo(&args.name, &args.dir),
        Commands::CreateSvelteKit { args } => commands::scaffold::scaffold_sveltekit(&args),
        Commands::CreateHono { args } => commands::scaffold::scaffold_hono(&args),
        Commands::CreateKoa { args } => commands::scaffold::scaffold_koa(&args),
        Commands::CreateHapi { args } => commands::scaffold::scaffold_hapi(&args),
        Commands::CreateTauri { args } => commands::scaffold::scaffold_tauri(&args),
        Commands::CreateLeptos { args } => commands::scaffold::scaffold_leptos(&args),
        Commands::CreateSymfony { args } => commands::scaffold::scaffold_symfony(&args),
        Commands::CreateCodeIgniter { args } => commands::scaffold::scaffold_codeigniter(&args),
        Commands::CreateWordPress { args } => commands::scaffold::scaffold_wordpress(&args),
        Commands::Watch { args } => commands::watch::run_watch(args),
        Commands::Dev { args } => commands::dev::run_dev(args),
        Commands::Build { args } => commands::build::run_build(args),
        Commands::Test { args } => commands::test::run_test(args),
        Commands::Lint { args } => commands::lint::run_lint(args),
        Commands::Format { args } => commands::format::run_format(args),
        Commands::Check { args } => commands::check::run_check(args),
        Commands::Bench { args } => commands::bench::run_bench(args),
        Commands::Start => commands::scripts::run_start(),
        Commands::RunScript { args } => commands::scripts::run_script(args),
        Commands::Db { args } => commands::db::run_db(args.action, args.orm.as_deref()),
        Commands::Prisma { action } => commands::orm::run_prisma(action),
        Commands::Drizzle { action } => commands::orm::run_drizzle(action),
        Commands::Typeorm { action } => commands::orm::run_typeorm(action),
        Commands::MikroOrm { action } => commands::orm::run_mikroorm(action),
        Commands::Sequelize { action } => commands::orm::run_sequelize(action),
        Commands::Mongoose { action } => commands::orm::run_mongoose(action),
        Commands::Kysely { action } => commands::orm::run_kysely(action),
        Commands::KnexCmd { action } => commands::orm::run_knex(action),
        Commands::Orm { action } => commands::orm::run_orm(action),
        Commands::Install { frozen_lockfile } => commands::pm::run_install(frozen_lockfile),
        Commands::Add { args } => commands::pm::run_add(&args.packages, args.dev),
        Commands::Remove { args } => commands::pm::run_remove(&args.packages),
        Commands::Uninstall { args } => commands::pm::run_remove(&args.packages),
        Commands::Outdated => commands::pm::run_outdated(),
        Commands::Update => commands::pm::run_update(false),
        Commands::Audit => commands::pm::run_audit(),
        Commands::Dedupe => commands::pm::run_dedupe(),
        Commands::Lock { args } => match args.action {
            commands::pm::LockAction::Verify => commands::pm::run_lock_verify(),
            commands::pm::LockAction::Update { force } => commands::pm::run_lock_update(force),
            commands::pm::LockAction::Migrate { keep } => commands::pm::run_lock_migrate(keep),
        },
        Commands::Publish { args } => commands::registry::run_publish(&args),
        Commands::Unpublish { args } => commands::registry::run_unpublish(&args.name),
        Commands::Login { args } => commands::registry::run_login(args),
        Commands::Logout { args } => commands::registry::run_logout(args),
        Commands::Whoami => commands::registry::run_whoami(),
        Commands::Search { args } => commands::registry::run_search(&args),
        Commands::InfoCmd { args } => commands::registry::run_info(&args.package, json_output),
        Commands::Pack { output } => commands::pm::run_pack(output.as_deref()),
        Commands::Link { args } => commands::pm::run_link(&args),
        Commands::Unlink { package } => commands::pm::run_unlink_global(&package),
        Commands::DistTag { action } => commands::pm::run_dist_tag(action),
        Commands::Why { package } => commands::pm::run_why(&package),
        Commands::Laravel { action } => commands::laravel::run_laravel(action),
        Commands::Info { args } => handle_info(args.json),
        Commands::Workspace { action } => commands::workspace::run_workspace(action),
        Commands::Plugin { action } => commands::plugin::run_plugin(action),
        Commands::Cache { action } => commands::cache::run_cache(action),
        Commands::Docker { action } => commands::docker::run_docker(action),
        Commands::Napi { action } => commands::napi::run_napi(action),
        Commands::Deploy { args } => commands::deploy::run_deploy(args),
        Commands::Compat { args } => commands::compat::run_compat(args),
        Commands::Ai { args } => commands::ai::run_ai(args),
        Commands::Init => commands::utils::run_init(),
        Commands::Upgrade => commands::utils::run_upgrade(),
        Commands::Doctor => commands::utils::run_doctor(),
        Commands::Version => commands::utils::run_version(),
        Commands::Clean { yes } => commands::utils::run_clean(yes),
        Commands::Coverage { args } => commands::coverage::run_coverage(args),
        Commands::Telemetry { action } => commands::utils::run_telemetry(action),
        Commands::Config { action } => commands::config::run_config_action(action),
        Commands::Serve { host, port, dir, watch } => {
            commands::dev::run_dev(commands::dev::DevArgs {
                dir: dir.unwrap_or_else(|| std::env::current_dir().unwrap()),
                port: Some(port),
                host: Some(host),
                watch,
                hot: watch,
                no_hmr_inject: false,
            })
        }
        Commands::Template { action } => match action {
            TemplateAction::List { category } => {
                commands::template::list_templates_with_filter(category.as_deref());
                Ok(())
            }
            TemplateAction::Show { name } => {
                commands::template::show_template(&name);
                Ok(())
            }
            TemplateAction::Create { name, project_name, dir } => {
                commands::template::create_template(&name, &project_name, None, dir.as_deref())
            }
        },
        Commands::Completions { shell } => {
            let mut cmd = <Cli as CommandFactory>::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
            Ok(())
        }
    }
}

// ── Engine helpers ────────────────────────────────────────────────────────

fn run_engine_watch<F>(name: &str, source: &str, watch: bool, f: F) -> anyhow::Result<()>
where
    F: Fn(&Path) -> anyhow::Result<engines::EngineOutput>,
{
    if watch {
        let path = PathBuf::from(source);
        let watch_path = path.clone();
        let output = f(&path)?;
        print_engine_output(&output);
        watch_file(&path, move || {
            if let Ok(output) = f(&watch_path) {
                print_engine_output(&output);
            }
        });
        return Ok(());
    }
        let path = PathBuf::from(source);
        if path.exists() {
            let code = std::fs::read_to_string(&path)?;
            let mut eng = new_engine(name)?;
            let output = eng_exec(&mut eng, name, &code)?;
            print_engine_output(&output);
            if output.exit_code != 0 {
                std::process::exit(output.exit_code);
            }
            return Ok(());
        }
        let mut eng = new_engine(name)?;
        let output = eng_exec(&mut eng, name, source)?;
        print_engine_output(&output);
        if output.exit_code != 0 {
            std::process::exit(output.exit_code);
        }
        Ok(())
    }

fn new_engine(name: &str) -> anyhow::Result<Box<dyn EngineTrait>> {
    match name {
        "C" => Ok(Box::new(engines::CEngine::new()?)),
        "C++" => Ok(Box::new(engines::CppEngine::new()?)),
        "TS" => Ok(Box::new(engines::TsEngine::new()?)),
        "PHP" => Ok(Box::new(engines::PhpEngine::new()?)),
        "Py" => Ok(Box::new(engines::PyEngine::new()?)),
        "Rb" => Ok(Box::new(engines::RbEngine::new()?)),
        "Go" => Ok(Box::new(engines::GoEngine::new()?)),
        "Zig" => Ok(Box::new(engines::ZigEngine::new()?)),
        "Rs" => Ok(Box::new(engines::RsEngine::new()?)),
        "Js" => Ok(Box::new(engines::JsEngine::new()?)),
        _ => anyhow::bail!("Unknown engine: {name}"),
    }
}

fn eng_exec(eng: &mut Box<dyn EngineTrait>, _name: &str, code: &str) -> anyhow::Result<engines::EngineOutput> {
    eng.exec(code)
}

trait EngineTrait {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput>;
}

impl EngineTrait for engines::CEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code, None) }
}
impl EngineTrait for engines::CppEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code, None) }
}
impl EngineTrait for engines::TsEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code) }
}
impl EngineTrait for engines::PhpEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code) }
}
impl EngineTrait for engines::PyEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code) }
}
impl EngineTrait for engines::RbEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code) }
}
impl EngineTrait for engines::GoEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code, None) }
}
impl EngineTrait for engines::ZigEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code, None) }
}
impl EngineTrait for engines::RsEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code, None) }
}
impl EngineTrait for engines::JsEngine {
    fn exec(&mut self, code: &str) -> anyhow::Result<engines::EngineOutput> { self.exec(code) }
}

fn print_engine_output(output: &engines::EngineOutput) {
    if !output.stdout.is_empty() { print!("{}", output.stdout); }
    if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
}
