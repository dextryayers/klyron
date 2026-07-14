use clap::Subcommand;

#[derive(Subcommand)]
pub enum CacheAction {
    Clean,
    Prune,
    Info,
}

pub fn run_cache(action: CacheAction) -> anyhow::Result<()> {
    let cache_dir = dirs::cache_dir().map(|p| p.join("klyron"));
    if let Some(ref dir) = cache_dir {
        std::fs::create_dir_all(dir).ok();
    }
    match action {
        CacheAction::Clean => {
            println!("🧹 Clearing Klyron cache...");
            if let Some(ref dir) = cache_dir {
                if dir.exists() {
                    std::fs::remove_dir_all(dir)?;
                    std::fs::create_dir_all(dir)?;
                    println!("  Cleared: {}", dir.display());
                }
            }
            println!("  Also clearing npm cache...");
            crate::run_cmd("npm", &["cache", "clean", "--force"], &std::env::current_dir()?).ok();
            Ok(())
        }
        CacheAction::Prune => {
            println!("🧹 Pruning expired cache entries...");
            let ttl = std::time::Duration::from_secs(24 * 60 * 60);
            let now = std::time::SystemTime::now();
            let mut pruned = 0u64;
            if let Some(dir) = &cache_dir {
                if dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Ok(meta) = std::fs::metadata(&path) {
                            if let Ok(modified) = meta.modified() {
                                if now.duration_since(modified).map(|d| d > ttl).unwrap_or(false) {
                                    if path.is_dir() {
                                        std::fs::remove_dir_all(&path).ok();
                                    } else {
                                        std::fs::remove_file(&path).ok();
                                    }
                                    pruned += 1;
                                }
                            }
                        }
                    }
                    }
                }
            }
            println!("  Pruned {pruned} expired entries");
            Ok(())
        }
        CacheAction::Info => {
            println!("📊 Cache Info");
            if let Some(dir) = cache_dir {
                if dir.exists() {
                    let size = get_dir_size(&dir);
                    println!("  Location: {}", dir.display());
                    println!("  Size: {} MB", size / 1024 / 1024);
                } else {
                    println!("  Cache directory not found");
                }
            }
            Ok(())
        }
    }
}

fn get_dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += get_dir_size(&path);
            } else if let Ok(meta) = std::fs::metadata(&path) {
                total += meta.len();
            }
        }
    }
    total
}
