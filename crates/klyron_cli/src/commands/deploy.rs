use clap::Args;
use klyron_deploy::{Deployment, DeployConfig, DeployPlatform};

#[derive(Args)]
pub struct DeployArgs {
    pub platform: String,
    #[arg(long)]
    pub preview: bool,
}

pub fn run_deploy(args: DeployArgs) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let platform = match args.platform.as_str() {
        "vercel" => DeployPlatform::Vercel,
        "cloudflare" => DeployPlatform::Cloudflare,
        "railway" => DeployPlatform::Railway,
        "fly" => DeployPlatform::Fly,
        "docker" => DeployPlatform::Docker,
        p => anyhow::bail!(
            "Unknown platform: {p}. Use: vercel, cloudflare, railway, fly, docker"
        ),
    };
    Deployment::deploy(DeployConfig {
        platform,
        preview: args.preview,
        project_dir: dir,
    })
}
