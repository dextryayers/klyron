use clap::Args;

#[derive(Args)]
pub struct ArtisanArgs {
    #[arg(last = true)]
    pub args: Vec<String>,
    #[arg(long)]
    pub project: Option<String>,
}

#[derive(Args)]
pub struct ComposerArgs {
    #[arg(last = true)]
    pub args: Vec<String>,
    #[arg(long)]
    pub project: Option<String>,
}

#[derive(Args)]
pub struct BladeArgs {
    pub view: String,
    #[arg(long)]
    pub data: Option<String>,
    #[arg(long)]
    pub project: Option<String>,
}

#[derive(Args)]
pub struct TinkerArgs {
    #[arg(long)]
    pub project: Option<String>,
}

pub fn run_artisan(args: &[String], project: Option<&str>) -> anyhow::Result<()> {
    let dir = if let Some(p) = project { std::path::PathBuf::from(p) } else { std::env::current_dir()? };
    let mut cmd = vec!["artisan"];
    cmd.extend(args.iter().map(|s| s.as_str()));
    crate::run_cmd("php", &cmd, &dir)
}

pub fn run_composer(args: &[String], project: Option<&str>) -> anyhow::Result<()> {
    let dir = if let Some(p) = project { std::path::PathBuf::from(p) } else { std::env::current_dir()? };
    let cmd: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    crate::run_cmd("composer", &cmd, &dir)
}

pub fn run_blade(view: &str, data: Option<&str>, project: Option<&str>) -> anyhow::Result<()> {
    let dir = if let Some(p) = project { std::path::PathBuf::from(p) } else { std::env::current_dir()? };
    if let Some(d) = data {
        crate::run_cmd("php", &["artisan", "blade:render", view, "--data", d], &dir)
    } else {
        crate::run_cmd("php", &["artisan", "blade:render", view], &dir)
    }
}

pub fn run_tinker(project: Option<&str>) -> anyhow::Result<()> {
    let dir = if let Some(p) = project { std::path::PathBuf::from(p) } else { std::env::current_dir()? };
    crate::run_cmd("php", &["artisan", "tinker"], &dir)
}
