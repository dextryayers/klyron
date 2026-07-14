pub mod engines;
pub mod commands;
pub mod scaffold_inline;
pub mod color;

pub(crate) use commands::helpers::*;
pub(crate) use scaffold_inline::*;
pub(crate) use color::Color;

use std::path::{Path, PathBuf};
use std::io::Write;
use clap::{Parser, Subcommand, Args, CommandFactory};
use klyron_adapter::AdapterRegistry;
use klyron_adapter::adapters::register_all;
use klyron_core::Runtime;
use klyron_engine::{JsEngineKind, EngineRuntime, detect_best_engine, benchmark_all_engines};
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
        klyron_ext_klyron::init(),
        klyron_ext_html::init(),
        klyron_ext_ffi::init(),
        klyron_ext_ws::init(),
        klyron_ext_process::init(),
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
#[command(name = "klyron", version, about = "Klyron - Universal Polyglot Runtime", long_about = None, styles = STYLES)]
pub struct Cli {
    #[arg(short = 'v', long = "verbose", global = true, action = clap::ArgAction::Count, help = "Increase verbosity (use -q for quiet)")]
    pub verbose: u8,

    #[arg(short = 'q', long = "quiet", global = true, default_value_t = false, conflicts_with = "verbose")]
    pub quiet: bool,

    #[arg(long = "engine", global = true, help = "JavaScript engine to use (v8, boa, quickjs, jsc, auto)")]
    pub engine: Option<String>,

    #[arg(long = "json", global = true, help = "Output in JSON format")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
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
}

