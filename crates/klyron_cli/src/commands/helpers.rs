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
    if dir.join("bun.lockb").exists() { "bun" }
    else if dir.join("pnpm-lock.yaml").exists() { "pnpm" }
    else if dir.join("yarn.lock").exists() { "yarn" }
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
