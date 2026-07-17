use clap::Subcommand;
use klyron_plugin::manifest::KLYRON_API_VERSION;
use klyron_plugin::PluginRegistry;
use std::path::{Path, PathBuf};

const PLUGINS_DIR_NAME: &str = "plugins";

#[derive(Subcommand)]
pub enum PluginAction {
    /// Install a plugin from path or URL
    Install {
        source: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        no_rollback: bool,
    },
    /// Remove an installed plugin
    Remove {
        name: String,
    },
    /// List installed plugins
    List,
    /// Show detailed plugin information
    Info {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Update plugin(s) to latest version
    Update {
        name: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Search the plugin marketplace
    Search {
        query: String,
    },
    /// Publish a plugin to the marketplace
    Publish {
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        token: Option<String>,
    },
    /// Scaffold a new plugin project
    Create {
        name: String,
        #[arg(long)]
        lang: Option<String>,
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Enable or disable a plugin
    Toggle {
        name: String,
    },
}

fn plugins_dir() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".klyron").join(PLUGINS_DIR_NAME))
        .unwrap_or_else(|| PathBuf::from("/tmp/klyron-plugins"))
}

fn build_registry(no_rollback: bool) -> anyhow::Result<PluginRegistry> {
    let mut reg = if no_rollback {
        PluginRegistry::new()?.with_no_rollback()
    } else {
        PluginRegistry::new()?
    };
    reg.refresh_installed()?;
    Ok(reg)
}

pub fn run_plugin(action: PluginAction) -> anyhow::Result<()> {
    match action {
        PluginAction::Install { source, force, no_rollback } => {
            run_install(&source, force, no_rollback)
        }
        PluginAction::Remove { name } => run_remove(&name),
        PluginAction::List => run_list(),
        PluginAction::Info { name, json } => run_info(&name, json),
        PluginAction::Update { name, source, force } => {
            run_update(name.as_deref(), source.as_deref(), force)
        }
        PluginAction::Search { query } => run_search(&query),
        PluginAction::Publish { path, token } => run_publish(path.as_deref(), token.as_deref()),
        PluginAction::Create { name, lang, dir } => run_create(&name, lang.as_deref(), dir.as_deref()),
        PluginAction::Toggle { name } => run_toggle(&name),
    }
}

fn run_install(source: &str, force: bool, no_rollback: bool) -> anyhow::Result<()> {
    let source_path = if source.starts_with("http://") || source.starts_with("https://") {
        println!(" Downloading plugin from: {}", source);
        let response = reqwest::blocking::get(source)
            .map_err(|e| anyhow::anyhow!("Failed to download plugin: {}", e))?;
        let bytes = response.bytes()
            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

        let plugin_dir = plugins_dir();
        std::fs::create_dir_all(&plugin_dir)?;

        let temp_path = plugin_dir.join("_temp_install.wasm");
        std::fs::write(&temp_path, &bytes)?;
        temp_path
    } else {
        let p = PathBuf::from(source);
        if !p.exists() {
            anyhow::bail!("Plugin source not found: {}", source);
        }
        p
    };

    let mut reg = build_registry(no_rollback)?;
    let result = reg.install(&source_path, force)?;

    if source_path.file_name().map_or(false, |n| n == "_temp_install.wasm") {
        let _ = std::fs::remove_file(&source_path);
    }

    println!(" Installed plugin: {} v{}", result.name, result.version);
    println!("  Size: {} bytes", result.size_bytes);
    println!("  API: {} ({}compatible)", result.compat.min_version, if result.is_compatible { "" } else { "in" });
    println!("  Hash: {}", result.wasm_hash);
    println!("  Path: {}", reg.plugins_dir().join(&result.name).display());
    Ok(())
}

fn run_remove(name: &str) -> anyhow::Result<()> {
    let mut reg = build_registry(false)?;
    let plugin_dir = reg.plugins_dir().join(name);
    if !plugin_dir.exists() {
        anyhow::bail!("Plugin '{}' is not installed", name);
    }

    reg.remove(name)?;
    println!(" Removed plugin: {}", name);
    Ok(())
}

