use clap::Args;
use klyron_pm::KlyronLockfile;
use std::path::Path;

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

#[derive(Args)]
pub struct LockArgs {
    #[command(subcommand)]
    pub action: LockAction,
}

#[derive(clap::Subcommand)]
pub enum LockAction {
    Verify,
    Update {
        #[arg(long)]
        force: bool,
    },
}

#[derive(Args)]
pub struct LinkArgs {
    pub package_dir: Option<String>,
    #[arg(long)]
    pub global: bool,
    #[arg(long)]
    pub global_dir: Option<String>,
    pub package_name: Option<String>,
    pub target_dir: Option<String>,
}

#[derive(clap::Subcommand)]
pub enum DistTagAction {
    Add {
        package: String,
        version: String,
        tag: String,
        #[arg(long, default_value = "https://registry.npmjs.org")]
        registry: String,
    },
    Remove {
        package: String,
        tag: String,
        #[arg(long, default_value = "https://registry.npmjs.org")]
        registry: String,
    },
    List {
        package: String,
        #[arg(long, default_value = "https://registry.npmjs.org")]
        registry: String,
    },
}

fn detect_package_manager(dir: &Path) -> &str {
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

pub fn run_install(frozen: bool) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);

    match project {
        "node" => {
            if dir.join("klyron.lock").exists() {
                let pm_dir = dir.clone();
                klyron_pm::install_with_lockfile(&pm_dir, frozen)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                return Ok(());
            }
            let pm = detect_package_manager(&dir);
            let mut args = vec!["install"];
            if frozen {
                match pm {
                    "npm" => args.push("--frozen-lockfile"),
                    "yarn" => args.push("--frozen-lockfile"),
                    "pnpm" => args.push("--frozen-lockfile"),
                    _ => args.push("--no-save"),
                };
            }
            crate::run_cmd(pm, &args, &dir)?;
            let _ = generate_klyron_lock_after_install(&dir);

            // Auto-link workspace members
            let ws = klyron_pm::WorkspaceManager::new(&dir);
            if ws.config.is_some() {
                let global_dir = klyron_pm::get_global_link_dir();
                for member in &ws.member_packages {
                    if let Err(e) = klyron_pm::link_package(member, &global_dir) {
                        eprintln!("Warning: Failed to link workspace member {}: {e}", member.display());
                    }
                }
                let member_names = ws.get_member_names();
                for name in &member_names {
                    if let Err(e) = klyron_pm::link_global(name, &dir) {
                        eprintln!("Warning: Failed to link global package {name}: {e}");
                    }
                }
                println!("Linked {} workspace members", member_names.len());
            }

            // Install .bin scripts
            let nm = dir.join("node_modules");
            if nm.exists() {
                match klyron_pm::install_all_bin_scripts(&nm) {
                    Ok(count) => if count > 0 { println!("Installed {count} binary scripts"); },
                    Err(e) => eprintln!("Warning: Failed to install bin scripts: {e}"),
                }
            }

            Ok(())
        }
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

fn generate_klyron_lock_after_install(dir: &Path) -> anyhow::Result<()> {
    use klyron_pm::{PackageManager, InstallResult, generate_lockfile};
    let pm = PackageManager::new(dir);
    let opts = klyron_pm::InstallOptions::default();
    let nodes = pm.install(&opts).map_err(|e| anyhow::anyhow!("{e}"))?;
    let now = std::time::SystemTime::now();
    let result = InstallResult {
        nodes,
        start_time: now,
        end_time: now,
    };
    let lockfile_path = dir.join("klyron.lock");
    generate_lockfile(&result, &lockfile_path).map_err(|e| anyhow::anyhow!("{e}"))
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

pub fn run_update(force: bool) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    let pm = detect_package_manager(&dir);
    match project {
        "node" => {
            if force {
                generate_klyron_lock_after_install(&dir)?;
                return Ok(());
            }
            crate::run_cmd(pm, &["update"], &dir)?;
            generate_klyron_lock_after_install(&dir)
        }
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

pub fn run_lock_verify() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let lockfile_path = dir.join("klyron.lock");
    if !lockfile_path.exists() {
        anyhow::bail!("klyron.lock not found in {}", dir.display());
    }
    let data = std::fs::read(&lockfile_path)?;
    let lock = klyron_pm::lockfile::KlyronLockfile::from_bytes(&data)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let mismatches = lock.verify_integrity(&dir)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    if mismatches.is_empty() {
        println!("klyron.lock verified: {} packages intact", lock.packages.len());
    } else {
        for m in &mismatches {
            println!("warning: {}", m);
        }
        anyhow::bail!("{} integrity mismatches found", mismatches.len());
    }
    Ok(())
}

pub fn run_lock_update(force: bool) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    if force {
        generate_klyron_lock_after_install(&dir)?;
        println!("klyron.lock force-updated");
    } else {
        run_update(false)?;
    }
    Ok(())
}

// ── Pack ──────────────────────────────────────────────────────────────────

pub fn run_pack(output: Option<&Path>) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let pkg_json = dir.join("package.json");
    if !pkg_json.exists() {
        anyhow::bail!("No package.json found in {}", dir.display());
    }

    match klyron_pm::pack_package(&dir, output) {
        Ok(path) => {
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            println!("Packed: {} ({} bytes)", path.display(), size);
            Ok(())
        }
        Err(e) => anyhow::bail!("Pack failed: {e}"),
    }
}

