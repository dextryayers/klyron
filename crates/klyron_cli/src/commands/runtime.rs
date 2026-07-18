use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use clap::Args;
use klyron_core::{Runtime, permissions::{PermissionSet, PolicyTemplate, SandboxLevel}};

#[derive(Args)]
pub struct EvalArgs {
    pub code: String,
    #[arg(long)]
    pub policy: Option<PolicyTemplate>,
    #[arg(short, long)]
    pub module: bool,
    #[arg(long, alias = "ts")]
    pub typescript: bool,
    #[arg(long)]
    pub jsx: bool,
}

#[derive(Args)]
pub struct RunArgs {
    pub path: PathBuf,
    #[arg(long)]
    pub policy: Option<PolicyTemplate>,
    #[arg(long, default_value = "none")]
    pub sandbox: SandboxLevel,
    #[arg(long)] pub allow_read: Vec<String>,
    #[arg(long)] pub allow_write: Vec<String>,
    #[arg(long)] pub allow_net: Vec<String>,
    #[arg(long)] pub allow_env: Vec<String>,
    #[arg(long)] pub allow_run: Vec<String>,
    #[arg(long)] pub allow_read_all: bool,
    #[arg(long)] pub allow_write_all: bool,
    #[arg(long)] pub allow_net_all: bool,
    #[arg(long)] pub allow_env_all: bool,
    #[arg(long)] pub allow_ffi: bool,
    #[arg(long)] pub allow_sys: bool,
    #[arg(long)] pub deny_read: Vec<String>,
    #[arg(long)] pub deny_write: Vec<String>,
    #[arg(long)] pub deny_net: Vec<String>,
    #[arg(long)] pub deny_env: Vec<String>,
    #[arg(long)] pub prompt: bool,
    #[arg(long)] pub audit: bool,
    #[arg(long)] pub watch: bool,
    #[arg(long)] pub max_memory: Option<u64>,
    #[arg(long)] pub max_cpu: Option<u64>,
    #[arg(long)] pub max_fds: Option<u64>,
}

pub fn run_eval(code: &str, _module: bool, extensions: Vec<deno_core::Extension>) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    crate::load_dotenv(&dir);

    if let Some(tsconfig) = crate::detect_tsconfig(&dir) {
        let opts = crate::apply_tsconfig_compiler_options(&tsconfig);
        if !opts.is_empty() {
            crate::log_debug(format!("tsconfig.json options: {}", opts.join(" ")));
        }
    }

    let runtime = Runtime::builder()
        .async_(true)
        .enable_typescript(true)
        .extensions(extensions)
        .build()?;
    let result = runtime.eval(code)?;
    if !result.is_empty() {
        println!("{}", result);
    }
    Ok(())
}

pub fn run_eval_typescript(code: &str, jsx: bool) -> anyhow::Result<()> {
    use klyron_transpiler::{transpile_ts_to_js, TranspileOptions, Lang, Target};
    let lang = if jsx { Lang::Tsx } else { Lang::TypeScript };
    let _options = TranspileOptions { lang, target: Target::EsNext, minify: false, sourcemap: false };
    let js_code = transpile_ts_to_js(code)?;
    let js_code = if js_code.is_empty() { code.to_string() } else { js_code };

    let runtime = Runtime::builder()
        .async_(true)
        .enable_typescript(true)
        .extensions(crate::all_extensions())
        .build()?;
    let result = runtime.eval(&js_code)?;
    if !result.is_empty() {
        println!("{}", result);
    }
    Ok(())
}