fn run_list() -> anyhow::Result<()> {
    let reg = build_registry(false)?;
    let infos = reg.get_all_info();

    if infos.is_empty() {
        println!(" No plugins installed.");
        println!(" Install one with: klyron plugin install <path-or-url>");
        return Ok(());
    }

    println!(" Installed plugins ({}):", infos.len());
    println!();

    for info in &infos {
        let enabled_mark = if info.enabled { " " } else { " " };
        let compatible = if info.is_compatible() { "" } else { " [INCOMPATIBLE]" };
        println!(
            "{} {} v{}{}",
            enabled_mark,
            info.manifest.name,
            info.manifest.version,
            compatible
        );
        if let Some(ref desc) = info.manifest.description {
            println!("     {}", desc);
        }
        println!(
            "     Path: {}",
            info.install_path
        );
        if let Some(ref hooks) = info.manifest.hooks {
            println!("     Hooks: {}", hooks.join(", "));
        }
        if !info.manifest.permissions.is_empty() {
            println!("     Permissions: {}", info.manifest.permissions.join(", "));
        }
        println!("     API: {} - {}", info.compat.min_version, info.compat.max_version);
        println!("     Size: {} bytes", info.size_bytes);
        println!("     Hash: {}", info.wasm_hash);
        println!("     Installed: {}", info.installed_at);
        println!();
    }

    println!("{} plugin(s) total.", infos.len());
    Ok(())
}

fn run_info(name: &str, json: bool) -> anyhow::Result<()> {
    let reg = build_registry(false)?;
    let info = reg
        .get_info(name)
        .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

    if json {
        println!("{}", serde_json::to_string_pretty(info)?);
        return Ok(());
    }

    println!(" Plugin: {}", info.manifest.name);
    println!("  Version: {}", info.manifest.version);
    println!("  Enabled: {}", info.enabled);
    if let Some(ref desc) = info.manifest.description {
        println!("  Description: {}", desc);
    }
    if let Some(ref authors) = info.manifest.authors {
        println!("  Authors: {}", authors.join(", "));
    }
    if let Some(ref license) = info.manifest.license {
        println!("  License: {}", license);
    }
    println!("  API Version: {} - {}", info.compat.min_version, info.compat.max_version);
    println!("  Compatible: {}", info.is_compatible());
    println!("  Path: {}", info.install_path);
    println!("  Size: {} bytes", info.size_bytes);
    println!("  Hash: {}", info.wasm_hash);
    println!("  Installed: {}", info.installed_at);

    if let Some(ref hooks) = info.manifest.hooks {
        println!("  Hooks:");
        for hook in hooks {
            let registered = if reg.is_loaded(name) {
                let phase = klyron_plugin::HookPhase::from_str(hook);
                match phase {
                    Some(p) if reg.hook_registry().is_registered(name, &p) => "registered",
                    _ => "not registered",
                }
            } else {
                "not loaded"
            };
            println!("    - {} ({})", hook, registered);
        }
    }

    if !info.manifest.permissions.is_empty() {
        println!("  Permissions: {}", info.manifest.permissions.join(", "));
    }

    if let Some(ref deps) = info.manifest.dependencies {
        println!("  Dependencies:");
        for dep in deps {
            let status = if reg.is_loaded(&dep.name) {
                "loaded"
            } else if dep.optional.unwrap_or(false) {
                "optional (missing)"
            } else {
                "MISSING (required)"
            };
            println!("    - {} v{} [{}]", dep.name, dep.version, status);
        }
    }

    if let Some(ref sandbox) = info.manifest.sandbox {
        println!("  Sandbox:");
        if let Some(mem) = sandbox.max_memory_bytes {
            println!("    Max Memory: {} bytes", mem);
        }
        if let Some(fuel) = sandbox.max_fuel {
            println!("    Max Fuel: {}", fuel);
        }
        if let Some(cpu) = sandbox.max_cpu_ms {
            println!("    Max CPU: {}ms", cpu);
        }
        if let Some(ref domains) = sandbox.allowed_domains {
            println!("    Allowed Domains: {}", domains.join(", "));
        }
        if let Some(ref paths) = sandbox.allowed_paths {
            println!("    Allowed Paths: {}", paths.join(", "));
        }
    }

    Ok(())
}