// ── Link / Unlink ─────────────────────────────────────────────────────────

pub fn run_link(args: &LinkArgs) -> anyhow::Result<()> {
    if args.global {
        let dir = std::env::current_dir()?;
        let global_dir = args.global_dir.as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(klyron_pm::get_global_link_dir);
        let link_path = klyron_pm::link_package(&dir, &global_dir)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("Linked {} -> global store", link_path.display());
        Ok(())
    } else if let Some(package_name) = &args.package_name {
        let target_dir = args.target_dir.as_deref()
            .map(|s| PathBuf::from(s))
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        let link_dest = klyron_pm::link_global(package_name, &target_dir)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("Linked {} -> {}", package_name, link_dest.display());
        Ok(())
    } else if let Some(package_dir) = &args.package_dir {
        let dir = Path::new(package_dir);
        let global_dir = args.global_dir.as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(klyron_pm::get_global_link_dir);
        let link_path = klyron_pm::link_package(dir, &global_dir)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("Linked {} -> global store", link_path.display());
        Ok(())
    } else {
        anyhow::bail!("Usage: klyron link [package_dir] or klyron link --global or klyron link <package_name> <target_dir>");
    }
}

pub fn run_unlink_global(package_name: &str) -> anyhow::Result<()> {
    klyron_pm::unlink_package(package_name)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Unlinked {package_name}");
    Ok(())
}

// ── Dist-tag ──────────────────────────────────────────────────────────────

pub fn run_dist_tag(action: DistTagAction) -> anyhow::Result<()> {
    match action {
        DistTagAction::Add { package, version, tag, registry } => {
            klyron_pm::add_dist_tag(&package, &version, &tag, &registry)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Added tag '{tag}' -> {package}@{version}");
            Ok(())
        }
        DistTagAction::Remove { package, tag, registry } => {
            klyron_pm::remove_dist_tag(&package, &tag, &registry)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Removed tag '{tag}' from {package}");
            Ok(())
        }
        DistTagAction::List { package, registry } => {
            let tags = klyron_pm::list_dist_tags(&package, &registry)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Dist-tags for {package}:");
            let mut sorted: Vec<_> = tags.into_iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            for (tag, version) in &sorted {
                println!("  {tag}: {version}");
            }
            Ok(())
        }
    }
}

// ── Why ───────────────────────────────────────────────────────────────────

pub fn run_why(package_name: &str) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;

    // Try to load klyron.lock first
    let lock_path = dir.join("klyron.lock");
    let lockfile = if lock_path.exists() {
        let data = std::fs::read(&lock_path)?;
        Some(klyron_pm::lockfile::KlyronLockfile::from_bytes(&data)
            .map_err(|e| anyhow::anyhow!("{e}"))?)
    } else {
        None
    };

    if let Some(lock) = lockfile {
        // Convert to KlyronLockfile (the main one)
        let mut klock = KlyronLockfile::new(None);
        for (key, pkg) in &lock.packages {
            klock.packages.insert(key.clone(), klyron_pm::KlyronLockPackage {
                version: pkg.version.clone(),
                resolved: Some(pkg.resolved.clone()),
                integrity: Some(pkg.integrity.clone()),
                link: None,
                dev: None,
                optional: None,
                dependencies: Some(pkg.dependencies.clone()),
                optional_dependencies: Some(pkg.optional_dependencies.clone()),
                peer_dependencies: Some(pkg.peer_dependencies.clone()),
                engines: None,
            });
        }
        match klyron_pm::why_package(package_name, &klock) {
            Ok(paths) => {
                if paths.is_empty() {
                    println!("{package_name} is a root dependency or not found");
                } else {
                    println!("{package_name} is depended on via:");
                    for wp in &paths {
                        let indent = "  ".repeat(wp.depth);
                        println!("{indent}{}", wp.path.join(" -> "));
                    }
                }
            }
            Err(e) => anyhow::bail!("{e}"),
        }
    } else {
        // Fall back to npm
        let status = std::process::Command::new("npm")
            .args(["why", package_name])
            .current_dir(&dir)
            .status()?;
        if !status.success() {
            anyhow::bail!("npm why failed");
        }
    }
    Ok(())
}
