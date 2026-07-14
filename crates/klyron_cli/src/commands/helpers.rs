use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub fn detect_project_type(dir: &Path) -> &'static str {
    if dir.join("composer.json").exists() { return "laravel"; }
    if dir.join("package.json").exists() { return "node"; }
    if dir.join("Cargo.toml").exists() { return "rust"; }
    if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() || dir.join("manage.py").exists() { return "python"; }
    if dir.join("Gemfile").exists() { return "ruby"; }
    if dir.join("go.mod").exists() { return "go"; }
    if dir.join("build.zig").exists() { return "zig"; }
    if dir.join("Makefile").exists() { return "make"; }
    if dir.join("deno.json").exists() { return "deno"; }
    "unknown"
}

pub fn detect_package_runner(dir: &Path) -> &'static str {
    if dir.join("bun.lock").exists() || dir.join("bun.lockb").exists() { "bun" }
    else if dir.join("pnpm-lock.yaml").exists() { "pnpm" }
    else if dir.join("yarn.lock").exists() { "yarn" }
    else if dir.join("npm-shrinkwrap.json").exists() { "npm" }
    else { "npm" }
}

pub fn run_cmd(program: &str, args: &[&str], dir: &Path) -> anyhow::Result<()> {
    run_cmd_str(program, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(), dir)
}

pub fn run_cmd_str(program: &str, args: &[String], dir: &Path) -> anyhow::Result<()> {
    let status = StdCommand::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run {}: {e}", program))?;
    if !status.success() {
        anyhow::bail!("{} exited with code {}", program, status);
    }
    Ok(())
}

pub fn write_files(base: &Path, files: Vec<(&str, &str)>) -> anyhow::Result<()> {
    for (name, content) in files {
        let path = base.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
    }
    Ok(())
}

pub fn watch_file(path: &PathBuf, on_change: impl Fn()) {
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

pub fn run_npx(args: &[&str], dir: &Path) -> anyhow::Result<()> {
    let status = StdCommand::new("npx")
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run npx: {e}"))?;
    if !status.success() {
        anyhow::bail!("npx {} exited with code {}", args.join(" "), status);
    }
    Ok(())
}

pub fn mkdirs(base: &Path, dirs: &[&str]) -> anyhow::Result<()> {
    for d in dirs {
        std::fs::create_dir_all(base.join(d))?;
    }
    Ok(())
}

fn has_npm_dep(dir: &Path, dep: &str) -> Option<bool> {
    let pkg = dir.join("package.json");
    let content = std::fs::read_to_string(pkg).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        if deps.contains_key(dep) { return Some(true); }
    }
    if let Some(deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
        if deps.contains_key(dep) { return Some(true); }
    }
    Some(false)
}

pub fn detect_orm(dir: &Path) -> String {
    if dir.join("schema.prisma").exists() || dir.join("prisma/schema.prisma").exists() {
        return "prisma".to_string();
    }
    if dir.join("drizzle.config.ts").exists() || dir.join("drizzle.config.js").exists() {
        return "drizzle".to_string();
    }
    if dir.join("ormconfig.json").exists() || dir.join("ormconfig.ts").exists() || dir.join("ormconfig.js").exists() {
        return "typeorm".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "typeorm") {
        return "typeorm".to_string();
    }
    if dir.join("mikro-orm.config.ts").exists() || dir.join("mikro-orm.config.js").exists() {
        return "mikroorm".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "@mikro-orm/core") {
        return "mikroorm".to_string();
    }
    if dir.join("config/config.json").exists() && has_npm_dep(dir, "sequelize").unwrap_or(false) {
        return "sequelize".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "sequelize") {
        if dir.join("models/index.js").exists() || dir.join("models").exists() {
            return "sequelize".to_string();
        }
    }
    if let Some(true) = has_npm_dep(dir, "mongoose") {
        return "mongoose".to_string();
    }
    if dir.join("kysely.ts").exists() || dir.join("kysely.config.ts").exists() {
        return "kysely".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "kysely") {
        return "kysely".to_string();
    }
    if dir.join("knexfile.ts").exists() || dir.join("knexfile.js").exists() {
        return "knex".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "knex") {
        return "knex".to_string();
    }
    "unknown".to_string()
}

pub fn start_dev_server(host: &str, port: u16, dir: &Path) -> anyhow::Result<()> {
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

// ── .env file auto-loading ───────────────────────────────────────────────

pub fn load_dotenv(dir: &Path) {
    let files = get_dotenv_files(dir);
    for path in files {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some(eq_idx) = line.find('=') {
                    let key = line[..eq_idx].trim();
                    let value = line[eq_idx + 1..].trim();
                    if !key.is_empty() && std::env::var(key).is_err() {
                        std::env::set_var(key, value);
                    }
                }
            }
        }
    }
}