pub fn run_file(args: RunArgs, extensions: Vec<deno_core::Extension>) -> anyhow::Result<()> {
    let watch = args.watch;
    let path = args.path.clone();

    crate::anim::cmd_header("run", &format!("Running {}", path.display()));

    let dir = std::env::current_dir()?;
    crate::load_dotenv(&dir);

    if let Some(tsconfig) = crate::detect_tsconfig(&dir) {
        let opts = crate::apply_tsconfig_compiler_options(&tsconfig);
        if !opts.is_empty() {
            crate::log_debug(format!("tsconfig.json options: {}", opts.join(" ")));
        }
    }

    if !dir.join("klyron.json").exists() {
        let _ = crate::auto_create_klyron_config(&dir);
    }

    if watch {
        return run_file_watch(path);
    }

    if args.sandbox.is_sandboxed() {
        klyron_core::sandbox::Sandbox::apply(args.sandbox, args.max_memory, args.max_cpu, args.max_fds)
            .map_err(|e| anyhow::anyhow!("Sandbox: {e}"))?;
    }
    let mut perm_set = PermissionSet::default();
    perm_set.allow_read = args.allow_read.clone();
    perm_set.deny_read = args.deny_read.clone();
    perm_set.allow_write = args.allow_write.clone();
    perm_set.deny_write = args.deny_write.clone();
    perm_set.allow_net = args.allow_net.clone();
    perm_set.deny_net = args.deny_net.clone();
    perm_set.allow_env = args.allow_env.clone();
    perm_set.deny_env = args.deny_env.clone();
    perm_set.allow_run = args.allow_run.clone();
    if args.allow_ffi { perm_set.allow_ffi = true; }
    if args.allow_sys { perm_set.allow_sys = true; }
    if args.allow_read_all { perm_set.allow_read_all = true; }
    if args.allow_write_all { perm_set.allow_write_all = true; }
    if args.allow_net_all { perm_set.allow_net_all = true; }
    if args.allow_env_all { perm_set.allow_env_all = true; }
    perm_set.prompt = args.prompt;
    perm_set.sandbox = args.sandbox;
    perm_set.max_memory = args.max_memory;
    perm_set.max_cpu = args.max_cpu;
    perm_set.max_fds = args.max_fds;

    let runtime = Runtime::builder()
        .async_(true)
        .enable_typescript(true)
        .permissions(perm_set)
        .extensions(extensions)
        .build()?;

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }
    let source = std::fs::read_to_string(&path)?;
    let result = runtime.execute_script(path.to_str().unwrap_or("<file>"), &source)?;
    if !result.is_empty() && result != "undefined" {
        println!("{result}");
    }
    if args.audit {
        if let Some(perms) = runtime.permissions() {
            for entry in perms.drain_audit_log() {
                println!("AUDIT: {} {} {} (allowed: {})", entry.timestamp, entry.operation, entry.resource, entry.allowed);
            }
        }
    }
    Ok(())
}

