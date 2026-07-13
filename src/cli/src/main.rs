mod engines;
use engines::{CEngine, CppEngine, TsEngine, PhpEngine, PyEngine, RbEngine};

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
  /// Compile and run C code
  Cc {
    source: String,
    #[arg(long)]
    args: Option<String>,
    #[arg(long)]
    watch: bool,
  },
  /// Compile and run C++ code
  Cxx {
    source: String,
    #[arg(long)]
    args: Option<String>,
    #[arg(long)]
    watch: bool,
  },
  /// Run TypeScript code
  Ts {
    source: String,
    #[arg(long)]
    watch: bool,
  },
  /// Run PHP code
  Php {
    source: String,
    #[arg(long)]
    watch: bool,
  },
  /// Run Python code
  Py {
    source: String,
    #[arg(long)]
    watch: bool,
  },
  /// Run Ruby code
  Rb {
    source: String,
    #[arg(long)]
    watch: bool,
  },
  /// Run Laravel Artisan commands
  Artisan {
    args: Vec<String>,
    #[arg(long)]
    project: Option<String>,
  },
  /// Run Composer commands
  Composer {
    args: Vec<String>,
    #[arg(long)]
    project: Option<String>,
  },
  /// Render a Blade template
  Blade {
    view: String,
    #[arg(long)]
    data: Option<String>,
    #[arg(long)]
    project: Option<String>,
  },
  /// Start an interactive Artisan Tinker REPL
  Tinker {
    #[arg(long)]
    project: Option<String>,
  },
  /// Scaffold a Next.js project
  CreateNextApp {
    name: String,
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,
  },
  /// Scaffold a React (Vite) project
  CreateReactApp {
    name: String,
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,
  },
  /// Scaffold a Laravel project
  CreateLaravel {
    name: String,
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,
  },
  /// Start a static file dev server
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

      if audit {
        if let Some(perms) = runtime.permissions() {
          for entry in perms.drain_audit_log() {
            println!("AUDIT: {} {} {} (allowed: {})", entry.timestamp, entry.operation, entry.resource, entry.allowed);
          }
        }
      }

      if watch {
        let path_clone = path.clone();
        watch_file(&path, move || {
          if let Ok(source) = std::fs::read_to_string(&path_clone) {
            match runtime.execute_script(path_clone.to_str().unwrap_or("<file>"), &source) {
              Ok(result) => { if !result.is_empty() && result != "undefined" { println!("{}", result); } }
              Err(e) => eprintln!("Error: {e}"),
            }
          }
        });
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
    Commands::Cc { source, args, watch } => {
      if watch {
        let path = PathBuf::from(&source);
        let code = std::fs::read_to_string(&path)
          .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", source))?;
        let mut engine = CEngine::new()?;
        let output = engine.exec(&code, args.as_deref())?;
        if !output.stdout.is_empty() { print!("{}", output.stdout); }
        if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        let watch_path = path.clone();
        let watch_args = args.clone();
        watch_file(&path, move || {
          let code = std::fs::read_to_string(&watch_path).unwrap_or_default();
          let mut engine = CEngine::new().unwrap();
          let output = engine.exec(&code, watch_args.as_deref()).unwrap();
          if !output.stdout.is_empty() { print!("{}", output.stdout); }
          if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        });
        return Ok(());
      }
      let mut engine = CEngine::new()?;
      let output = engine.exec(&source, args.as_deref())?;
      if !output.stdout.is_empty() { print!("{}", output.stdout); }
      if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
      std::process::exit(output.exit_code);
    }
    Commands::Cxx { source, args, watch } => {
      if watch {
        let path = PathBuf::from(&source);
        let code = std::fs::read_to_string(&path)
          .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", source))?;
        let mut engine = CppEngine::new()?;
        let output = engine.exec(&code, args.as_deref())?;
        if !output.stdout.is_empty() { print!("{}", output.stdout); }
        if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        let watch_path = path.clone();
        let watch_args = args.clone();
        watch_file(&path, move || {
          let code = std::fs::read_to_string(&watch_path).unwrap_or_default();
          let mut engine = CppEngine::new().unwrap();
          let output = engine.exec(&code, watch_args.as_deref()).unwrap();
          if !output.stdout.is_empty() { print!("{}", output.stdout); }
          if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        });
        return Ok(());
      }
      let mut engine = CppEngine::new()?;
      let output = engine.exec(&source, args.as_deref())?;
      if !output.stdout.is_empty() { print!("{}", output.stdout); }
      if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
      std::process::exit(output.exit_code);
    }
    Commands::Ts { source, watch } => {
      if watch {
        let path = PathBuf::from(&source);
        let code = std::fs::read_to_string(&path)
          .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", source))?;
        let mut engine = TsEngine::new()?;
        let output = engine.exec(&code)?;
        if !output.stdout.is_empty() { print!("{}", output.stdout); }
        if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        let watch_path = path.clone();
        watch_file(&path, move || {
          let code = std::fs::read_to_string(&watch_path).unwrap_or_default();
          let mut engine = TsEngine::new().unwrap();
          let output = engine.exec(&code).unwrap();
          if !output.stdout.is_empty() { print!("{}", output.stdout); }
          if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        });
        return Ok(());
      }
      let mut engine = TsEngine::new()?;
      let output = engine.exec(&source)?;
      if !output.stdout.is_empty() { print!("{}", output.stdout); }
      if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
      std::process::exit(output.exit_code);
    }
    Commands::Php { source, watch } => {
      if watch {
        let path = PathBuf::from(&source);
        let code = std::fs::read_to_string(&path)
          .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", source))?;
        let mut engine = PhpEngine::new()?;
        let output = engine.exec(&code)?;
        if !output.stdout.is_empty() { print!("{}", output.stdout); }
        if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        let watch_path = path.clone();
        watch_file(&path, move || {
          let code = std::fs::read_to_string(&watch_path).unwrap_or_default();
          let mut engine = PhpEngine::new().unwrap();
          let output = engine.exec(&code).unwrap();
          if !output.stdout.is_empty() { print!("{}", output.stdout); }
          if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        });
        return Ok(());
      }
      let mut engine = PhpEngine::new()?;
      let output = engine.exec(&source)?;
      if !output.stdout.is_empty() { print!("{}", output.stdout); }
      if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
      std::process::exit(output.exit_code);
    }
    Commands::Py { source, watch } => {
      if watch {
        let path = PathBuf::from(&source);
        let code = std::fs::read_to_string(&path)
          .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", source))?;
        let mut engine = PyEngine::new()?;
        let output = engine.exec(&code)?;
        if !output.stdout.is_empty() { print!("{}", output.stdout); }
        if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        let watch_path = path.clone();
        watch_file(&path, move || {
          let code = std::fs::read_to_string(&watch_path).unwrap_or_default();
          let mut engine = PyEngine::new().unwrap();
          let output = engine.exec(&code).unwrap();
          if !output.stdout.is_empty() { print!("{}", output.stdout); }
          if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        });
        return Ok(());
      }
      let mut engine = PyEngine::new()?;
      let output = engine.exec(&source)?;
      if !output.stdout.is_empty() { print!("{}", output.stdout); }
      if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
      std::process::exit(output.exit_code);
    }
    Commands::Rb { source, watch } => {
      if watch {
        let path = PathBuf::from(&source);
        let code = std::fs::read_to_string(&path)
          .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", source))?;
        let mut engine = RbEngine::new()?;
        let output = engine.exec(&code)?;
        if !output.stdout.is_empty() { print!("{}", output.stdout); }
        if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        let watch_path = path.clone();
        watch_file(&path, move || {
          let code = std::fs::read_to_string(&watch_path).unwrap_or_default();
          let mut engine = RbEngine::new().unwrap();
          let output = engine.exec(&code).unwrap();
          if !output.stdout.is_empty() { print!("{}", output.stdout); }
          if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
        });
        return Ok(());
      }
      let mut engine = RbEngine::new()?;
      let output = engine.exec(&source)?;
      if !output.stdout.is_empty() { print!("{}", output.stdout); }
      if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
      std::process::exit(output.exit_code);
    }
    Commands::Artisan { args, project } => {
      let mut engine = PhpEngine::new()?;
      let output = engine.artisan(&args.join(" "), project.as_deref())?;
      if !output.stdout.is_empty() { print!("{}", output.stdout); }
      if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
      std::process::exit(output.exit_code);
    }
    Commands::Composer { args, project } => {
      let mut engine = PhpEngine::new()?;
      let output = engine.composer(&args.join(" "), project.as_deref())?;
      if !output.stdout.is_empty() { print!("{}", output.stdout); }
      if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
      std::process::exit(output.exit_code);
    }
    Commands::Blade { view, data, project } => {
      let mut engine = PhpEngine::new()?;
      let output = engine.blade(&view, data.as_deref(), project.as_deref())?;
      if !output.stdout.is_empty() { print!("{}", output.stdout); }
      if !output.stderr.is_empty() { eprint!("{}", output.stderr); }
      std::process::exit(output.exit_code);
    }
    Commands::Tinker { project } => {
      let mut engine = PhpEngine::new()?;
      let output = engine.tinker(project.as_deref())?;
      std::process::exit(output.exit_code);
    }
    Commands::CreateNextApp { name, dir } => {
      let project_dir = dir.join(&name);
      if project_dir.exists() {
        anyhow::bail!("Directory already exists: {}", project_dir.display());
      }
      std::fs::create_dir_all(project_dir.join("pages/api"))?;
      std::fs::create_dir_all(project_dir.join("public"))?;
      std::fs::create_dir_all(project_dir.join("styles"))?;

      std::fs::write(
        project_dir.join("package.json"),
        r#"{
  "name": "APPNAME",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint"
  },
  "dependencies": {
    "next": "latest",
    "react": "latest",
    "react-dom": "latest"
  }
}
"#.replace("APPNAME", &name),
      )?;

      std::fs::write(
        project_dir.join("next.config.js"),
        "/** @type {import('next').NextConfig} */\nconst nextConfig = {\n  reactStrictMode: true,\n};\nmodule.exports = nextConfig;\n",
      )?;

      std::fs::write(
        project_dir.join("jsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./*"]
    }
  }
}
"#,
      )?;

      std::fs::write(
        project_dir.join("pages/index.js"),
        r#"import Head from 'next/head'
import styles from '../styles/Home.module.css'

export default function Home() {
  return (
    <div className={styles.container}>
      <Head>
        <title>Create Next App</title>
        <meta name="description" content="Generated by create next app" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <main className={styles.main}>
        <h1 className={styles.title}>
          Welcome to <a href="https://nextjs.org">Next.js!</a>
        </h1>

        <p className={styles.description}>
          Scaffolded by <code>klyron create-next-app</code>
        </p>
      </main>
    </div>
  )
}
"#,
      )?;

      std::fs::write(
        project_dir.join("pages/api/hello.js"),
        r#"export default function handler(req, res) {
  res.status(200).json({ name: 'John Doe' })
}
"#,
      )?;

      std::fs::write(
        project_dir.join("styles/globals.css"),
        "html,\nbody {\n  padding: 0;\n  margin: 0;\n  font-family: -apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Oxygen,\n    Ubuntu, Cantarell, Fira Sans, Droid Sans, Helvetica Neue, sans-serif;\n}\n\na {\n  color: inherit;\n  text-decoration: none;\n}\n\n* {\n  box-sizing: border-box;\n}\n",
      )?;

      std::fs::write(
        project_dir.join("styles/Home.module.css"),
        ".container {\n  min-height: 100vh;\n  padding: 0 0.5rem;\n  display: flex;\n  flex-direction: column;\n  justify-content: center;\n  align-items: center;\n}\n\n.main {\n  padding: 5rem 0;\n  flex: 1;\n  display: flex;\n  flex-direction: column;\n  justify-content: center;\n  align-items: center;\n}\n\n.title a {\n  color: #0070f3;\n  text-decoration: none;\n}\n\n.title a:hover,\n.title a:focus,\n.title a:active {\n  text-decoration: underline;\n}\n\n.title {\n  margin: 0;\n  line-height: 1.15;\n  font-size: 4rem;\n}\n\n.description {\n  line-height: 1.5;\n  font-size: 1.5rem;\n}\n",
      )?;

      std::fs::write(
        project_dir.join(".gitignore"),
        "# See https://help.github.com/articles/ignoring-files/ for more about ignoring files.\n\n# dependencies\n/node_modules\n/.pnp\n.pnp.js\n\n# testing\n/coverage\n\n# next.js\n/.next/\n/out/\n\n# production\n/build\n\n# misc\n.DS_Store\n*.pem\n\n# debug\nnpm-debug.log*\nyarn-debug.log*\nyarn-error.log*\n\n# local env files\n.env*.local\n\n# vercel\n.vercel\n\n# typescript\n*.tsbuildinfo\nnext-env.d.ts\n",
      )?;

      println!("✅ Created Next.js app at {}", project_dir.display());
      println!("\n  cd {}", project_dir.display());
      println!("  npm install");
      println!("  npm run dev\n");
      Ok(())
    }
    Commands::CreateReactApp { name, dir } => {
      let project_dir = dir.join(&name);
      if project_dir.exists() {
        anyhow::bail!("Directory already exists: {}", project_dir.display());
      }
      std::fs::create_dir_all(project_dir.join("src"))?;
      std::fs::create_dir_all(project_dir.join("public"))?;

      std::fs::write(
        project_dir.join("package.json"),
        r#"{
  "name": "APPNAME",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@vitejs/plugin-react": "^4.2.0",
    "vite": "^5.0.0"
  }
}
"#.replace("APPNAME", &name),
      )?;

      std::fs::write(
        project_dir.join("vite.config.js"),
        r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
})
"#,
      )?;

      std::fs::write(
        project_dir.join("index.html"),
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>APPNAME</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.jsx"></script>
  </body>
</html>
"#.replace("APPNAME", &name),
      )?;

      std::fs::write(
        project_dir.join("src/main.jsx"),
        r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.jsx'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
"#,
      )?;

      std::fs::write(
        project_dir.join("src/App.jsx"),
        r#"import { useState } from 'react'

function App() {
  const [count, setCount] = useState(0)

  return (
    <div>
      <h1>Vite + React</h1>
      <p>Scaffolded by <code>klyron create-react-app</code></p>
      <div>
        <button onClick={() => setCount((c) => c + 1)}>
          count is {count}
        </button>
      </div>
    </div>
  )
}

export default App
"#,
      )?;

      std::fs::write(
        project_dir.join("src/index.css"),
        "body {\n  margin: 0;\n  display: flex;\n  place-items: center;\n  min-width: 320px;\n  min-height: 100vh;\n}\n\nh1 {\n  font-size: 3.2em;\n  line-height: 1.1;\n}\n\nbutton {\n  border-radius: 8px;\n  border: 1px solid transparent;\n  padding: 0.6em 1.2em;\n  font-size: 1em;\n  font-weight: 500;\n  background: #1a1a1a;\n  color: #fff;\n  cursor: pointer;\n}\n\nbutton:hover {\n  border-color: #646cff;\n}\n",
      )?;

      std::fs::write(
        project_dir.join(".gitignore"),
        "# dependencies\n/node_modules\n/.pnp\n.pnp.js\n\n# build\n/dist\n\n# misc\n.DS_Store\n*.pem\n\n# debug\nnpm-debug.log*\nyarn-debug.log*\nyarn-error.log*\n\n# env\n.env.local\n.env.*.local\n",
      )?;

      println!("✅ Created React (Vite) app at {}", project_dir.display());
      println!("\n  cd {}", project_dir.display());
      println!("  npm install");
      println!("  npm run dev\n");
      Ok(())
    }
    Commands::CreateLaravel { name, dir } => {
      let project_dir = dir.join(&name);
      if project_dir.exists() {
        anyhow::bail!("Directory already exists: {}", project_dir.display());
      }
      std::fs::create_dir_all(project_dir.join("app/Http/Controllers"))?;
      std::fs::create_dir_all(project_dir.join("app/Models"))?;
      std::fs::create_dir_all(project_dir.join("bootstrap"))?;
      std::fs::create_dir_all(project_dir.join("config"))?;
      std::fs::create_dir_all(project_dir.join("database/migrations"))?;
      std::fs::create_dir_all(project_dir.join("public"))?;
      std::fs::create_dir_all(project_dir.join("resources/views"))?;
      std::fs::create_dir_all(project_dir.join("routes"))?;
      std::fs::create_dir_all(project_dir.join("storage/logs"))?;

      std::fs::write(
        project_dir.join("composer.json"),
        r#"{
  "name": "app/laravel",
  "description": "Laravel project scaffolded by klyron",
  "type": "project",
  "require": {
    "php": "^8.1",
    "laravel/framework": "^10.0"
  },
  "autoload": {
    "psr-4": {
      "App\\": "app/"
    }
  },
  "scripts": {
    "post-create-project-cmd": [
      "@php artisan key:generate"
    ]
  }
}
"#,
      )?;

      std::fs::write(
        project_dir.join(".env"),
        "APP_NAME=Laravel\nAPP_ENV=local\nAPP_DEBUG=true\nAPP_URL=http://localhost\n\nDB_CONNECTION=sqlite\nDB_DATABASE=database/database.sqlite\n",
      )?;

      std::fs::write(
        project_dir.join("routes/web.php"),
        "<?php\n\nuse Illuminate\\Support\\Facades\\Route;\n\nRoute::get('/', function () {\n    return view('welcome');\n});\n",
      )?;

      std::fs::write(
        project_dir.join("resources/views/welcome.blade.php"),
        "<!DOCTYPE html>\n<html>\n<head>\n    <title>Laravel</title>\n</head>\n<body>\n    <h1>Laravel Project</h1>\n    <p>Scaffolded by <code>klyron create-laravel</code></p>\n</body>\n</html>\n",
      )?;

      std::fs::write(
        project_dir.join("public/index.php"),
        "<?php\n\nuse Illuminate\\Contracts\\Http\\Kernel;\n\nrequire __DIR__.'/../vendor/autoload.php';\n\n$app = require_once __DIR__.'/../bootstrap/app.php';\n\n$kernel = $app->make(Kernel::class);\n\n$response = $kernel->handle(\n    $request = Illuminate\\Http\\Request::capture()\n)->send();\n\n$kernel->terminate($request, $response);\n",
      )?;

      std::fs::write(
        project_dir.join("bootstrap/app.php"),
        "<?php\n\n$app = new Illuminate\\Foundation\\Application(\n    $_ENV['APP_BASE_PATH'] ?? dirname(__DIR__)\n);\n\n$app->singleton(\n    Illuminate\\Contracts\\Http\\Kernel::class,\n    App\\Http\\Kernel::class\n);\n\nreturn $app;\n",
      )?;

      std::fs::write(
        project_dir.join(".gitignore"),
        "/vendor\n/node_modules\n/.env\n/storage/*.log\n",
      )?;

      println!("✅ Created Laravel app at {}", project_dir.display());
      println!("\n  cd {}", project_dir.display());
      println!("  composer install");
      println!("  cp .env.example .env (if needed)");
      println!("  php artisan key:generate\n");
      Ok(())
    }
    Commands::Serve { host, port, dir, watch } => {
      let serve_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap());
      println!("Klyron dev server: http://{host}:{port}");
      println!("Serving: {}", serve_dir.display());
      if watch {
        let serve_dir_clone = serve_dir.clone();
        let host_clone = host.clone();
        let port_clone = port;
        watch_file(&serve_dir_clone, move || {
          println!("Directory change detected, server running at http://{host_clone}:{port_clone}");
        });
      }
      start_dev_server(&host, port, &serve_dir)
    }
  }
}

fn watch_file(path: &PathBuf, on_change: impl Fn()) {
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
            let _ = tx.send(());
          }
        }
      }
    }
  });
  while rx.recv().is_ok() {
    print!("\n\u{1b}[2K\u{1b}[GFile changed. Re-running...\n");
    std::io::stdout().flush().ok();
    on_change();
    print!("> ");
    std::io::stdout().flush().ok();
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
    Ok(())
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
