use std::path::PathBuf;

use clap::{Parser, Subcommand};
use klyron_core::{
  permissions::{PermissionSet, PolicyTemplate, SandboxLevel},
  Runtime,
};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "klyron", version, about = "Klyron JS - Universal Polyglot Runtime", long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Evaluate JavaScript/TypeScript code
  Eval {
    code: String,
    #[arg(long)]
    policy: Option<PolicyTemplate>,
    #[arg(long, short)]
    module: bool,
  },
  /// Run a JavaScript/TypeScript file
  Run {
    path: PathBuf,
    #[arg(long)]
    policy: Option<PolicyTemplate>,
    #[arg(long, default_value = "none")]
    sandbox: SandboxLevel,
    #[arg(long)]
    allow_read: Vec<String>,
    #[arg(long)]
    allow_write: Vec<String>,
    #[arg(long)]
    allow_net: Vec<String>,
    #[arg(long)]
    allow_env: Vec<String>,
    #[arg(long)]
    allow_run: Vec<String>,
    #[arg(long)]
    allow_read_all: bool,
    #[arg(long)]
    allow_write_all: bool,
    #[arg(long)]
    allow_net_all: bool,
    #[arg(long)]
    allow_env_all: bool,
    #[arg(long)]
    allow_ffi: bool,
    #[arg(long)]
    allow_sys: bool,
    #[arg(long)]
    deny_read: Vec<String>,
    #[arg(long)]
    deny_write: Vec<String>,
    #[arg(long)]
    deny_net: Vec<String>,
    #[arg(long)]
    deny_env: Vec<String>,
    #[arg(long)]
    prompt: bool,
    #[arg(long)]
    audit: bool,
    #[arg(long)]
    watch: bool,
    #[arg(long)]
    max_memory: Option<u64>,
    #[arg(long)]
    max_cpu: Option<u64>,
    #[arg(long)]
    max_fds: Option<u64>,
  },
  /// Start an interactive REPL
  Repl,
  /// Bundle dependencies into a single file
  Bundle {
    entry: PathBuf,
    #[arg(long, default_value = "bundle.js")]
    output: PathBuf,
    #[arg(long)]
    minify: bool,
  },
  /// Start a development server
  Serve {
    #[arg(long, default_value = "localhost")]
    host: String,
    #[arg(long, default_value_t = 3000)]
    port: u16,
    #[arg(long)]
    dir: Option<PathBuf>,
    #[arg(long)]
    watch: bool,
  },
}

fn all_extensions() -> Vec<deno_core::Extension> {
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
  ]
}

fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_target(false)
    .init();

  let cli = Cli::parse();

  match cli.command {
    Commands::Eval { code, policy, module: _ } => {
      let perm_set = if let Some(tmpl) = policy { tmpl.apply() } else { PermissionSet::default() };
      let runtime = Runtime::builder()
        .async_(true)
        .enable_typescript(true)
        .permissions(perm_set)
        .extensions(all_extensions())
        .build()?;
      let result = runtime.eval(&code)?;
      println!("{}", result);
      Ok(())
    }
    Commands::Run {
      path,
      policy,
      sandbox,
      allow_read,
      allow_write,
      allow_net,
      allow_env,
      allow_run,
      allow_read_all,
      allow_write_all,
      allow_net_all,
      allow_env_all,
      allow_ffi,
      allow_sys,
      deny_read,
      deny_write,
      deny_net,
      deny_env,
      prompt,
      audit,
      watch,
      max_memory,
      max_cpu,
      max_fds,
    } => {
      if sandbox.is_sandboxed() {
        klyron_core::sandbox::Sandbox::apply(sandbox, max_memory, max_cpu, max_fds)
          .map_err(|e| anyhow::anyhow!("Sandbox: {}", e))?;
      }

      let mut perm_set = if let Some(tmpl) = policy { tmpl.apply() } else { PermissionSet::default() };

      if !allow_read.is_empty() { perm_set.allow_read = allow_read; }
      if !deny_read.is_empty() { perm_set.deny_read = deny_read; }
      if !allow_write.is_empty() { perm_set.allow_write = allow_write; }
      if !deny_write.is_empty() { perm_set.deny_write = deny_write; }
      if !allow_net.is_empty() { perm_set.allow_net = allow_net; }
      if !deny_net.is_empty() { perm_set.deny_net = deny_net; }
      if !allow_env.is_empty() { perm_set.allow_env = allow_env; }
      if !deny_env.is_empty() { perm_set.deny_env = deny_env; }
      if !allow_run.is_empty() { perm_set.allow_run = allow_run; }
      if allow_ffi { perm_set.allow_ffi = true; }
      if allow_sys { perm_set.allow_sys = true; }
      if allow_read_all { perm_set.allow_read_all = true; }
      if allow_write_all { perm_set.allow_write_all = true; }
      if allow_net_all { perm_set.allow_net_all = true; }
      if allow_env_all { perm_set.allow_env_all = true; }
      perm_set.prompt = prompt;
      perm_set.sandbox = sandbox;
      perm_set.max_memory = max_memory;
      perm_set.max_cpu = max_cpu;
      perm_set.max_fds = max_fds;

      let runtime = Runtime::builder()
        .async_(true)
        .enable_typescript(true)
        .permissions(perm_set)
        .extensions(all_extensions())
        .build()?;

      if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
      }

      let source = std::fs::read_to_string(&path)?;
      let result = runtime.execute_script(path.to_str().unwrap_or("<file>"), &source)?;

      if !result.is_empty() && result != "undefined" {
        println!("{}", result);
      }

      if watch {
        watch_and_reload(&path, &runtime)?;
      }

      if audit {
        if let Some(perms) = runtime.permissions() {
          for entry in perms.drain_audit_log() {
            println!("AUDIT: {} {} {} (allowed: {})", entry.timestamp, entry.operation, entry.resource, entry.allowed);
          }
        }
      }

      Ok(())
    }
    Commands::Repl => {
      println!("Klyron REPL v{}", env!("CARGO_PKG_VERSION"));
      println!("Type '.help' for help, '.exit' to quit");
      repl_loop()
    }
    Commands::Bundle { entry, output, minify: _ } => {
      let source = std::fs::read_to_string(&entry)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", entry.display()))?;
      let runtime = Runtime::builder()
        .enable_typescript(true)
        .extensions(all_extensions())
        .build()?;
      let js = runtime.execute_script(entry.to_str().unwrap_or("<entry>"), &source)?;
      std::fs::write(&output, js)
        .map_err(|e| anyhow::anyhow!("Cannot write {}: {e}", output.display()))?;
      println!("Bundled {} -> {}", entry.display(), output.display());
      Ok(())
    }
    Commands::Serve { host, port, dir, watch: _ } => {
      let serve_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap());
      println!("Klyron dev server: http://{host}:{port}");
      println!("Serving: {}", serve_dir.display());
      start_dev_server(&host, port, &serve_dir)?;
      Ok(())
    }
  }
}