fn run_file_watch(file_path: PathBuf) -> anyhow::Result<()> {
    let path = file_path.clone();
    let path_str = path.display().to_string();

    println!("Watching {} for changes...", path_str);

    let watcher = klyron_watcher::WatcherBuilder::new()
        .add_path(&path_str)
        .debounce(300)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to start watcher: {e}"))?;

    let run_once = {
        let path = path.clone();
        move || -> anyhow::Result<()> {
            let extensions = crate::all_extensions();
            let runtime = Runtime::builder()
                .async_(true)
                .enable_typescript(true)
                .permissions(PermissionSet::default())
                .extensions(extensions)
                .build()?;
            let source = std::fs::read_to_string(&path)?;
            let result = runtime.execute_script(path.to_str().unwrap_or("<file>"), &source)?;
            if !result.is_empty() && result != "undefined" {
                println!("{}", result);
            }
            Ok(())
        }
    };

    let _ = run_once();

    let watch_path = path.clone();
    watcher.start(move |event| {
        use klyron_watcher::WatchEvent;
        match event {
            WatchEvent::Modify(p) | WatchEvent::Any(p) if p == watch_path => {
                println!("\n\x1b[2K\x1b[GFile changed: {}", p.display());
                match run_once() {
                    Ok(_) => {},
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            _ => {}
        }
    }).map_err(|e| anyhow::anyhow!("Watcher error: {e}"))?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}

pub fn repl_loop() -> anyhow::Result<()> {
    repl_loop_ext(None)
}

pub fn repl_loop_ext(engine: Option<klyron_engine::EngineRuntime>) -> anyhow::Result<()> {
    let engine_kind = engine.as_ref().map(|e| e.kind().to_string());
    println!("Klyron REPL v{}", env!("CARGO_PKG_VERSION"));
    print!("JS Engine: ");
    if let Some(ref kind) = engine_kind {
        println!("{kind} (--engine override)");
    } else {
        println!("Deno Core (V8)");
    }
    println!("Type '.help' for help, '.exit' to quit");
    let mut current_engine = engine;
    loop {
        let mut input = String::new();
        std::io::Write::flush(&mut std::io::stdout())?;
        if std::io::stdin().read_line(&mut input)? == 0 { break; }
        let input = input.trim();
        match input {
            ".exit" | ".quit" => break,
            ".help" => {
                println!("REPL Commands:");
                println!("  .exit / .quit  Exit REPL");
                println!("  .help           Show this help");
                println!("  .clear          Clear screen");
                println!("  .version        Show version");
                println!("  .engine         Show/switch engine");
                println!("  .engine <name>  Switch to engine (boa, quickjs, auto)");
                println!("  .color          Toggle colored output");
            }
            ".clear" => { print!("\x1B[2J\x1B[1;1H"); std::io::stdout().flush()?; }
            ".version" => println!("Klyron v{}", env!("CARGO_PKG_VERSION")),
            ".engine" => {
                match current_engine.as_ref() {
                    Some(e) => println!("Current engine: {}", e.kind()),
                    None => println!("Current engine: Deno Core (V8)"),
                }
                println!("Available: boa, quickjs, auto. Use `.engine <name>` to switch.");
            }
            s if s.starts_with(".engine ") => {
                let name = s.trim_start_matches(".engine ").trim();
                match name {
                    "boa" | "quickjs" | "auto" => {
                        let kind = match name {
                            "boa" => klyron_engine::JsEngineKind::Boa,
                            "quickjs" => klyron_engine::JsEngineKind::QuickJS,
                            "auto" => klyron_engine::detect_best_engine(),
                            _ => unreachable!(),
                        };
                        match klyron_engine::EngineRuntime::new(kind) {
                            Ok(eng) => {
                                current_engine = Some(eng);
                                println!("Switched to engine: {kind}");
                            }
                            Err(e) => eprintln!("Failed to switch to {name}: {e}"),
                        }
                    }
                    other => eprintln!("Unknown engine '{other}'. Available: boa, quickjs, auto"),
                }
            }
            "" => continue,
            _ => {
                if let Some(ref eng) = current_engine {
                    match eng.eval(input) {
                        Ok(result) => {
                            if !result.is_empty() {
                                println!("{}", klyron_shell::colorize_output(&result));
                            }
                        }
                        Err(e) => eprintln!("\x1b[31mError: {e}\x1b[0m"),
                    }
                } else {
                    let runtime = Runtime::builder()
                        .async_(true)
                        .enable_typescript(true)
                        .extensions(crate::all_extensions())
                        .build()?;
                    match runtime.eval(input) {
                        Ok(result) => {
                            if !result.is_empty() && result != "undefined" {
                                println!("{}", klyron_shell::colorize_output(&result));
                            }
                        }
                        Err(e) => eprintln!("\x1b[31mError: {e}\x1b[0m"),
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn shell_loop() -> anyhow::Result<()> {
    println!("Klyron Shell — type commands or JS/TS expressions");
    println!("Type 'exit' to quit");
    loop {
        let mut input = String::new();
        print!("$ ");
        std::io::Write::flush(&mut std::io::stdout())?;
        if std::io::stdin().read_line(&mut input)? == 0 { break; }
        let input = input.trim();
        if input.is_empty() || input == "exit" || input == ".exit" { break; }
        let status = Command::new("sh")
            .arg("-c").arg(input)
            .status();
        match status {
            Ok(s) if !s.success() => eprintln!("exit code: {}", s),
            Err(e) => eprintln!("error: {e}"),
            _ => {}
        }
    }
    Ok(())
}

pub fn eval_with_args(args: EvalArgs) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    crate::load_dotenv(&dir);

    if let Some(tsconfig) = crate::detect_tsconfig(&dir) {
        let opts = crate::apply_tsconfig_compiler_options(&tsconfig);
        if !opts.is_empty() {
            crate::log_debug(format!("tsconfig.json options: {}", opts.join(" ")));
        }
    }

    if args.typescript || args.jsx {
        return run_eval_typescript(&args.code, args.jsx);
    }

    let runtime = Runtime::builder()
        .async_(true)
        .enable_typescript(true)
        .extensions(crate::all_extensions())
        .build()?;

    let result = if args.module {
        runtime.execute_script("<eval>", &args.code)?
    } else {
        runtime.eval(&args.code)?
    };

    if !result.is_empty() && result != "undefined" {
        println!("{}", result);
    }
    Ok(())
}

pub fn shell_eval_line(code: &str) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::builder()
        .async_(true)
        .enable_typescript(true)
        .extensions(crate::all_extensions())
        .build()?;
    let result = runtime.eval(code)?;
    if !result.is_empty() && result != "undefined" {
        println!("{}", result);
    }
    Ok(())
}