fn get_dotenv_files(dir: &Path) -> Vec<PathBuf> {
    let node_env = std::env::var("NODE_ENV").unwrap_or_default();
    let mut files = Vec::new();
    let local = dir.join(".env.local");
    if local.exists() { files.push(local); }
    if !node_env.is_empty() {
        let specific = dir.join(format!(".env.{}", node_env));
        if specific.exists() { files.push(specific); }
    }
    let env = dir.join(".env");
    if env.exists() { files.push(env); }
    files
}

// ── tsconfig.json auto-detection ────────────────────────────────────────

pub fn detect_tsconfig(dir: &Path) -> Option<serde_json::Value> {
    for name in &["tsconfig.json", "tsconfig.ts", "jsconfig.json"] {
        let path = dir.join(name);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    return Some(json);
                }
            }
        }
    }
    None
}

pub fn apply_tsconfig_compiler_options(tsconfig: &serde_json::Value) -> Vec<String> {
    let mut opts = Vec::new();
    if let Some(compiler) = tsconfig.get("compilerOptions").and_then(|v| v.as_object()) {
        if let Some(target) = compiler.get("target").and_then(|v| v.as_str()) {
            opts.push(format!("--target={}", target));
        }
        if let Some(module) = compiler.get("module").and_then(|v| v.as_str()) {
            opts.push(format!("--module={}", module));
        }
        if let Some(jsx) = compiler.get("jsx").and_then(|v| v.as_str()) {
            opts.push(format!("--jsx={}", jsx));
        }
        if compiler.get("strict").and_then(|v| v.as_bool()).unwrap_or(false) {
            opts.push("--strict".into());
        }
        if let Some(paths) = compiler.get("paths").and_then(|v| v.as_object()) {
            for (alias, _targets) in paths {
                opts.push(format!("--alias={}", alias));
            }
        }
    }
    opts
}

// ── Framework detection ─────────────────────────────────────────────────

pub fn detect_framework_from_pkg(dir: &Path) -> (String, Option<String>) {
    let pkg_path = dir.join("package.json");
    if !pkg_path.exists() {
        return ("Unknown".into(), None);
    }
    let content = match std::fs::read_to_string(&pkg_path) {
        Ok(c) => c,
        Err(_) => return ("Unknown".into(), None),
    };
    let pkg: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return ("Unknown".into(), None),
    };
    let mut deps = HashMap::new();
    if let Some(d) = pkg.get("dependencies").and_then(|v| v.as_object()) {
        for (k, v) in d { deps.insert(k.clone(), v.as_str().unwrap_or("").to_string()); }
    }
    if let Some(d) = pkg.get("devDependencies").and_then(|v| v.as_object()) {
        for (k, v) in d { deps.insert(k.clone(), v.as_str().unwrap_or("").to_string()); }
    }

    let checks: &[(&str, &[&str])] = &[
        ("Next.js", &["next"]),
        ("React", &["react"]),
        ("Vue", &["vue", "nuxt"]),
        ("Svelte", &["svelte", "@sveltejs/kit"]),
        ("Angular", &["@angular/core"]),
        ("Astro", &["astro"]),
        ("NestJS", &["@nestjs/core"]),
        ("Express", &["express"]),
        ("Fastify", &["fastify"]),
        ("Hono", &["hono"]),
        ("Solid", &["solid-js"]),
        ("Gatsby", &["gatsby"]),
        ("Remix", &["@remix-run/react"]),
        ("Preact", &["preact"]),
        ("Lit", &["lit"]),
        ("Node", &[]),
    ];

    for (name, packages) in checks {
        for pkg_name in *packages {
            if let Some(ver) = deps.get(*pkg_name) {
                return (name.to_string(), Some(ver.clone()));
            }
        }
    }
    ("Node".into(), None)
}

// ── klyron.json config auto-create ──────────────────────────────────────

pub fn auto_create_klyron_config(dir: &Path) -> anyhow::Result<()> {
    let config_path = dir.join("klyron.json");
    if config_path.exists() {
        return Ok(());
    }

    let project_type = detect_project_type(dir);
    let (framework, _version) = detect_framework_from_pkg(dir);
    let project_name = dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-app")
        .to_string();

    let mut config = serde_json::json!({
        "name": project_name,
        "version": "0.1.0",
        "type": project_type,
        "framework": framework,
        "compiler": {
            "target": "esnext",
            "module": "esnext",
            "minify": false,
            "sourcemap": false
        },
        "dev": {
            "port": 3000,
            "hmr": true
        },
        "build": {
            "outDir": "dist",
            "minify": true
        }
    });

    if project_type == "node" || project_type == "unknown" {
        config["type"] = serde_json::json!("node");
    }

    if let Some(tsconfig) = detect_tsconfig(dir) {
        if let Some(compiler) = tsconfig.get("compilerOptions") {
            config["compiler"] = compiler.clone();
        }
    }

    eprint!("No klyron.json found. Create one? [Y/n] ");
    std::io::Write::flush(&mut std::io::stderr())?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    if input == "n" || input == "no" {
        println!("Skipping config creation.");
        return Ok(());
    }

    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, content)?;
    println!("Created {}", config_path.display());
    Ok(())
}