fn watch_and_reload(path: &PathBuf, runtime: &Runtime) -> anyhow::Result<()> {
  use std::io::Write;
  let (tx, rx) = std::sync::mpsc::channel();
  let path_clone = path.clone();
  std::thread::spawn(move || {
    let mut last_modified = std::time::SystemTime::now();
    loop {
      std::thread::sleep(std::time::Duration::from_millis(500));
      if let Ok(metadata) = std::fs::metadata(&path_clone) {
        if let Ok(modified) = metadata.modified() {
          if modified > last_modified {
            last_modified = modified;
            let _ = tx.send(true);
          }
        }
      }
    }
  });
  loop {
    if rx.recv().is_ok() {
      print!("\n\u{1b}[2K\u{1b}[GFile changed. Re-running...\n> ");
      std::io::stdout().flush()?;
      if let Ok(source) = std::fs::read_to_string(path) {
        match runtime.execute_script(path.to_str().unwrap_or("<file>"), &source) {
          Ok(result) => {
            if !result.is_empty() && result != "undefined" {
              println!("{}", result);
            }
          }
          Err(e) => eprintln!("Error: {e}"),
        }
      }
      print!("> ");
      std::io::stdout().flush()?;
    }
  }
}

fn start_dev_server(host: &str, port: u16, dir: &std::path::Path) -> anyhow::Result<()> {
  let rt = tokio::runtime::Runtime::new()?;
  rt.block_on(async {
    let service = tower_http::services::ServeDir::new(dir)
      .append_index_html_on_directories(true);
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await
      .map_err(|e| anyhow::anyhow!("Cannot bind {addr}: {e}"))?;
    println!("Listening on http://{addr}");
    axum::serve(listener, axum::routing::any_service(service))
      .await
      .map_err(|e| anyhow::anyhow!("Server error: {e}"))?;
    Ok::<_, anyhow::Error>(())
  })
}

fn repl_loop() -> anyhow::Result<()> {
  let runtime = Runtime::builder()
    .async_(true)
    .enable_typescript(true)
    .extensions(all_extensions())
    .build()?;

  loop {
    let mut input = String::new();
    print!("> ");
    use std::io::Write;
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;

    let input = input.trim();
    match input {
      ".exit" | ".quit" => break,
      ".help" => {
        println!("Commands:\n  .exit, .quit  Exit\n  .help         Show this help\n  .clear        Clear console\n  .version      Show version");
        continue;
      }
      ".clear" => {
        print!("\u{1b}[2J\u{1b}[H");
        std::io::stdout().flush()?;
        continue;
      }
      ".version" => {
        println!("Klyron v{}", env!("CARGO_PKG_VERSION"));
        continue;
      }
      "" => continue,
      _ => {}
    }

    match runtime.eval(input) {
      Ok(result) => {
        if !result.is_empty() && result != "undefined" {
          println!("{}", result);
        }
      }
      Err(e) => eprintln!("Error: {}", e),
    }
  }

  Ok(())
}
