use clap::Args;

#[derive(Args)]
pub struct PublishArgs {
    #[arg(long)]
    pub registry: Option<String>,
}

#[derive(Args)]
pub struct UnpublishArgs {
    pub name: String,
    #[arg(long)]
    pub registry: Option<String>,
}

#[derive(Args)]
pub struct LoginArgs {
    pub registry: Option<String>,
}

#[derive(Args)]
pub struct LogoutArgs {
    pub registry: Option<String>,
}

#[derive(Args)]
pub struct SearchArgs {
    pub query: String,
    #[arg(long)]
    pub registry: Option<String>,
}

#[derive(Args)]
pub struct InfoArgs {
    pub package: String,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub registry: Option<String>,
}

pub fn run_publish() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    if dir.join("package.json").exists() {
        crate::run_cmd("npm", &["publish"], &dir)
    } else if dir.join("Cargo.toml").exists() {
        crate::run_cmd("cargo", &["publish"], &dir)
    } else {
        anyhow::bail!("No package manifest found (package.json or Cargo.toml)")
    }
}

pub fn run_unpublish(name: &str) -> anyhow::Result<()> {
    crate::run_cmd("npm", &["unpublish", name], &std::env::current_dir()?)
}

pub fn run_login(registry: Option<&str>) -> anyhow::Result<()> {
    let reg = registry.unwrap_or("npm");
    match reg {
        "npm" => crate::run_cmd("npm", &["login"], &std::env::current_dir()?),
        "pypi" => crate::run_cmd("python3", &["-m", "twine", "upload"], &std::env::current_dir()?),
        _ => crate::run_cmd("npm", &["login"], &std::env::current_dir()?),
    }
}

pub fn run_logout(registry: Option<&str>) -> anyhow::Result<()> {
    let reg = registry.unwrap_or("npm");
    match reg {
        "npm" => crate::run_cmd("npm", &["logout"], &std::env::current_dir()?),
        _ => crate::run_cmd("npm", &["logout"], &std::env::current_dir()?),
    }
}

pub fn run_whoami() -> anyhow::Result<()> {
    crate::run_cmd("npm", &["whoami"], &std::env::current_dir()?)
}

pub fn run_search(query: &str) -> anyhow::Result<()> {
    crate::run_cmd("npm", &["search", query], &std::env::current_dir()?)
}

pub fn run_info(package: &str) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let status = std::process::Command::new("npm")
        .args(["info", package])
        .current_dir(&dir)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run npm: {e}"))?;
    if !status.success() {
        anyhow::bail!("npm info failed");
    }
    Ok(())
}
