use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct WintersmithAdapter;

#[async_trait]
impl FrameworkAdapter for WintersmithAdapter {
    fn name(&self) -> &'static str { "wintersmith" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"wintersmith\""))
            .unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["2.5"] }
    fn default_version(&self) -> &'static str { "2.5" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::StaticSiteGenerator }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["wintersmith", "preview"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["wintersmith", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, _dir: &Path, _filter: Option<&str>) -> Result<()> { Ok(()) }

    async fn lint(&self, _dir: &Path, _fix: bool) -> Result<()> { Ok(()) }

    async fn format(&self, _dir: &Path, _write: bool) -> Result<()> { Ok(()) }

    fn external_scaffold_command(&self, _name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        None
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        if let Some((cmd, args)) = self.external_scaffold_command(name, options.version.as_deref()) {
            let status = std::process::Command::new(&cmd).args(&args).current_dir(&options.dir).status()?;
            if !status.success() { anyhow::bail!("External scaffolding failed"); }
            Ok(())
        } else {
            std::fs::create_dir_all(options.dir.join(name))?;
            Ok(())
        }
    }
}
