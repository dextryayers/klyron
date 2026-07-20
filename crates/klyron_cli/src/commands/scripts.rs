use std::path::Path;

use clap::Args;

#[derive(Args)]
pub struct RunScriptArgs {
    pub script: String,
    #[arg(last = true)]
    pub args: Vec<String>,
}

fn run_via_pm(dir: &Path, pm_args: &[String]) -> anyhow::Result<()> {
    let runner = crate::detect_package_runner(dir);
    let status = std::process::Command::new(runner)
        .args(pm_args)
        .current_dir(dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run {}: {e}", runner))?;
    if !status.success() {
        anyhow::bail!("{} exited with code {}", runner, status.code().unwrap_or(-1));
    }
    Ok(())
}

fn detect_project_type_for_start(dir: &Path) -> &'static str {
    if dir.join("Cargo.toml").exists() { return "rust"; }
    if dir.join("composer.json").exists() || dir.join("artisan").exists() { return "laravel"; }
    if dir.join("go.mod").exists() { return "go"; }
    if dir.join("pyproject.toml").exists() || dir.join("manage.py").exists() { return "python"; }
    if dir.join("Gemfile").exists() { return "ruby"; }
    "node"
}

pub fn run_script(args: RunScriptArgs) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let mut pm_args = vec!["run".to_string(), args.script.clone()];
    pm_args.extend(args.args);
    run_via_pm(&dir, &pm_args)
}

pub fn run_start() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match detect_project_type_for_start(&dir) {
        "rust" => crate::run_cmd("cargo", &["run", "--release"], &dir),
        "laravel" => crate::run_cmd("php", &["artisan", "serve"], &dir),
        "go" => crate::run_cmd("go", &["run", "."], &dir),
        "python" => {
            let py = if cfg!(windows) { "python" } else { "python3" };
            crate::run_cmd(py, &["manage.py", "runserver"], &dir)
        }
        _ => run_via_pm(&dir, &["start".to_string()]),
    }
}

pub fn run_test_script() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    run_via_pm(&dir, &["test".to_string()])
}

pub fn run_lint_script() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    run_via_pm(&dir, &["run".to_string(), "lint".to_string()])
}

pub fn run_format_script() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    run_via_pm(&dir, &["run".to_string(), "format".to_string()])
}
