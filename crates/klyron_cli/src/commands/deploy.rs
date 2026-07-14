use clap::Args;

#[derive(Args)]
pub struct DeployArgs {
    pub platform: String,
    #[arg(long)]
    pub preview: bool,
}

pub fn run_deploy(args: DeployArgs) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match args.platform.as_str() {
        "vercel" => deploy_vercel(&dir, args.preview),
        "cloudflare" => deploy_cloudflare(&dir),
        "railway" => deploy_railway(&dir),
        "fly" => deploy_fly(&dir),
        "docker" => crate::run_cmd("docker", &["compose", "up", "-d"], &dir),
        p => anyhow::bail!("Unknown platform: {p}. Use: vercel, cloudflare, railway, fly, docker"),
    }
}

fn deploy_vercel(dir: &std::path::Path, preview: bool) -> anyhow::Result<()> {
    let vercel_json = r#"{
  "version": 2,
  "builds": [{ "src": "package.json", "use": "@vercel/static-build" }],
  "routes": [{ "src": "/(.*)", "dest": "/$1" }]
}"#;
    std::fs::write(dir.join("vercel.json"), vercel_json)?;
    if preview {
        crate::run_cmd("npx", &["vercel"], dir)
    } else {
        crate::run_cmd("npx", &["vercel", "--prod"], dir)
    }
}

fn deploy_cloudflare(dir: &std::path::Path) -> anyhow::Result<()> {
    let wrangler = r#"name = "klyron-app"
main = "dist/index.js"
compatibility_date = "2024-12-01"
"#;
    std::fs::write(dir.join("wrangler.toml"), wrangler)?;
    crate::run_cmd("npx", &["wrangler", "deploy"], dir)
}

fn deploy_railway(dir: &std::path::Path) -> anyhow::Result<()> {
    let railway = r#"{
  "build": { "builder": "NIXPACKS" },
  "deploy": { "numReplicas": 1, "sleepApplication": false }
}"#;
    std::fs::write(dir.join("railway.json"), railway)?;
    crate::run_cmd("npx", &["railway", "up"], dir)
}

fn deploy_fly(dir: &std::path::Path) -> anyhow::Result<()> {
    let fly = r#"app = "klyron-app"
primary_region = "iad"

[build]
  builder = "heroku/buildpacks:20"

[http_service]
  internal_port = 3000
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 0
"#;
    std::fs::write(dir.join("fly.toml"), fly)?;
    crate::run_cmd("flyctl", &["deploy"], dir)
}
