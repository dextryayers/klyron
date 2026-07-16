use clap::{Args, Subcommand};
use klyron_registry::{RegistryClient, RegistryKind};
use klyron_pm;

#[derive(Args)]
pub struct PublishArgs {
    #[arg(long)]
    pub registry: Option<String>,
    #[arg(long)]
    pub tag: Option<String>,
    #[arg(long)]
    pub access: Option<String>,
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Args)]
pub struct UnpublishArgs {
    pub name: String,
    #[arg(long)]
    pub registry: Option<String>,
}

#[derive(Args)]
pub struct LoginArgs {
    pub registry: Option<String>,
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Args)]
pub struct LogoutArgs {
    pub registry: Option<String>,
}

fn auth_store_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
        .join("klyron")
        .join("auth.json")
}

fn load_auth() -> serde_json::Value {
    let path = auth_store_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    }
}

fn save_auth(auth: &serde_json::Value) -> anyhow::Result<()> {
    let path = auth_store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(auth)?)?;
    Ok(())
}

#[derive(Args)]
pub struct SearchArgs {
    pub query: String,
    #[arg(long)]
    pub registry: Option<String>,
    #[arg(long)]
    pub exact: bool,
    #[arg(long)]
    pub author: Option<String>,
    #[arg(long)]
    pub page: Option<u32>,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct InfoArgs {
    pub package: String,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub registry: Option<String>,
}

#[derive(Subcommand)]
pub enum RegistryAction {
    Add {
        name: String,
        url: String,
    },
    Remove {
        name: String,
    },
    List,
    Ping {
        url: String,
    },
    MapScope {
        scope: String,
        registry: String,
    },
    UnmapScope {
        scope: String,
    },
    ListScopes,
    GenerateKeypair,
}

fn resolve_registry(name: Option<&str>) -> RegistryClient {
    match name {
        Some("npm") => RegistryClient::from_kind(RegistryKind::Npm),
        Some("pypi") => RegistryClient::from_kind(RegistryKind::PyPI),
        Some("rubygems") => RegistryClient::from_kind(RegistryKind::RubyGems),
        Some("cargo") => RegistryClient::from_kind(RegistryKind::Cargo),
        Some("packagist") => RegistryClient::from_kind(RegistryKind::Packagist),
        Some("goproxy") => RegistryClient::from_kind(RegistryKind::GoProxy),
        Some(other) => {
            eprintln!("Unknown registry '{other}', defaulting to npm");
            RegistryClient::from_kind(RegistryKind::Npm)
        }
        None => RegistryClient::from_kind(RegistryKind::Npm),
    }
}

pub fn run_publish(args: &PublishArgs) -> anyhow::Result<()> {
    let registry = resolve_registry(args.registry.as_deref());
    let dir = std::env::current_dir()?;

    if dir.join("package.json").exists() {
        let registry_url = "https://registry.npmjs.org";
        let token = args.token.as_deref();
        klyron_pm::publish_package(&dir, registry_url, token)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("Published successfully");
        Ok(())
    } else if dir.join("Cargo.toml").exists() {
        let tarball = if dir.join("Cargo.toml").exists() {
            crate::run_cmd("cargo", &["package"], &dir)?;
            let toml_str = std::fs::read_to_string(dir.join("Cargo.toml"))?;
            let toml_val: toml::Value = toml::from_str(&toml_str)?;
            let pkg = toml_val.get("package");
            let name = pkg.and_then(|p| p.get("name")).and_then(|v| v.as_str()).unwrap_or("crate");
            let version = pkg.and_then(|p| p.get("version")).and_then(|v| v.as_str()).unwrap_or("0.1.0");
            let target = dir.join("target").join("package");
            if target.join(format!("{name}-{version}.crate")).exists() {
                target.join(format!("{name}-{version}.crate"))
            } else {
                anyhow::bail!("Run `cargo package` first to generate .crate file")
            }
        } else {
            anyhow::bail!("No package manifest found (package.json or Cargo.toml)")
        };

        let data = std::fs::read(&tarball)?;
        registry.publish("package", &data, args.tag.as_deref())?;
        println!("Published successfully");
        let _ = std::fs::remove_file(&tarball);
        Ok(())
    } else {
        anyhow::bail!("No package manifest found (package.json or Cargo.toml)")
    }
}

pub fn run_unpublish(name: &str) -> anyhow::Result<()> {
    let registry = RegistryClient::detect(name);
    registry.unpublish(name)?;
    Ok(())
}

pub fn run_login(args: LoginArgs) -> anyhow::Result<()> {
    if let Some(token) = args.token {
        let mut auth = load_auth();
        let reg = args.registry.as_deref().unwrap_or("npm");
        let user = format!("token-user-{}", &token[..token.len().min(8)]);
        auth[reg] = serde_json::json!({
            "token": token,
            "user": user,
        });
        save_auth(&auth)?;
        let user = auth[reg]["user"].as_str().unwrap_or("unknown");
        eprintln!(
            "{} Logged in as {} (registry: {})",
            crate::Color::GREEN.paint("\u{2705}"),
            crate::Color::BOLD.paint(user),
            reg,
        );
        Ok(())
    } else {
        let reg = args.registry.as_deref().unwrap_or("npm");
        match reg {
            "npm" => crate::run_cmd("npm", &["login"], &std::env::current_dir()?),
            "pypi" => crate::run_cmd("python3", &["-m", "twine", "login"], &std::env::current_dir()?),
            _ => crate::run_cmd("npm", &["login"], &std::env::current_dir()?),
        }
    }
}

pub fn run_logout(args: LogoutArgs) -> anyhow::Result<()> {
    let reg = args.registry.as_deref().unwrap_or("npm");
    let mut auth = load_auth();
    if auth.as_object().map(|o| o.contains_key(reg)).unwrap_or(false) {
        auth.as_object_mut().map(|o| o.remove(reg));
        save_auth(&auth)?;
        eprintln!(
            "{} Logged out from registry: {}",
            crate::Color::GREEN.paint("\u{2705}"),
            reg,
        );
        Ok(())
    } else {
        match reg {
            "npm" => crate::run_cmd("npm", &["logout"], &std::env::current_dir()?),
            _ => crate::run_cmd("npm", &["logout"], &std::env::current_dir()?),
        }
    }
}

pub fn run_whoami() -> anyhow::Result<()> {
    let auth = load_auth();
    if let Some(obj) = auth.as_object() {
        if !obj.is_empty() {
            for (reg, data) in obj {
                if let Some(user) = data.get("user").and_then(|v| v.as_str()) {
                    println!("{user} (registry: {reg})");
                }
            }
            return Ok(());
        }
    }

    let registry = RegistryClient::from_kind(RegistryKind::Npm);
    match registry.whoami() {
        Ok(user) => {
            println!("{user}");
            Ok(())
        }
        Err(_e) => {
            let npm_check = std::process::Command::new("npm").arg("--version").output();
            match npm_check {
                Ok(_) => {
                    let dir = std::env::current_dir()?;
                    let status = std::process::Command::new("npm")
                        .args(["whoami"])
                        .current_dir(&dir)
                        .status();
                    match status {
                        Ok(s) if s.success() => Ok(()),
                        _ => {
                            println!("Not logged in. Run `klyron login` to authenticate.");
                            Ok(())
                        }
                    }
                }
                Err(_) => {
                    println!("Not logged in. Run `klyron login` to authenticate.");
                    Ok(())
                }
            }
        }
    }
}

pub fn run_search(args: &SearchArgs) -> anyhow::Result<()> {
    let page = args.page.unwrap_or(1);
    let limit = args.limit.unwrap_or(20);

    let results = klyron_pm::search::search_packages(
        &args.query,
        args.exact,
        args.author.as_deref(),
        None,
        page,
        limit,
    ).map_err(|e| anyhow::anyhow!("{e}"))?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "results": results.results,
            "total": results.total,
            "page": results.page,
            "limit": results.limit,
        }))?);
    } else if results.results.is_empty() {
        println!("No results found for '{}'", args.query);
    } else {
        println!("Search results for '{}' ({} total, page {}):", args.query, results.total, results.page);
        for pkg in &results.results {
            let desc = pkg.description.as_deref().unwrap_or("");
            let downloads = pkg.downloads.map(|d| format!(" ({}/mo)", d)).unwrap_or_default();
            println!("  {}@{} {}{}", pkg.name, pkg.version, desc, downloads);
        }
    }
    Ok(())
}

