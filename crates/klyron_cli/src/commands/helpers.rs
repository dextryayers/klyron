use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

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


