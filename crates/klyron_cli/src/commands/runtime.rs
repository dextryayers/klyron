use std::io::Write;
use std::path::PathBuf;
use clap::Args;
use klyron_core::{Runtime, permissions::{PermissionSet, PolicyTemplate, SandboxLevel}};

#[derive(Args)]
pub struct EvalArgs {
    pub code: String,
    #[arg(long)]
    pub policy: Option<PolicyTemplate>,
    #[arg(short, long)]
    pub module: bool,
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

pub fn run_file(args: RunArgs, extensions: Vec<deno_core::Extension>) -> anyhow::Result<()> {
    if args.sandbox.is_sandboxed() {
        klyron_core::sandbox::Sandbox::apply(args.sandbox, args.max_memory, args.max_cpu, args.max_fds)
            .map_err(|e| anyhow::anyhow!("Sandbox: {e}"))?;
    }
    let mut perm_set = PermissionSet::default();
    if !args.allow_read.is_empty() { perm_set.allow_read = args.allow_read; }
    if !args.deny_read.is_empty() { perm_set.deny_read = args.deny_read; }
    if !args.allow_write.is_empty() { perm_set.allow_write = args.allow_write; }
    if !args.deny_write.is_empty() { perm_set.deny_write = args.deny_write; }
    if !args.allow_net.is_empty() { perm_set.allow_net = args.allow_net; }
    if !args.deny_net.is_empty() { perm_set.deny_net = args.deny_net; }
    if !args.allow_env.is_empty() { perm_set.allow_env = args.allow_env; }
    if !args.deny_env.is_empty() { perm_set.deny_env = args.deny_env; }
    if !args.allow_run.is_empty() { perm_set.allow_run = args.allow_run; }
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

    if !args.path.exists() {
        anyhow::bail!("File not found: {}", args.path.display());
    }
    let source = std::fs::read_to_string(&args.path)?;
    let result = runtime.execute_script(args.path.to_str().unwrap_or("<file>"), &source)?;
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
                println!("  .engine <name>  Switch to engine (v8, boa, quickjs, jsc, auto)");
            }
            ".clear" => { print!("\x1B[2J\x1B[1;1H"); std::io::stdout().flush()?; }
            ".version" => println!("Klyron v{}", env!("CARGO_PKG_VERSION")),
            ".engine" => {
                match current_engine.as_ref() {
                    Some(e) => println!("Current engine: {}", e.kind()),
                    None => println!("Current engine: Deno Core (V8)"),
                }
                println!("Available: v8, boa, quickjs, jsc, auto. Use `.engine <name>` to switch.");
            }
            s if s.starts_with(".engine ") => {
                let name = s.trim_start_matches(".engine ").trim();
                match name {
                    "v8" | "boa" | "quickjs" | "jsc" | "auto" => {
                        let kind = match name {
                            "v8" => klyron_engine::JsEngineKind::V8,
                            "boa" => klyron_engine::JsEngineKind::Boa,
                            "quickjs" => klyron_engine::JsEngineKind::QuickJS,
                            "jsc" => klyron_engine::JsEngineKind::JSC,
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
                    other => eprintln!("Unknown engine '{other}'. Available: v8, boa, quickjs, jsc, auto"),
                }
            }
            "" => continue,
            _ => {
                if let Some(ref eng) = current_engine {
                    match eng.eval(input) {
                        Ok(result) => { if !result.is_empty() { println!("{result}"); } }
                        Err(e) => eprintln!("Error: {e}"),
                    }
                } else {
                    let runtime = Runtime::builder()
                        .async_(true)
                        .enable_typescript(true)
                        .extensions(crate::all_extensions())
                        .build()?;
                    match runtime.eval(input) {
                        Ok(result) => { if !result.is_empty() && result != "undefined" { println!("{}", result); } }
                        Err(e) => eprintln!("Error: {e}"),
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
        let status = std::process::Command::new("sh")
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