pub fn run_registry(action: RegistryAction) -> anyhow::Result<()> {
    use klyron_pm::registry;
    match action {
        RegistryAction::Add { name, url } => {
            registry::add_registry(&name, &url)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Added registry '{name}' -> {url}");
        }
        RegistryAction::Remove { name } => {
            registry::remove_registry(&name)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Removed registry '{name}'");
        }
        RegistryAction::List => {
            let regs = registry::list_registries();
            if regs.is_empty() {
                println!("No custom registries configured");
            } else {
                println!("Configured registries:");
                for r in &regs {
                    let tok = if r.token.is_some() { " (authenticated)" } else { "" };
                    println!("  {} -> {} (priority: {}){}", r.name, r.url, r.priority, tok);
                }
            }
        }
        RegistryAction::Ping { url } => {
            let ok = registry::ping_registry(&url);
            if ok {
                println!("Registry at {url} is reachable");
            } else {
                eprintln!("Registry at {url} is NOT reachable");
            }
        }
        RegistryAction::MapScope { scope, registry: reg } => {
            registry::map_scope(&scope, &reg)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Mapped scope {scope} -> registry {reg}");
        }
        RegistryAction::UnmapScope { scope } => {
            registry::unmap_scope(&scope)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Unmapped scope {scope}");
        }
        RegistryAction::ListScopes => {
            let scopes = registry::list_mapped_scopes();
            if scopes.is_empty() {
                println!("No scope mappings configured");
            } else {
                println!("Scope mappings:");
                for (scope, reg) in &scopes {
                    println!("  {scope} -> {reg}");
                }
            }
        }
        RegistryAction::GenerateKeypair => {
            let (secret, public) = klyron_pm::signing::generate_keypair();
            klyron_pm::signing::save_keypair("default", &secret, &public)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Generated keypair and saved to ~/.config/klyron/keys/");
            println!("Public key:\n{}", hex::encode(&public));
        }
    }
    Ok(())
}

