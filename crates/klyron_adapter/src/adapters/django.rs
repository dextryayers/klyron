use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct DjangoAdapter;

#[async_trait]
impl FrameworkAdapter for DjangoAdapter {
    fn name(&self) -> &'static str { "django" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("manage.py").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.2", "5.0"] }
    fn default_version(&self) -> &'static str { "5.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Polyglot }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("python3");
        cmd.args(["manage.py", "runserver"]).current_dir(dir);
        if let Some(p) = port { cmd.arg(format!("0.0.0.0:{}", p)); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("python3").args(["-m", "pytest", "."]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("ruff").args(["check", "."]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("black").args(["."]).current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("black").args(["--check", "."]).current_dir(dir).status().await?;
        }
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("python3".into(), vec!["-m".into(), "django".into(), "startproject".into(), name.into()]))
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
