use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct CakePHPAdapter;

#[async_trait]
impl FrameworkAdapter for CakePHPAdapter {
    fn name(&self) -> &'static str { "cakephp" }

    fn detect(&self, dir: &Path) -> bool {
        let composer = dir.join("composer.json");
        if let Ok(content) = std::fs::read_to_string(composer) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(require) = json.get("require") {
                    for key in &["cakephp/cakephp"] {
                        if require.get(*key).is_some() { return true; }
                    }
                }
            }
        }
        dir.join("config/app.php").exists() && dir.join("bin/cake").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.4", "4.5", "5.0"] }
    fn default_version(&self) -> &'static str { "4.5" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("php");
        cmd.args(["bin/cake.php", "server"]).current_dir(dir);
        if let Some(p) = port { cmd.args(["-p", &p.to_string()]); }
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
            .args(["./vendor/bin/phpcs", "--standard=CakePHP", "--extensions=php", "src/", "tests/"])
            .current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        if write {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/phpcbf", "--standard=CakePHP", "--extensions=php", "src/", "tests/"])
                .current_dir(dir).status().await?;
        } else {
            tokio::process::Command::new("php")
                .args(["./vendor/bin/phpcs", "--standard=CakePHP", "--extensions=php", "src/", "tests/"])
                .current_dir(dir).status().await?;
        }
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("composer".into(), vec![
            "create-project".into(),
            "cakephp/app".into(),
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
