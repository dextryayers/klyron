use clap::Args;
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
}

#[derive(Args)]
pub struct LogoutArgs {
    pub registry: Option<String>,
}

#[derive(Args)]
pub struct SearchArgs {
    pub query: String,
    #[arg(long)]
    pub registry: Option<String>,
}

#[derive(Args)]
pub struct InfoArgs {
    pub package: String,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub registry: Option<String>,
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

pub fn run_login(registry: Option<&str>) -> anyhow::Result<()> {
    let reg = registry.unwrap_or("npm");
    match reg {
        "npm" => crate::run_cmd("npm", &["login"], &std::env::current_dir()?),
        "pypi" => crate::run_cmd("python3", &["-m", "twine", "login"], &std::env::current_dir()?),
        _ => crate::run_cmd("npm", &["login"], &std::env::current_dir()?),
    }
}

pub fn run_logout(registry: Option<&str>) -> anyhow::Result<()> {
    let reg = registry.unwrap_or("npm");
    match reg {
        "npm" => crate::run_cmd("npm", &["logout"], &std::env::current_dir()?),
        _ => crate::run_cmd("npm", &["logout"], &std::env::current_dir()?),
    }
}

pub fn run_whoami() -> anyhow::Result<()> {
    let registry = RegistryClient::from_kind(RegistryKind::Npm);
    match registry.whoami() {
        Ok(user) => {
            println!("{user}");
            Ok(())
        }
        Err(_e) => {
            crate::run_cmd("npm", &["whoami"], &std::env::current_dir()?)
        }
    }
}

pub fn run_search(query: &str) -> anyhow::Result<()> {
    let registry = RegistryClient::detect(query);
    let result = registry.search(query, 20)?;
    if result.results.is_empty() {
        println!("No results found for '{query}'");
    } else {
        println!("Search results for '{query}' ({} total):", result.total);
        for pkg in &result.results {
            let desc = pkg.description.as_deref().unwrap_or("");
            println!("  {}@{}  {}", pkg.name, pkg.version, desc);
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
