use clap::Args;

#[derive(Args)]
pub struct AddArgs {
    pub packages: Vec<String>,
    #[arg(long)]
    pub dev: bool,
}

#[derive(Args)]
pub struct RemoveArgs {
    pub packages: Vec<String>,
}

pub fn run_add(packages: &[String], dev: bool) -> anyhow::Result<()> {
    let pkg_list = packages.join(" ");
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    let runner = crate::detect_package_runner(&dir);

    match project {
        "node" => {
            let mut args = vec![runner, "install"];
            if dev { args.push("--save-dev"); }
            for p in packages { args.push(p); }
            crate::run_cmd_str(runner, &args[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>(), &dir)
        }
        "laravel" => crate::run_cmd("composer", &["require", &pkg_list], &dir),
        "python" => crate::run_cmd("pip", &["install", &pkg_list], &dir),
        "ruby" => {
            let mut args = vec!["add".to_string()];
            for p in packages { args.push(p.clone()); }
            crate::run_cmd_str("bundle", &args, &dir)
        }
        "rust" => crate::run_cmd("cargo", &["add", &pkg_list], &dir),
        "go" => crate::run_cmd("go", &["get", &pkg_list], &dir),
        _ => anyhow::bail!("Package add not supported for {project}"),
    }
}

pub fn run_install() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    let runner = crate::detect_package_runner(&dir);

    match project {
        "node" => crate::run_cmd(runner, &["install"], &dir),
        "laravel" => crate::run_cmd("composer", &["install"], &dir),
        "python" => {
            if dir.join("Pipfile").exists() { crate::run_cmd("pipenv", &["install"], &dir) }
            else if dir.join("poetry.lock").exists() { crate::run_cmd("poetry", &["install"], &dir) }
            else if dir.join("requirements.txt").exists() { crate::run_cmd("pip", &["install", "-r", "requirements.txt"], &dir) }
            else { anyhow::bail!("No requirements file found") }
        }
        "ruby" => crate::run_cmd("bundle", &["install"], &dir),
        "rust" => Ok(()),
        "go" => crate::run_cmd("go", &["mod", "download"], &dir),
        _ => anyhow::bail!("Install not supported for {project}"),
    }
}

pub fn run_remove(packages: &[String]) -> anyhow::Result<()> {
    let pkg_list = packages.join(" ");
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    let runner = crate::detect_package_runner(&dir);

    match project {
        "node" => {
            let subcmd = match runner { "bun" => "remove", "pnpm" => "remove", "yarn" => "remove", _ => "uninstall" };
            let mut args = vec![runner, subcmd];
            for p in packages { args.push(p); }
            crate::run_cmd_str(runner, &args[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>(), &dir)
        }
        "laravel" => crate::run_cmd("composer", &["remove", &pkg_list], &dir),
        "python" => crate::run_cmd("pip", &["uninstall", "-y", &pkg_list], &dir),
        "ruby" => crate::run_cmd("bundle", &["remove", &pkg_list], &dir),
        "rust" => crate::run_cmd("cargo", &["remove", &pkg_list], &dir),
        "go" => crate::run_cmd("go", &["mod", "tidy"], &dir),
        _ => anyhow::bail!("Remove not supported for {project}"),
    }
}

pub fn run_update() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    match project {
        "node" => crate::run_cmd("npm", &["update"], &dir),
        "laravel" => crate::run_cmd("composer", &["update"], &dir),
        "rust" => crate::run_cmd("cargo", &["update"], &dir),
        "python" => crate::run_cmd("pip", &["install", "--upgrade", "-r", "requirements.txt"], &dir),
        "go" => crate::run_cmd("go", &["mod", "tidy"], &dir),
        _ => anyhow::bail!("Update not supported for {project}"),
    }
}

pub fn run_outdated() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    match project {
        "node" => crate::run_cmd("npm", &["outdated"], &dir),
        "laravel" => crate::run_cmd("composer", &["outdated"], &dir),
        "rust" => crate::run_cmd("cargo", &["outdated"], &dir),
        "python" => crate::run_cmd("pip", &["list", "--outdated"], &dir),
        "go" => crate::run_cmd("go", &["list", "-u", "-m", "all"], &dir),
        "ruby" => crate::run_cmd("bundle", &["outdated"], &dir),
        _ => anyhow::bail!("Outdated not supported for {project}"),
    }
}

pub fn run_audit() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    match project {
        "node" => crate::run_cmd("npm", &["audit"], &dir),
        "rust" => crate::run_cmd("cargo", &["audit"], &dir),
        _ => {
            println!("Audit not configured for {project}, running npm audit...");
            crate::run_cmd("npm", &["audit"], &dir)
        }
    }
}

pub fn run_dedupe() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    match project {
        "node" => crate::run_cmd("npm", &["dedupe"], &dir),
        _ => anyhow::bail!("Dedupe not supported for {project}"),
    }
}
