use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct CodeIgniterAdapter;

#[async_trait]
impl FrameworkAdapter for CodeIgniterAdapter {
    fn name(&self) -> &'static str { "codeigniter" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("spark").exists() || dir.join("app/Config").exists() || dir.join("system").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.4", "4.5"] }
    fn default_version(&self) -> &'static str { "4.5" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("php");
        cmd.args(["spark", "serve"]).current_dir(dir);
        if let Some(p) = port { cmd.arg(format!("--port={}", p)); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("php")
            .args(["./vendor/bin/phpunit"])
            .current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("php")
            .args(["./vendor/bin/php-cs-fixer", "fix", "--dry-run", "--diff"])
            .current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/php-cs-fixer", "fix"])
                .current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/php-cs-fixer", "fix", "--dry-run", "--diff"])
                .current_dir(dir).status().await?;
        }
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("composer".into(), vec![
            "create-project".into(),
            "codeigniter4/appstarter".into(),
            name.into(),
        ]))
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