fn run_update(name: Option<&str>, source: Option<&str>, force: bool) -> anyhow::Result<()> {
    let mut reg = build_registry(false)?;

    match name {
        Some(n) => {
            if !reg.is_loaded(n) {
                anyhow::bail!("Plugin '{}' is not installed", n);
            }

            let new_source = match source {
                Some(s) => PathBuf::from(s),
                None => {
                    let plugin_dir = reg.plugins_dir().join(n);
                    plugin_dir.join(format!("{}.wasm", n))
                }
            };

            if !new_source.exists() {
                anyhow::bail!("Update source not found: {}", new_source.display());
            }

            let result = reg.update(n, &new_source, force)?;
            println!(" Updated plugin: {} v{}", result.name, result.version);
        }
        None => {
            let names: Vec<String> = reg.get_all_info().iter().map(|i| i.manifest.name.clone()).collect();
            if names.is_empty() {
                println!(" No plugins installed.");
                return Ok(());
            }

            for name in &names {
                let plugin_dir = reg.plugins_dir().join(name);
                let wasm_path = plugin_dir.join(format!("{}.wasm", name));
                if wasm_path.exists() {
                    match reg.update(name, &wasm_path, force) {
                        Ok(result) => {
                            println!(" Updated: {} v{}", result.name, result.version);
                        }
                        Err(e) => {
                            eprintln!(" Failed to update '{}': {}", name, e);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn run_search(query: &str) -> anyhow::Result<()> {
    let reg = build_registry(false)?;
    let results = reg.search(query);

    if results.is_empty() {
        println!(" No plugins found matching '{}'", query);
        println!(" Try a different search term.");
        return Ok(());
    }

    println!(" Search results for '{}':", query);
    println!();
    for entry in &results {
        println!(
            " {} v{}",
            entry.name, entry.version
        );
        println!("     {}", entry.description);
        println!(
            "     Author: {} | Downloads: {} | Rating: {:.1}",
            entry.author, entry.downloads, entry.rating
        );
        if !entry.tags.is_empty() {
            println!("     Tags: {}", entry.tags.join(", "));
        }
        println!();
    }
    println!("{} result(s) found.", results.len());
    Ok(())
}

fn run_publish(path: Option<&Path>, _token: Option<&str>) -> anyhow::Result<()> {
    let plugin_path = path.unwrap_or_else(|| Path::new("."));
    let manifest_path = plugin_path.join("klyron-plugin.json");

    if !manifest_path.exists() {
        anyhow::bail!(
            "No klyron-plugin.json found in {}. Run 'klyron plugin create <name>' first.",
            plugin_path.display()
        );
    }

    let content = std::fs::read_to_string(&manifest_path)?;
    let _manifest: klyron_plugin::manifest::PluginManifest = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Invalid klyron-plugin.json: {}", e))?;

    let wasm_path = plugin_path.join("target/wasm32-unknown-unknown/release/*.wasm");
    let wasm_glob = glob::glob(wasm_path.to_str().unwrap_or(""))
        .map_err(|e| anyhow::anyhow!("Glob error: {}", e))?;
    let wasm_files: Vec<_> = wasm_glob.filter_map(|e| e.ok()).collect();

    if wasm_files.is_empty() {
        anyhow::bail!(
            "No WASM file found. Build first with: cargo build --target wasm32-unknown-unknown --release"
        );
    }

    println!(" Publishing plugin...");
    println!("  Manifest: {}", manifest_path.display());
    for f in &wasm_files {
        println!("  WASM: {}", f.display());
    }
    println!("  (Publishing to registry.klyron.dev requires authentication)");
    println!("  Run: klyron login first, then klyron plugin publish --token <token>");
    println!(" Publish aborted: plugin registry upload not available. Use `klyron pack` to create a tarball.");

    Ok(())
}

fn run_create(name: &str, lang: Option<&str>, dir: Option<&Path>) -> anyhow::Result<()> {
    let target_dir = match dir {
        Some(d) => d.join(name),
        None => PathBuf::from("plugins").join(name),
    };

    if target_dir.exists() {
        anyhow::bail!("Directory already exists: {}", target_dir.display());
    }

    std::fs::create_dir_all(&target_dir)?;
    std::fs::create_dir_all(target_dir.join("src"))?;

    let lang = lang.unwrap_or("rust");

    let manifest = serde_json::json!({
        "name": name,
        "version": "0.1.0",
        "description": format!("Klyron plugin: {}", name),
        "authors": ["You"],
        "license": "MIT",
        "klyron_api": KLYRON_API_VERSION,
        "permissions": ["stdio"],
        "hooks": ["onBeforeBuild", "onAfterBuild"],
        "sandbox": {
            "max_memory_bytes": 67108864,
            "max_fuel": 1000000,
            "max_cpu_ms": 5000
        }
    });

    std::fs::write(
        target_dir.join("klyron-plugin.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    match lang {
        "rust" | "rs" => {
            std::fs::write(
                target_dir.join("Cargo.toml"),
                format!(
                    r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "s"
lto = true
"#,
                    name
                ),
            )?;

            std::fs::write(
                target_dir.join("src").join("lib.rs"),
                r#"use std::slice;

#[no_mangle]
pub extern "C" fn on_before_build(ctx_ptr: i32, ctx_len: i32) -> i32 {
    let context = unsafe {
        let data = slice::from_raw_parts(ctx_ptr as *const u8, ctx_len as usize);
        String::from_utf8_lossy(data).to_string()
    };

    let response = format!("klyron-typescript: received {} bytes of context", context.len());
    let bytes = response.as_bytes();
    let ptr = bytes.as_ptr() as i32;
    std::mem::forget(bytes);
    ptr
}

#[no_mangle]
pub extern "C" fn on_after_build(ctx_ptr: i32, ctx_len: i32) -> i32 {
    let _context = unsafe {
        let data = slice::from_raw_parts(ctx_ptr as *const u8, ctx_len as usize);
        String::from_utf8_lossy(data).to_string()
    };

    let response = "klyron-minify: build complete";
    let bytes = response.as_bytes();
    let ptr = bytes.as_ptr() as i32;
    std::mem::forget(bytes);
    ptr
}

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr() as i32;
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: i32, size: i32) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr as *mut u8, size as usize, size as usize);
    }
}
"#,
            )?;
        }
        "ts" | "typescript" => {
            std::fs::write(
                target_dir.join("package.json"),
                format!(
                    r#"{{
  "name": "{}",
  "version": "0.1.0",
  "description": "Klyron plugin",
  "main": "src/index.ts",
  "scripts": {{
    "build": "cargo build --target wasm32-unknown-unknown --release",
    "test": "echo 'No tests'"
  }},
  "dependencies": {{}}
}}
"#,
                    name
                ),
            )?;

            std::fs::write(
                target_dir.join("src").join("index.ts"),
                r#"// Klyron Plugin - TypeScript Template
// Build with: cargo build --target wasm32-unknown-unknown --release

export function onBeforeBuild(context: string): string {
  console.log(`klyron-typescript: received ${context.length} bytes`);
  return `processed ${context.length} bytes`;
}

export function onAfterBuild(context: string): string {
  console.log('klyron-minify: build complete');
  return 'build complete';
}
"#,
            )?;
        }
        other => {
            anyhow::bail!("Unsupported language: {}. Use 'rust' or 'typescript'.", other);
        }
    }

    std::fs::write(
        target_dir.join("README.md"),
        format!(
            "# {}\n\nA Klyron plugin.\n\n## Installation\n\n```bash\nklyron plugin install ./plugins/{}\n```\n\n## Hooks\n\n- `onBeforeBuild` - Called before build\n- `onAfterBuild` - Called after build\n",
            name, name
        ),
    )?;

    println!(" Created plugin scaffold at: {}", target_dir.display());
    println!("  Language: {}", lang);
    println!("  Hooks: onBeforeBuild, onAfterBuild");
    println!();
    println!(" Next steps:");
    println!("   cd {}", target_dir.display());
    if lang == "rust" || lang == "rs" {
        println!("   cargo build --target wasm32-unknown-unknown --release");
    }
    println!("   klyron plugin install ./target/wasm32-unknown-unknown/release/{}.wasm", name);

    Ok(())
}

fn run_toggle(name: &str) -> anyhow::Result<()> {
    let mut reg = build_registry(false)?;
    let info = reg
        .get_info(name)
        .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

    let was_enabled = info.enabled;
    let now_enabled = reg.toggle(name)?;

    if was_enabled {
        println!(" Disabled plugin: {}", name);
    } else {
        println!(" Enabled plugin: {}", name);
    }

    if now_enabled {
        let plugin_dir = reg.plugins_dir().join(name);
        let wasm_path = plugin_dir.join(format!("{}.wasm", name));
        if wasm_path.exists() && !reg.is_loaded(name) {
            match reg.load(wasm_path.to_str().unwrap(), false) {
                Ok(_) => println!(" Loaded plugin: {}", name),
                Err(e) => eprintln!(" Warning: failed to load plugin: {}", e),
            }
        }
    } else if reg.is_loaded(name) {
        let _ = reg.unload(name);
    }

    Ok(())
}
