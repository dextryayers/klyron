use clap::Subcommand;
use klyron_workspace::Workspace;

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
    Workspace::init(&dir, "my-workspace")?;
    println!("Workspace initialized: klyron.toml");
    Ok(())
}

fn workspace_list() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    println!("Workspace members in {}", dir.display());
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && (path.join("package.json").exists() || path.join("Cargo.toml").exists())
        {
            println!(
                "  {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
        }
    }
    Ok(())
}

fn workspace_add(name: &str) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    Workspace::add_member(&dir, name)?;
    println!("Added workspace member: {}", name);
    Ok(())
}

fn workspace_remove(name: &str) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    Workspace::remove_member(&dir, name)?;
    println!("Removed workspace member: {}", name);
    Ok(())
}

fn workspace_run(script: &str) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    Workspace::run_script(&dir, script)
}