#[derive(Args)]
pub struct InfoArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Eval { #[command(flatten)] args: commands::runtime::EvalArgs },
    Run { #[command(flatten)] args: commands::runtime::RunArgs },
    Repl,
    Shell,
    Bundle { entry: PathBuf, #[arg(long, default_value = "bundle.js")] output: PathBuf, #[arg(long)] minify: bool },
    Cc { source: String, #[arg(long)] args: Option<String>, #[arg(long)] watch: bool },
    Cxx { source: String, #[arg(long)] args: Option<String>, #[arg(long)] watch: bool },
    Ts { source: String, #[arg(long)] watch: bool },
    Php { source: String, #[arg(long)] watch: bool },
    Py { source: String, #[arg(long)] watch: bool },
    Rb { source: String, #[arg(long)] watch: bool },
    Go { source: String, #[arg(long)] watch: bool },
    Zig { source: String, #[arg(long)] watch: bool },
    Rs { source: String, #[arg(long)] watch: bool },
    Js { source: String, #[arg(long)] watch: bool },
    Artisan { #[command(flatten)] args: commands::artisan::ArtisanArgs },
    Composer { #[command(flatten)] args: commands::artisan::ComposerArgs },
    Blade { #[command(flatten)] args: commands::artisan::BladeArgs },
    Tinker { #[command(flatten)] args: commands::artisan::TinkerArgs },
    Create { #[command(flatten)] args: CreateArgs },
    CreateNextApp { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateReactApp { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateAngular { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateVue { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateSvelte { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateExpress { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateFastify { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateNest { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateNuxt { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateRemix { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateGatsby { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateAstro { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateAdonis { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLaravel { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLaravelReact { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLaravelVue { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLaravelInertiaReact { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLaravelInertiaVue { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLaravelLivewire { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLaravelNext { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLaravelAstro { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLaravelApi { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateDjango { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateRails { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateActix { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateAxum { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateRocket { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateSolid { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateQwik { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreatePreact { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLit { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateFastApi { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateFlask { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateGoGin { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateGoFiber { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateGoEcho { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateTauri { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateLeptos { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateSymfony { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateCodeIgniter { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateWordPress { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateSvelteKit { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateHono { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateKoa { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    CreateHapi { #[command(flatten)] args: commands::scaffold::ScaffoldArgs },
    Dev { #[command(flatten)] args: commands::dev::DevArgs },
    Build { #[command(flatten)] args: commands::build::BuildArgs },
    Test { #[command(flatten)] args: commands::test::TestArgs },
    Lint { #[command(flatten)] args: commands::lint::LintArgs },
    Format { #[command(flatten)] args: commands::format::FormatArgs },
    Check { #[command(flatten)] args: commands::check::CheckArgs },
    Bench { #[command(flatten)] args: commands::bench::BenchArgs },
    Start,
    RunScript { #[command(flatten)] args: commands::scripts::RunScriptArgs },
    Db { #[command(flatten)] args: commands::db::DbArgs },
    Prisma { #[command(subcommand)] action: commands::orm::PrismaAction },
    Drizzle { #[command(subcommand)] action: commands::orm::DrizzleAction },
    Typeorm { #[command(subcommand)] action: commands::orm::TypeOrmAction },
    MikroOrm { #[command(subcommand)] action: commands::orm::MikroOrmAction },
    Sequelize { #[command(subcommand)] action: commands::orm::SequelizeAction },
    Mongoose { #[command(subcommand)] action: commands::orm::MongooseAction },
    Kysely { #[command(subcommand)] action: commands::orm::KyselyAction },
    KnexCmd { #[command(subcommand)] action: commands::orm::KnexAction },
    Orm { #[command(subcommand)] action: commands::orm::OrmCommand },
    Install,
    Add { #[command(flatten)] args: commands::pm::AddArgs },
    Remove { #[command(flatten)] args: commands::pm::RemoveArgs },
    Uninstall { #[command(flatten)] args: commands::pm::RemoveArgs },
    Outdated,
    Update,
    Audit,
    Dedupe,
    Publish { #[command(flatten)] args: commands::registry::PublishArgs },
    Unpublish { #[command(flatten)] args: commands::registry::UnpublishArgs },
    Login { #[command(flatten)] args: commands::registry::LoginArgs },
    Logout { #[command(flatten)] args: commands::registry::LogoutArgs },
    Whoami,
    Search { #[command(flatten)] args: commands::registry::SearchArgs },
    InfoCmd { #[command(flatten)] args: commands::registry::InfoArgs },
    Workspace { #[command(subcommand)] action: commands::workspace::WorkspaceAction },
    Plugin { #[command(subcommand)] action: commands::plugin::PluginAction },
    Cache { #[command(subcommand)] action: commands::cache::CacheAction },
    Docker { #[command(subcommand)] action: commands::docker::DockerAction },
    Napi { #[command(subcommand)] action: commands::napi::NapiAction },
    Deploy { #[command(flatten)] args: commands::deploy::DeployArgs },
    Compat { #[command(flatten)] args: commands::compat::CompatArgs },
    Ai { #[command(flatten)] args: commands::ai::AiArgs },
    Init,
    Upgrade,
    Doctor,
    Info { #[command(flatten)] args: InfoArgs },
    Version,
    Clean { #[arg(long)] yes: bool },
    Telemetry { #[command(flatten)] args: commands::utils::TelemetryArgs },
    Config { #[command(flatten)] args: commands::utils::ConfigArgs },
    Laravel { #[command(subcommand)] action: commands::laravel::LaravelCommand },
    Serve { #[arg(long, default_value = "localhost")] host: String, #[arg(long, default_value_t = 3000)] port: u16, #[arg(long)] dir: Option<PathBuf>, #[arg(long)] watch: bool },
    Completions { shell: clap_complete::Shell },
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

    let info = serde_json::json!({
        "name": "klyron",
        "version": env!("CARGO_PKG_VERSION"),
        "engines": all_engines,
        "js_engines": js_engines["js_engines"],
        "best_engine": js_engines["best"],
        "adapters": {
            "count": ADAPTER_REGISTRY.lock().names().len(),
        }
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("Klyron v{}", info["version"]);
        let engines = info["engines"].as_array().unwrap().iter()
            .map(|v| v.as_str().unwrap()).collect::<Vec<_>>().join(", ");
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
        let adapter = registry.get(name).unwrap();
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
        return Ok(());
    }

    let framework = args.framework.as_deref().unwrap_or("");
    if framework.is_empty() {
        list_frameworks();
        return Ok(());
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

    let name = args.name.as_deref().unwrap_or("");
    if name.is_empty() {
        anyhow::bail!("Missing project name. Usage: klyron create <framework> <name> [options]");
    }

    let scaffold_args = commands::scaffold::ScaffoldArgs {
        name: name.to_string(),
        dir: args.dir,
        version: args.version,
        external: args.external,
        stack: args.stack,
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

    let cli = Cli::parse();

    if cli.quiet {
        set_verbosity(2);
    } else {
        set_verbosity(cli.verbose);
    }

    log_debug("Running klyron command");

    let engine = resolve_engine(cli.engine.as_deref())?;

    let result = dispatch_command(cli.command, engine);

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

pub fn dispatch_command(cmd: Commands, engine: Option<EngineRuntime>) -> anyhow::Result<()> {
    match cmd {
        Commands::Eval { args } => {
            if let Some(eng) = engine {
                let result = exec_with_engine(&eng, &args.code)?;
                if !result.is_empty() { println!("{result}"); }
                Ok(())
            } else {
                commands::runtime::run_eval(&args.code, args.module, all_extensions())
            }
        }
        Commands::Run { args } => {
            if let Some(eng) = engine {
                let result = exec_file_with_engine(&eng, &args.path)?;
                if !result.is_empty() { println!("{result}"); }
                Ok(())
            } else {
                commands::runtime::run_file(args, all_extensions())
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
        Commands::Install => commands::pm::run_install(),
        Commands::Add { args } => commands::pm::run_add(&args.packages, args.dev),
        Commands::Remove { args } => commands::pm::run_remove(&args.packages),
        Commands::Uninstall { args } => commands::pm::run_remove(&args.packages),
        Commands::Outdated => commands::pm::run_outdated(),
        Commands::Update => commands::pm::run_update(),
        Commands::Audit => commands::pm::run_audit(),
        Commands::Dedupe => commands::pm::run_dedupe(),
        Commands::Publish { args } => commands::registry::run_publish(&args),
        Commands::Unpublish { args } => commands::registry::run_unpublish(&args.name),
        Commands::Login { args } => commands::registry::run_login(args.registry.as_deref()),
        Commands::Logout { args } => commands::registry::run_logout(args.registry.as_deref()),
        Commands::Whoami => commands::registry::run_whoami(),
        Commands::Search { args } => commands::registry::run_search(&args.query),
        Commands::InfoCmd { args } => commands::registry::run_info(&args.package),
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
        Commands::Telemetry { args } => commands::utils::run_telemetry(args.enabled),
        Commands::Config { args } => commands::utils::run_config(args.key, args.value),
        Commands::Serve { host, port, dir, watch } => {
            let serve_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap());
            log_info(format!("Klyron dev server: http://{host}:{port}"));
            log_info(format!("Serving: {}", serve_dir.display()));
            if watch {
                let serve_dir_clone = serve_dir.clone();
                let host_clone = host.clone();
                watch_file(&serve_dir_clone, move || {
                    log_info(format!("Directory change detected, server running at http://{host_clone}:{port}"));
                });
            }
            start_dev_server(&host, port, &serve_dir)
        }
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
