use clap::Args;
use klyron_deploy::{Deployment, DeployConfig, DeployPlatform};

#[derive(Args)]
pub struct DeployArgs {
    pub platform: String,
    #[arg(long)]
    pub preview: bool,
    #[arg(long)]
    pub serverless: bool,
    #[arg(long, default_value = "/health")]
    pub health_check: String,
    #[arg(long)]
    pub env: Vec<String>,
    #[arg(long)]
    pub secret: Vec<String>,
}

pub fn run_deploy(args: DeployArgs) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let platform = match args.platform.as_str() {
        "vercel" => DeployPlatform::Vercel,
        "netlify" => DeployPlatform::Netlify,
        "cloudflare" => DeployPlatform::Cloudflare,
        "railway" => DeployPlatform::Railway,
        "fly" => DeployPlatform::Fly,
        "docker" => DeployPlatform::Docker,
        p => anyhow::bail!(
            "Unknown platform: {p}. Use: vercel, netlify, cloudflare, railway, fly, docker"
        ),
    };

    let mut env_vars = std::collections::HashMap::new();
    for e in &args.env {
        if let Some((key, val)) = e.split_once('=') {
            env_vars.insert(key.to_string(), val.to_string());
        }
    }

    println!("{} Deploying to {}...", crate::color::Color::CYAN.paint("→"), args.platform);

    if args.serverless {
        println!("{} Generating serverless functions...", crate::color::Color::CYAN.paint("→"));
        Deployment::generate_serverless_functions(&dir, &DeployConfig {
            platform,
            preview: args.preview,
            project_dir: dir.clone(),
            env_vars: env_vars.clone(),
            secrets: args.secret.clone(),
            serverless: true,
            health_check_path: args.health_check.clone(),
        })?;
    }

    Deployment::deploy(DeployConfig {
        platform,
        preview: args.preview,
        project_dir: dir,
        env_vars,
        secrets: args.secret,
        serverless: args.serverless,
        health_check_path: args.health_check,
    })?;

    println!("{} Deploy complete!", crate::color::Color::GREEN.paint("✓"));
    Ok(())
}
