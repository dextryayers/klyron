use clap::Subcommand;
use klyron_workspace::Workspace;

#[derive(Subcommand)]
pub enum WorkspaceAction {
    Init,
    List,
    Add { name: String },
    Remove { name: String },
    Run { script: String },
    Version { part: Option<String> },
    Graph,
}

pub fn run_workspace(action: WorkspaceAction) -> anyhow::Result<()> {
    match action {
        WorkspaceAction::Init => workspace_init(),
        WorkspaceAction::List => workspace_list(),
        WorkspaceAction::Add { name } => workspace_add(&name),
        WorkspaceAction::Remove { name } => workspace_remove(&name),
        WorkspaceAction::Run { script } => workspace_run(&script),
        WorkspaceAction::Version { part } => workspace_version(part.as_deref()),
        WorkspaceAction::Graph => workspace_graph(),
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
    let mut found = false;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && (path.join("package.json").exists() || path.join("Cargo.toml").exists())
        {
            let version = detect_version(&path);
            let ver_str = match version {
                Some(v) => format!("v{v}"),
                None => "no version".into(),
            };
            println!("  {} ({})", path.file_name().unwrap_or_default().to_string_lossy(), ver_str);
            found = true;
        }
    }
    if !found {
        println!("  (no workspace members found)");
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

fn workspace_version(part: Option<&str>) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let part = part.unwrap_or("patch");
    if !["major", "minor", "patch"].contains(&part) {
        anyhow::bail!("Invalid version part '{part}'. Use: major, minor, patch");
    }

    let members = Workspace::list_members(&dir)?;
    if members.is_empty() {
        println!("No workspace members found");
        return Ok(());
    }

    for member in &members {
        let member_path = dir.join(&member.name);
        let bumped = bump_version(&member_path, part);
        match bumped {
            Some(v) => println!("  {}: bumped to {v}", member.name),
            None => println!("  {}: no version file found", member.name),
        }
    }
    Ok(())
}

fn workspace_graph() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match Workspace::render_dependency_graph(&dir) {
        Ok(dot) => {
            println!("{}", dot);
            Ok(())
        }
        Err(_) => {
            println!("Dependency graph:");
            let members = Workspace::list_members(&dir)?;
            for member in &members {
                println!("  {}", member.name);
            }
            Ok(())
        }
    }
}

fn detect_version(dir: &std::path::Path) -> Option<String> {
    let pkg_json = dir.join("package.json");
    if pkg_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("version").and_then(|v| v.as_str()) {
                    return Some(v.to_string());
                }
            }
        }
    }
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            if let Ok(toml_val) = toml::from_str::<toml::Value>(&content) {
                if let Some(pkg) = toml_val.get("package") {
                    if let Some(v) = pkg.get("version").and_then(|v| v.as_str()) {
                        return Some(v.to_string());
                    }
                }
            }
        }
    }
    None
}

fn bump_version(dir: &std::path::Path, part: &str) -> Option<String> {
    let pkg_json = dir.join("package.json");
    if pkg_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_json) {
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json.as_object_mut() {
                    let current = obj.get("version").and_then(|v| v.as_str())
                        .unwrap_or("0.0.0")
                        .to_string();
                    let new_ver = semver_bump(&current, part);
                    obj.insert("version".into(), serde_json::Value::String(new_ver.clone()));
                    if std::fs::write(&pkg_json, serde_json::to_string_pretty(&json).unwrap()).is_ok() {
                        return Some(new_ver);
                    }
                }
            }
        }
    }
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            if let Ok(mut toml_val) = toml::from_str::<toml::Value>(&content) {
                if let Some(pkg) = toml_val.get_mut("package") {
                    if let Some(toml::Value::String(v)) = pkg.get_mut("version") {
                        let new_ver = semver_bump(v, part);
                        *v = new_ver.clone();
                        if std::fs::write(&cargo_toml, toml_val.to_string()).is_ok() {
                            return Some(new_ver);
                        }
                    }
                }
            }
        }
    }
    None
}

fn semver_bump(version: &str, part: &str) -> String {
    let clean = version.trim_start_matches('^').trim_start_matches('~');
    let parts: Vec<&str> = clean.split('.').collect();
    let major: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    match part {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{}.{}.0", major, minor + 1),
        _ => format!("{}.{}.{}", major, minor, patch + 1),
    }
}
