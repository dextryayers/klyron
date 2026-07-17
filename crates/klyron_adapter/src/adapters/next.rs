use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct NextAdapter;

#[async_trait]
impl FrameworkAdapter for NextAdapter {
    fn name(&self) -> &'static str { "next" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("next.config.ts").exists() || dir.join("next.config.js").exists() || dir.join("next.config.mjs").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["14.0", "15.0", "15.1", "15.2", "15.3"] }
    fn default_version(&self) -> &'static str { "15.3" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Fullstack }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["next", "dev"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); } else { cmd.env("PORT", "3000"); }
        cmd.env("NEXT_TELEMETRY_DISABLED", "1");
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["next", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("npx").args(["vitest", "run"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("npx").args(["next", "lint"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        tokio::process::Command::new("npx")
            .args(if write { ["prettier", "--write", "."] } else { ["prettier", "--check", "."] })
            .current_dir(dir).status().await?;
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("npx".into(), vec!["create-next-app@latest".into(), name.into()]))
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
