use clap::Args;
use std::path::{Path, PathBuf};
use crate::anim::{PulseSpinner, cmd_header, success_banner};

enum InstallEvent {
    Progress(usize, usize, String),
    Done(Result<(), String>),
}

fn spinner_dot() -> String {
    crate::Color::CYAN.paint("\u{25CB}")
}

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
    Migrate {
        #[arg(long)]
        keep: bool,
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
    if dir.join("bun.lock").exists() || dir.join("bun.lockb").exists() { "bun" }
    else if dir.join("pnpm-lock.yaml").exists() { "pnpm" }
    else if dir.join("yarn.lock").exists() { "yarn" }
    else { "npm" }
}

pub fn run_add(packages: &[String], dev: bool) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);
    let _pm = detect_package_manager(&dir);

    match project {
        "node" => {
            add_packages_node(&dir, packages, dev)
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

/// Resolve, download and record packages into package.json + klyron.lock
/// using klyron's own registry client (no npm delegation).
fn add_packages_node(dir: &Path, packages: &[String], dev: bool) -> anyhow::Result<()> {
    use klyron_pm::resolver::fetch_package_metadata;
    use klyron_pm::{download_and_extract_tarball, lockfile::LockfilePackage,
                    KlyronLockfile};
    use semver::{Version, VersionReq};

    let pkg_path = dir.join("package.json");
    let mut pkg: serde_json::Value = if pkg_path.exists() {
        let content = std::fs::read_to_string(&pkg_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({"name": "my-app", "version": "1.0.0"})
    };
    if !pkg.is_object() {
        anyhow::bail!("package.json is not an object");
    }

    let dep_key = if dev { "devDependencies" } else { "dependencies" };
    let mut deps_obj = pkg
        .get_mut(dep_key)
        .cloned()
        .and_then(|v| if v.is_object() { Some(v) } else { None })
        .unwrap_or_else(|| serde_json::json!({}));

    let nm = dir.join("node_modules");
    std::fs::create_dir_all(&nm)?;

    let mut new_entries: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for spec in packages {
        let (name, range) = match spec.split_once('@') {
            Some((n, r)) if !n.is_empty() => (n.to_string(), r.to_string()),
            _ => (spec.clone(), "*".to_string()),
        };
        let meta = fetch_package_metadata(&name)
            .map_err(|e| anyhow::anyhow!("Failed to fetch {}: {e}", name))?;

        let chosen = if range == "*" || range.is_empty() {
            meta.dist_tags
                .get("latest")
                .and_then(|v| Version::parse(v).ok())
                .or_else(|| {
                    meta.versions
                        .keys()
                        .filter_map(|v| Version::parse(v).ok())
                        .max()
                })
        } else {
            let req = VersionReq::parse(&range).map_err(|e| anyhow::anyhow!("{e}"))?;
            meta.versions
                .keys()
                .filter_map(|v| Version::parse(v).ok())
                .filter(|v| req.matches(v))
                .max()
        };
        let ver = chosen.ok_or_else(|| anyhow::anyhow!("No version of {name} matches {range}"))?;
        let ver_str = ver.to_string();

        let url = meta
            .versions
            .get(&ver_str)
            .map(|vm| vm.resolved.clone())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| {
                format!("https://registry.npmjs.org/{name}/-/{name}-{ver_str}.tgz")
            });

        let pkg_dir = nm.join(&name);
        if !pkg_dir.join("package.json").exists() {
            download_and_extract_tarball(&url, &pkg_dir)
                .map_err(|e| anyhow::anyhow!("Failed to download {name}: {e}"))?;
        }
        new_entries.insert(name.clone(), ver_str.clone());

        if let Some(obj) = deps_obj.as_object_mut() {
            obj.insert(name.clone(), serde_json::Value::String(ver_str.clone()));
            pkg.as_object_mut()
                .unwrap()
                .insert(dep_key.to_string(), deps_obj.clone());
        }
        println!("{} added {}@{}", if dev { "devDependency" } else { "dependency" }, name, ver_str);
    }

    std::fs::write(&pkg_path, serde_json::to_string_pretty(&pkg)?)?;

    // Update klyron.lock (binary). Load existing, merge, write back.
    let lock_path = dir.join("klyron.lock");
    let mut lock = if lock_path.exists() {
        let data = std::fs::read(&lock_path)?;
        KlyronLockfile::from_bytes(&data).map_err(|e| anyhow::anyhow!("{e}"))?
    } else {
        KlyronLockfile::new()
    };
    for (name, ver_str) in &new_entries {
        let url = format!("https://registry.npmjs.org/{name}/-/{name}-{ver_str}.tgz");
        lock.packages.insert(
            name.clone(),
            LockfilePackage {
                name: name.clone(),
                version: ver_str.clone(),
                resolved: url,
                integrity: String::new(),
                integrity_hashes: Vec::new(),
                signature: None,
                signer: None,
                dependencies: std::collections::HashMap::new(),
                optional_dependencies: std::collections::HashMap::new(),
                peer_dependencies: std::collections::HashMap::new(),
                bin: None,
                has_node_modules: false,
                install_time_ms: 0,
            },
        );
    }
    let bytes = lock.to_bytes().map_err(|e| anyhow::anyhow!("{e}"))?;
    std::fs::write(&lock_path, &bytes)?;

    Ok(())
}

fn ensure_gitignore_has_klyron_lock(dir: &std::path::Path) -> anyhow::Result<()> {
    let gitignore = dir.join(".gitignore");
    if gitignore.exists() {
        let content = std::fs::read_to_string(&gitignore)?;
        if !content.lines().any(|l| l.trim() == "/klyron.lock") {
            let mut new_content = content;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str("/klyron.lock\n");
            std::fs::write(&gitignore, new_content)?;
            println!("Added /klyron.lock to .gitignore");
        }
    }
    Ok(())
}

pub fn run_install(frozen: bool) -> anyhow::Result<()> {
    cmd_header("install", "Installing project dependencies");
    let mut spinner = PulseSpinner::new("Analyzing project...");

    let dir = std::env::current_dir()?;
    let project = crate::detect_project_type(&dir);

    let _ = ensure_gitignore_has_klyron_lock(&dir);

    if project == "node" {
        let (framework, version) = crate::detect_framework_from_pkg(&dir);
        let version_display = version.as_deref().unwrap_or("");
        let fw_label = if version_display.is_empty() { framework.clone() } else { format!("{} {}", framework, version_display) };
        let fw_colored = crate::Color::CYAN.paint(&fw_label);
        crate::log_info(format!("  {} {}  {}", spinner_dot(), crate::Color::BOLD.paint("Framework:"), fw_colored));

        let config_path = dir.join("klyron.json");
        if !config_path.exists() {
            let language = crate::detect_project_language(&dir);
            let project_name = dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("my-app")
                .to_string();

            // Framework-specific config values
            let (dev_port, build_dir) = match framework.as_str() {
                "Next.js" => (3000, ".next"),
                "Vue" | "Nuxt" => (5173, "dist"),
                "Svelte" | "SvelteKit" => (5173, "build"),
                "React" => (3000, "dist"),
                "Angular" => (4200, "dist"),
                "Astro" => (4321, "dist"),
                "NestJS" => (3000, "dist"),
                "Express" => (3000, "dist"),
                "Fastify" => (3000, "dist"),
                "Hono" => (3000, "dist"),
                "Solid" => (3000, "dist"),
                "Gatsby" => (8000, "public"),
                "Remix" => (3000, "build"),
                "Preact" => (5173, "dist"),
                "Lit" => (5173, "dist"),
                _ => (3000, "dist"),
            };

            let config = serde_json::json!({
                "name": project_name,
                "version": "0.1.0",
                "type": "node",
                "framework": framework,
                "frameworkVersion": version_display,
                "language": language,
                "compiler": {
                    "target": "esnext",
                    "module": "esnext",
                    "minify": false,
                    "sourcemap": false
                },
                "dev": {
                    "port": dev_port,
                    "hmr": true
                },
                "build": {
                    "outDir": build_dir,
                    "minify": true
                }
            });
            let content = serde_json::to_string_pretty(&config)?;
            std::fs::write(&config_path, content)?;
            let check = crate::Color::GREEN.paint("\u{2713}");
            crate::log_info(format!("  {}  {}  ({framework} config)", check, config_path.display()));
        }

    }

    match project {
        "node" => {
            spinner.done("Project analyzed");
            let mut install_spinner = PulseSpinner::new("Installing dependencies...");

            let (tx, rx) = std::sync::mpsc::channel::<InstallEvent>();
            let install_dir = dir.clone();
            let progress_tx = tx.clone();
            let done_tx = tx.clone();
            std::thread::spawn(move || {
                let cb = move |current: usize, total: usize, name: &str| {
                    let _ = progress_tx.send(InstallEvent::Progress(current + 1, total, name.to_string()));
                };
                let result = klyron_pm::install_with_lockfile(&install_dir, frozen, Some(&cb));
                let _ = done_tx.send(InstallEvent::Done(result.map_err(|e| e.to_string())));
            });

            let install_result = loop {
                match rx.recv() {
                    Ok(InstallEvent::Progress(current, total, ref name)) => {
                        install_spinner.set_message(&format!("Installing dependencies... ({current}/{total}) {name}"));
                    }
                    Ok(InstallEvent::Done(result)) => break result,
                    Err(_) => {
                        install_spinner.fail("Install process terminated unexpectedly");
                        anyhow::bail!("Install thread terminated unexpectedly");
                    }
                }
            };

            match install_result {
                Ok(()) => install_spinner.done("Dependencies installed"),
                Err(ref e) => {
                    install_spinner.fail(&format!("Install failed: {e}"));
                    anyhow::bail!("Install failed: {e}");
                }
            }

            success_banner("Install complete");

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
        "laravel" => {
            let mut bar = crate::anim::GradientBar::new(50, "Installing Composer packages...");
            let r = crate::run_cmd("composer", &["install"], &dir);
            bar.finish_with("Composer packages installed");
            success_banner("Install complete");
            r
        }
        "python" => {
            let mut bar = crate::anim::GradientBar::new(30, "Installing Python packages...");
            let r = if dir.join("Pipfile").exists() { crate::run_cmd("pipenv", &["install"], &dir) }
            else if dir.join("poetry.lock").exists() { crate::run_cmd("poetry", &["install"], &dir) }
            else if dir.join("requirements.txt").exists() { crate::run_cmd("pip", &["install", "-r", "requirements.txt"], &dir) }
            else { anyhow::bail!("No requirements file found") };
            bar.finish_with("Python packages installed");
            success_banner("Install complete");
            r
        }
        "ruby" => {
            let mut bar = crate::anim::GradientBar::new(20, "Installing Ruby gems...");
            let r = crate::run_cmd("bundle", &["install"], &dir);
            bar.finish_with("Ruby gems installed");
            success_banner("Install complete");
            r
        }
        "rust" => {
            let mut bar = crate::anim::GradientBar::new(60, "Building Rust project...");
            let r = crate::run_cmd("cargo", &["build"], &dir);
            bar.finish_with("Rust project built");
            success_banner("Install complete");
            r
        }
        "go" => {
            let mut bar = crate::anim::GradientBar::new(15, "Downloading Go modules...");
            let r = crate::run_cmd("go", &["mod", "download"], &dir);
            bar.finish_with("Go modules downloaded");
            success_banner("Install complete");
            r
        }
        _ => anyhow::bail!("Install not supported for {project}"),
    }
}

fn generate_klyron_lock_after_install(dir: &Path) -> anyhow::Result<()> {
    use klyron_pm::{PackageManager, InstallResult, generate_lockfile};
    let pm = PackageManager::new(dir);
    let opts = klyron_pm::InstallOptions::default();
    let nodes = match pm.install(&opts) {
        Ok(n) => n,
        Err(e) => {
            crate::log_info(format!(
                "{} {}",
                crate::Color::YELLOW.paint("\u{26A0}"),
                format!("klyron lock generation skipped: {e}")
            ));
            return Ok(());
        }
    };
    let now = std::time::SystemTime::now();
    let result = InstallResult {
        nodes,
        start_time: now,
        end_time: now,
    };
    let lockfile_path = dir.join("klyron.lock");
    if let Err(e) = generate_lockfile(&result, &lockfile_path) {
        crate::log_info(format!(
            "{} {}",
            crate::Color::YELLOW.paint("\u{26A0}"),
            format!("klyron lock write skipped: {e}")
        ));
    }
    Ok(())
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
    let klyron_lock = dir.join("klyron.lock");
    if klyron_lock.exists() {
        let data = std::fs::read(&klyron_lock)?;
        let lock = klyron_pm::lockfile::KlyronLockfile::from_bytes(&data)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut outdated = Vec::new();
        let mut current: Vec<String> = Vec::new();
        for key in lock.packages.keys() {
            if let Some(at) = key.rfind('@') {
                current.push(key[..at].to_string());
            }
        }
        current.sort();
        current.dedup();
        for name in &current {
            if let Some(pkg) = lock.get_package(name) {
                let wanted = klyron_pm::resolve_version(name, &format!("^{}", pkg.version))
                    .unwrap_or_else(|_| pkg.version.clone());
                let latest = klyron_pm::resolve_version(name, ">=0.0.0")
                    .unwrap_or_else(|_| pkg.version.clone());
                if wanted != latest || pkg.version != latest {
                    outdated.push(klyron_pm::OutdatedPackage {
                        name: name.clone(),
                        current: pkg.version.clone(),
                        wanted,
                        latest,
                    });
                }
            }
        }
        if outdated.is_empty() {
            println!("All packages are up to date");
        } else {
            println!("{:<30} Current   Wanted    Latest", "Package");
            println!("{}", "-".repeat(70));
            for p in &outdated {
                println!("{:<30} {:<9} {:<9} {:<9}", p.name, p.current, p.wanted, p.latest);
            }
        }
        return Ok(());
    }
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

pub fn run_audit_licenses() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let nm = dir.join("node_modules");
    if !nm.exists() {
        anyhow::bail!("node_modules not found in {}", dir.display());
    }

    let allowed = vec![
        "MIT".to_string(),
        "Apache-2.0".to_string(),
        "ISC".to_string(),
        "BSD-2-Clause".to_string(),
        "BSD-3-Clause".to_string(),
        "Unlicense".to_string(),
        "CC0-1.0".to_string(),
    ];

    let deps = klyron_pm::license::scan_node_modules_for_licenses(&nm)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let violations = klyron_pm::license::check_license_compliance(&deps, &allowed);

    if violations.is_empty() {
        println!("All {} packages have approved licenses", deps.len());
    } else {
        println!("License compliance violations ({}):", violations.len());
        for v in &violations {
            let label = if v.required { "required" } else { "unknown" };
            println!("  {}@{} - {} ({label})", v.package, v.version, v.license);
        }
    }
    Ok(())
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

pub fn run_lock_migrate(keep: bool) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let npm_lock = dir.join("package-lock.json");
    let yarn_lock = dir.join("yarn.lock");
    let klyron_lock = dir.join("klyron.lock");

    if klyron_lock.exists() {
        println!("klyron.lock already exists, nothing to migrate");
        return Ok(());
    }

    let source = if npm_lock.exists() {
        Some((npm_lock.as_path(), "package-lock.json"))
    } else if yarn_lock.exists() {
        Some((yarn_lock.as_path(), "yarn.lock"))
    } else {
        None
    };

    let (source_path, source_name) = match source {
        Some(pair) => pair,
        None => anyhow::bail!("No package-lock.json or yarn.lock found in {}", dir.display()),
    };

    let klock = if source_name == "package-lock.json" {
        klyron_pm::migrate_from_npm_lockfile(source_path)
            .map_err(|e| anyhow::anyhow!("{e}"))?
    } else {
        klyron_pm::migrate_from_yarn_lockfile(source_path)
            .map_err(|e| anyhow::anyhow!("{e}"))?
    };

    let bytes = klock.to_bytes()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    std::fs::write(&klyron_lock, &bytes)?;

    let count = klock.packages.len();
    println!("Migrated {count} packages from {source_name} to klyron.lock");

    if !keep {
        std::fs::remove_file(source_path)?;
        println!("Removed {source_name} (use --keep to keep it)");
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
        match klyron_pm::why_package(package_name, &lock) {
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

// ── Framework detection helpers ──────────────────────────────────────────

fn count_node_modules(dir: &std::path::Path) -> usize {
    let nm = dir.join("node_modules");
    if !nm.exists() {
        return 0;
    }
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(&nm) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') && name != "package-lock.json" {
                    count += 1;
                }
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires network access to npm registry"]
    fn test_run_add_node_no_npm() {
        let tmp = std::env::temp_dir().join(format!("klyron_add_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("package.json"),
            r#"{"name":"demo","version":"1.0.0"}"#,
        )
        .unwrap();

        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp).unwrap();
        let result = run_add(&["left-pad".to_string()], false);
        let _ = std::env::set_current_dir(prev);

        assert!(result.is_ok(), "run_add should succeed: {:?}", result.err());
        assert!(
            tmp.join("node_modules").join("left-pad").join("package.json").exists(),
            "package must be downloaded"
        );
        assert!(tmp.join("klyron.lock").exists(), "klyron.lock must be written");

        let pkg: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.join("package.json")).unwrap())
                .unwrap();
        assert!(
            pkg.get("dependencies")
                .and_then(|d| d.get("left-pad"))
                .is_some(),
            "package.json dependencies must list left-pad"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