pub fn detect_project_language(dir: &Path) -> &'static str {
    if dir.join("tsconfig.json").exists() { return "typescript"; }
    if dir.join("jsconfig.json").exists() { return "javascript"; }
    let pkg = dir.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&pkg) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(dev_deps) = json.get("devDependencies").and_then(|v| v.as_object()) {
                if dev_deps.contains_key("typescript") { return "typescript"; }
            }
            if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
                if deps.contains_key("typescript") { return "typescript"; }
            }
        }
    }
    if dir.join("*.ts").exists() || glob_some(dir, "*.ts") { return "typescript"; }
    "javascript"
}

fn glob_some(_dir: &Path, _pattern: &str) -> bool {
    false
}

// ── Module Resolution with Caching ──────────────────────────────────────

static RESOLVE_CACHE: Lazy<Mutex<HashMap<String, Option<PathBuf>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn clear_resolve_cache() {
    if let Ok(mut cache) = RESOLVE_CACHE.lock() {
        cache.clear();
    }
}

pub fn resolve_module(specifier: &str, base_path: &Path) -> Option<PathBuf> {
    let cache_key = format!("{}:{}", specifier, base_path.display());
    if let Ok(cache) = RESOLVE_CACHE.lock() {
        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }
    }

    let result = resolve_module_inner(specifier, base_path);

    if let Ok(mut cache) = RESOLVE_CACHE.lock() {
        cache.insert(cache_key, result.clone());
    }

    result
}

fn resolve_module_inner(specifier: &str, base_path: &Path) -> Option<PathBuf> {
    let base_dir = if base_path.is_file() {
        base_path.parent().unwrap_or(base_path)
    } else {
        base_path
    };

    let candidate = base_dir.join(specifier);
    if candidate.is_file() {
        return Some(candidate);
    }

    for ext in &[".js", ".ts", ".json", ".jsx", ".tsx", ".mjs", ".cjs"] {
        let with_ext = candidate.with_extension(ext.trim_start_matches('.'));
        if with_ext.is_file() {
            return Some(with_ext);
        }
    }

    for name in &["index.js", "index.ts", "index.json", "index.jsx", "index.tsx", "index.mjs", "index.cjs"] {
        let idx = candidate.join(name);
        if idx.is_file() {
            return Some(idx);
        }
    }

    let mut dir = Some(base_dir);
    while let Some(d) = dir {
        let nm_pkg = d.join("node_modules").join(specifier);
        if let Some(resolved) = resolve_package_entry(&nm_pkg) {
            return Some(resolved);
        }
        dir = d.parent();
    }

    None
}

fn resolve_package_entry(pkg_dir: &Path) -> Option<PathBuf> {
    if !pkg_dir.exists() {
        return None;
    }
    if pkg_dir.is_file() {
        return Some(pkg_dir.to_path_buf());
    }

    let pkg_json = pkg_dir.join("package.json");
    if pkg_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_json) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(exports) = pkg.get("exports") {
                    if let Some(s) = exports.as_str() {
                        let entry = pkg_dir.join(s);
                        if entry.is_file() { return Some(entry); }
                    }
                    if let Some(obj) = exports.as_object() {
                        if let Some(dot) = obj.get(".") {
                            if let Some(s) = dot.as_str() {
                                let entry = pkg_dir.join(s);
                                if entry.is_file() { return Some(entry); }
                            }
                            if let Some(sub_obj) = dot.as_object() {
                                for key in &["import", "require", "default"] {
                                    if let Some(val) = sub_obj.get(*key).and_then(|v| v.as_str()) {
                                        let entry = pkg_dir.join(val);
                                        if entry.is_file() { return Some(entry); }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(imports) = pkg.get("imports").and_then(|v| v.as_object()) {
                    for (_key, val) in imports {
                        if let Some(s) = val.as_str() {
                            let entry = pkg_dir.join(s);
                            if entry.is_file() { return Some(entry); }
                        }
                    }
                }
                if let Some(main) = pkg.get("main").and_then(|m| m.as_str()) {
                    let entry = pkg_dir.join(main);
                    if entry.is_file() { return Some(entry); }
                    for ext in &[".js", ".mjs", ".cjs", ".json", ".ts"] {
                        let with_ext = entry.with_extension(ext.trim_start_matches('.'));
                        if with_ext.is_file() { return Some(with_ext); }
                    }
                }
                if let Some(module) = pkg.get("module").and_then(|m| m.as_str()) {
                    let entry = pkg_dir.join(module);
                    if entry.is_file() { return Some(entry); }
                }
            }
        }
    }

    for name in &["index.js", "index.mjs", "index.cjs", "index.json", "index.ts", "index.jsx", "index.tsx"] {
        let idx = pkg_dir.join(name);
        if idx.is_file() {
            return Some(idx);
        }
    }

    None
}