pub fn run_info(package: &str, json: bool) -> anyhow::Result<()> {
    let registry_url = "https://registry.npmjs.org";

    match klyron_pm::package_info(package, registry_url) {
        Ok(info) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "name": info.name,
                    "description": info.description,
                    "latest_version": info.latest_version,
                    "all_versions": info.all_versions,
                    "maintainers": info.maintainers,
                    "homepage": info.homepage,
                    "license": info.license,
                    "repository": info.repository,
                }))?);
            } else {
                println!("{}@{}", info.name, info.latest_version);
                if let Some(desc) = &info.description {
                    println!("  Description: {desc}");
                }
                if let Some(lic) = &info.license {
                    println!("  License: {lic}");
                }
                if let Some(home) = &info.homepage {
                    println!("  Homepage: {home}");
                }
                if let Some(repo) = &info.repository {
                    println!("  Repository: {repo}");
                }
                if !info.maintainers.is_empty() {
                    let names: Vec<&str> = info.maintainers.iter().map(|m| m.name.as_str()).collect();
                    println!("  Maintainers: {}", names.join(", "));
                }
                println!("  Versions:");
                for v in &info.all_versions {
                    if v == &info.latest_version {
                        println!("    {v} (latest)");
                    } else {
                        println!("    {v}");
                    }
                }
            }
            Ok(())
        }
        Err(_) => {
            // Fallback to npm info
            let dir = std::env::current_dir()?;
            let status = std::process::Command::new("npm")
                .args(["info", package])
                .current_dir(&dir)
                .status()
                .map_err(|e| anyhow::anyhow!("Failed to run npm: {e}"))?;
            if !status.success() {
                anyhow::bail!("npm info failed for {package}");
            }
            Ok(())
        }
    }
}
