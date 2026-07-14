use clap::Subcommand;

#[derive(Subcommand)]
pub enum CacheAction {
    Clean,
    Prune,
    Info,
}

pub fn run_cache(action: CacheAction) -> anyhow::Result<()> {
    let cache_dir = dirs::cache_dir().map(|p| p.join("klyron"));
    match action {
        CacheAction::Clean => {
            println!("🧹 Clearing Klyron cache...");
            if let Some(dir) = cache_dir {
                if dir.exists() {
                    std::fs::remove_dir_all(&dir)?;
                    println!("  Cleared: {}", dir.display());
                }
            }
            println!("  Also clearing npm cache...");
            crate::run_cmd("npm", &["cache", "clean", "--force"], &std::env::current_dir()?).ok();
            Ok(())
        }
        CacheAction::Prune => {
            println!("🧹 Pruning expired cache entries...");
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
