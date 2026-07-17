use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct SymfonyAdapter;

#[async_trait]
impl FrameworkAdapter for SymfonyAdapter {
    fn name(&self) -> &'static str { "symfony" }

    fn detect(&self, dir: &Path) -> bool {
        let composer = dir.join("composer.json");
        if let Ok(content) = std::fs::read_to_string(composer) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(require) = json.get("require") {
                    for key in &["symfony/framework-bundle", "symfony/symfony", "symfony/http-kernel"] {
                        if require.get(*key).is_some() { return true; }
                    }
                }
            }
        }
        dir.join("bin/console").exists() && dir.join("config").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["5.4", "6.4", "7.0", "7.1", "7.2"] }
    fn default_version(&self) -> &'static str { "7.1" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let port = port.unwrap_or(8000);
        let mut cmd = tokio::process::Command::new("php");
        cmd.args(["-S", &format!("0.0.0.0:{}", port), "-t", "public"])
            .current_dir(dir);
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("php")
            .args(["./bin/phpunit"])
            .current_dir(dir)
            .status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("php")
            .args(["./vendor/bin/php-cs-fixer", "fix", "--dry-run"])
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
                .args(["./vendor/bin/php-cs-fixer", "fix", "--dry-run"])
                .current_dir(dir).status().await?;
        }
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, version: Option<&str>) -> Option<(String, Vec<String>)> {
        let mut args = vec!["create-project".into(), "symfony/skeleton".into(), name.into()];
        if let Some(v) = version {
            args.push(v.into());
        }
        Some(("composer".into(), args))
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
