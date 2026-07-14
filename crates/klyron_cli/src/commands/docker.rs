use clap::Subcommand;
use klyron_docker::{DockerManager, DockerConfig, DockerProfile};

#[derive(Subcommand)]
pub enum DockerAction {
    Init,
    Build,
    Run,
}

fn detect_project_type(dir: &std::path::Path) -> &'static str {
    if dir.join("package.json").exists() { "node" }
    else if dir.join("Cargo.toml").exists() { "rust" }
    else if dir.join("go.mod").exists() { "go" }
    else if dir.join("requirements.txt").exists() || dir.join("pyproject.toml").exists() { "python" }
    else if dir.join("composer.json").exists() { "php" }
    else { "unknown" }
}

fn generate_dockerignore(dir: &std::path::Path, project: &str) -> anyhow::Result<()> {
    let mut ignore = String::from("node_modules\ntarget\n.git\n*.md\n.env\n");
    match project {
        "node" => ignore.push_str("dist\n.next\n.cache\n"),
        "rust" => ignore.push_str("target\nCargo.lock\n"),
        "go" => ignore.push_str("vendor\n"),
        "python" => ignore.push_str("__pycache__\n*.pyc\n.venv\nvenv\n*.egg-info\n"),
        "php" => ignore.push_str("vendor\ncomposer.lock\n"),
        _ => {}
    }
    std::fs::write(dir.join(".dockerignore"), ignore)?;
    Ok(())
}

pub fn run_docker(action: DockerAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = detect_project_type(&dir);
    match action {
        DockerAction::Init => {
            println!("  Detected project type: {project}");
            DockerManager::generate_dockerfile(&dir)?;
            DockerManager::generate_compose(&dir)?;
            generate_dockerignore(&dir, project)?;
            println!("Docker files generated:");
            println!("  Dockerfile, docker-compose.yml, .dockerignore");
            println!();
            println!("Quick start:");
            println!("  klyron docker build   # Build image");
            println!("  klyron docker run     # Run container");
            Ok(())
        }
        DockerAction::Build => {
            let config = DockerConfig {
                image_name: format!("klyron-{project}"),
                port: 3000,
                project_dir: dir,
                profile: DockerProfile::Dev,
                additional_services: Vec::new(),
                build_args: std::collections::HashMap::new(),
            };
            println!("Building Docker image: {}", config.image_name);
            DockerManager::build(config)
        }
        DockerAction::Run => {
            let config = DockerConfig {
                image_name: format!("klyron-{project}"),
                port: 3000,
                project_dir: dir,
                profile: DockerProfile::Dev,
                additional_services: Vec::new(),
                build_args: std::collections::HashMap::new(),
            };
            println!("Running Docker container from image: {}", config.image_name);
            DockerManager::run(config)
        }
    }
}
