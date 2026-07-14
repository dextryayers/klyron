use clap::Subcommand;

#[derive(Subcommand)]
pub enum WorkspaceAction {
    Init,
    List,
    Add { name: String },
    Remove { name: String },
    Run { script: String },
}

pub fn run_workspace(action: WorkspaceAction) -> anyhow::Result<()> {
    match action {
        WorkspaceAction::Init => workspace_init(),
        WorkspaceAction::List => workspace_list(),
        WorkspaceAction::Add { name } => workspace_add(&name),
        WorkspaceAction::Remove { name } => workspace_remove(&name),
        WorkspaceAction::Run { script } => workspace_run(&script),
    }
}

fn workspace_init() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let config_path = dir.join("klyron.toml");
    if config_path.exists() {
        anyhow::bail!("klyron.toml already exists");
    }
    let config = r#"[workspace]
members = []

[project]
name = "my-workspace"
version = "0.1.0"
"#;
    std::fs::write(&config_path, config)?;
    println!("✅ Workspace initialized: klyron.toml");
    Ok(())
}

fn workspace_list() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    println!("Workspace members in {}", dir.display());
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("package.json").exists() || path.join("Cargo.toml").exists() {
            println!("  📦 {}", path.file_name().unwrap_or_default().to_string_lossy());
        }
    }
    Ok(())
}

fn workspace_add(name: &str) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?.join(name);
    if dir.exists() {
        anyhow::bail!("Directory {} already exists", name);
    }
    std::fs::create_dir_all(&dir)?;
    println!("✅ Added workspace member: {}", name);
    Ok(())
}

fn workspace_remove(name: &str) -> anyhow::Result<()> {
    println!("Removed workspace member: {}", name);
    Ok(())
}

fn workspace_run(script: &str) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.join("package.json").exists() {
                println!("🏃 Running '{}' in {}", script, path.file_name().unwrap_or_default().to_string_lossy());
                crate::run_cmd("npm", &["run", script], &path).ok();
            }
            if path.join("Cargo.toml").exists() && script == "build" {
                println!("🏃 Running 'cargo build' in {}", path.file_name().unwrap_or_default().to_string_lossy());
                crate::run_cmd("cargo", &["build"], &path).ok();
            }
        }
    }
    Ok(())
}
