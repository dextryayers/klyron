use clap::Subcommand;
use klyron_docker::{DockerManager, DockerConfig};

#[derive(Subcommand)]
pub enum DockerAction {
    Init,
    Build,
    Run,
}

pub fn run_docker(action: DockerAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        DockerAction::Init => {
            DockerManager::generate_dockerfile(&dir)?;
            DockerManager::generate_compose(&dir)?;
            std::fs::write(dir.join(".dockerignore"), "node_modules\ntarget\n.git\n*.md\n")?;
            println!("Docker files generated:");
            println!("  Dockerfile, docker-compose.yml, .dockerignore");
            Ok(())
        }
        DockerAction::Build => {
            let config = DockerConfig {
                image_name: "klyron-app".into(),
                port: 3000,
                project_dir: dir,
                profile: klyron_docker::DockerProfile::Dev,
                additional_services: Vec::new(),
                build_args: std::collections::HashMap::new(),
            };
            DockerManager::build(config)
        }
        DockerAction::Run => {
            let config = DockerConfig {
                image_name: "klyron-app".into(),
                port: 3000,
                project_dir: dir,
                profile: klyron_docker::DockerProfile::Dev,
                additional_services: Vec::new(),
                build_args: std::collections::HashMap::new(),
            };
            DockerManager::run(config)
        }
    }
}
