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

fn detect_package_manager(dir: &std::path::Path) -> &str {
    if dir.join("pnpm-lock.yaml").exists() { "pnpm" }
    else if dir.join("yarn.lock").exists() { "yarn" }
    else if dir.join("bun.lock").exists() || dir.join("bun.lockb").exists() { "bun" }
    else { "npm" }
}

pub fn run_add(packages: &[String], dev: bool) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    let pm = detect_package_manager(&dir);

    match project {
        "node" => {
            let mut args = vec!["install".to_string()];
            if dev { args.push("--save-dev".to_string()); }
            args.extend(packages.iter().cloned());
            crate::run_cmd(pm, &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), &dir)
        }
        "laravel" => {
            let mut args = vec!["require"];
            for p in packages { args.push(p); }
            crate::run_cmd("composer", &args, &dir)
        }
        "python" => {
            let mut args = vec!["install"];
            for p in packages { args.push(p); }
            crate::run_cmd("pip", &args, &dir)
        }
        "ruby" => {
            let mut args = vec!["add"];
            for p in packages { args.push(p); }
            crate::run_cmd("bundle", &args, &dir)
        }
        "rust" => {
            let mut args = vec!["add"];
            for p in packages { args.push(p); }
            crate::run_cmd("cargo", &args, &dir)
        }
        "go" => {
            let mut args = vec!["get"];
            for p in packages { args.push(p); }
            crate::run_cmd("go", &args, &dir)
        }
        _ => anyhow::bail!("Package add not supported for {project}"),
    }
}

pub fn run_install() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    let pm = detect_package_manager(&dir);

    match project {
        "node" => crate::run_cmd(pm, &["install"], &dir),
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
    let pm = detect_package_manager(&dir);

    match project {
        "node" => {
            let subcmd = match pm { "bun" => "remove", "pnpm" => "remove", "yarn" => "remove", _ => "uninstall" };
            let mut args = vec![pm, subcmd];
            for p in packages { args.push(p); }
            crate::run_cmd_str(pm, &args[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>(), &dir)
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
    let pm = detect_package_manager(&dir);
    match project {
        "node" => crate::run_cmd(pm, &["update"], &dir),
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
    let pm = detect_package_manager(&dir);
    match project {
        "node" => crate::run_cmd(pm, &["outdated"], &dir),
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
    let pm = detect_package_manager(&dir);
    match project {
        "node" => crate::run_cmd(pm, &["audit"], &dir),
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
    let pm = detect_package_manager(&dir);
    match project {
        "node" => crate::run_cmd(pm, &["dedupe"], &dir),
        _ => anyhow::bail!("Dedupe not supported for {project}"),
    }
}
