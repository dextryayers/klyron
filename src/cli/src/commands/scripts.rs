use clap::Args;

#[derive(Args)]
pub struct RunScriptArgs {
    pub script: String,
    #[arg(last = true)]
    pub args: Vec<String>,
}

pub fn run_script(args: RunScriptArgs) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let runner = crate::detect_package_runner(&dir);
    crate::run_cmd(runner, &["run", &args.script], &dir)
}

pub fn run_start() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let runner = crate::detect_package_runner(&dir);
    crate::run_cmd(runner, &["start"], &dir)
}

pub fn run_test_script() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let runner = crate::detect_package_runner(&dir);
    crate::run_cmd(runner, &["test"], &dir)
}

pub fn run_lint_script() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let runner = crate::detect_package_runner(&dir);
    crate::run_cmd(runner, &["run", "lint"], &dir)
}

pub fn run_format_script() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let runner = crate::detect_package_runner(&dir);
    crate::run_cmd(runner, &["run", "format"], &dir)
}
