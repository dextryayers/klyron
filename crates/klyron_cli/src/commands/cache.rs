use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum CacheAction {
    Clean {
        #[arg(long)]
        all: bool,
    },
    Ls {
        #[arg(long)]
        json: bool,
    },
    Info,
}

fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("klyron")
}

fn ensure_cache_dir() {
    let dir = cache_dir();
    std::fs::create_dir_all(&dir).ok();
}

pub fn run_cache(action: CacheAction) -> anyhow::Result<()> {
    ensure_cache_dir();

    match action {
        CacheAction::Clean { all } => run_clean(all),
        CacheAction::Ls { json } => run_ls(json),
        CacheAction::Info => run_info(),
    }
}

fn run_clean(all: bool) -> anyhow::Result<()> {
    let dir = cache_dir();
    if !dir.exists() {
        eprintln!("  Cache directory does not exist: {}", dir.display());
        return Ok(());
    }

    if all {
        eprintln!("  Clearing entire cache...");
        let entry_count = count_entries(&dir);
        std::fs::remove_dir_all(&dir)?;
        std::fs::create_dir_all(&dir)?;
        eprintln!(
            "  {} Removed {entry_count} entries from {}",
            crate::Color::GREEN.paint("\u{2705}"),
            dir.display(),
        );
    } else {
        eprintln!("  Removing stale/expired cache entries...");
        let ttl = std::time::Duration::from_secs(24 * 60 * 60);
        let now = std::time::SystemTime::now();
        let mut pruned = 0u64;

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(meta) = std::fs::metadata(&path) {
                    if let Ok(modified) = meta.modified() {
                        if now
                            .duration_since(modified)
                            .map(|d| d > ttl)
                            .unwrap_or(false)
                        {
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

        if pruned > 0 {
            eprintln!(
                "  {} Pruned {pruned} expired entries",
                crate::Color::GREEN.paint("\u{2705}"),
            );
        } else {
            eprintln!("  No expired entries found");
        }
        eprintln!("  Use --all to clear the entire cache");
    }

    Ok(())
}

fn run_ls(json: bool) -> anyhow::Result<()> {
    let dir = cache_dir();
    if !dir.exists() {
        if json {
            println!("[]");
        } else {
            eprintln!("  Cache directory does not exist: {}", dir.display());
        }
        return Ok(());
    }

    let mut entries: Vec<serde_json::Value> = Vec::new();

    if let Ok(read_dir) = std::fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let is_dir = path.is_dir();
            let size = std::fs::metadata(&path)
                .map(|m| m.len())
                .unwrap_or(0);
            let modified = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            if json {
                entries.push(serde_json::json!({
                    "name": name,
                    "type": if is_dir { "directory" } else { "file" },
                    "size": size,
                    "modified": modified,
                }));
            } else {
                let type_char = if is_dir { 'd' } else { 'f' };
                let size_str = if size > 1024 * 1024 {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                } else if size > 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{size} B")
                };
                eprintln!(
                    "  {type_char} {:<30} {:>8}",
                    name,
                    size_str,
                );
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        let total = entries.len();
        eprintln!("  {} entries in {}", total, dir.display());
    }

    Ok(())
}

fn run_info() -> anyhow::Result<()> {
    let dir = cache_dir();

    eprintln!(
        "  {} Cache Information",
        crate::Color::BOLD.paint("\u{1F4CA}"),
    );
    eprintln!("    Location: {}", dir.display());

    if !dir.exists() {
        eprintln!("    Status: not initialized");
        return Ok(());
    }

    let total_size = get_dir_size(&dir);
    let entry_count = count_entries(&dir);

    let size_str = if total_size > 1024 * 1024 * 1024 {
        format!("{:.2} GB", total_size as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if total_size > 1024 * 1024 {
        format!("{:.2} MB", total_size as f64 / (1024.0 * 1024.0))
    } else if total_size > 1024 {
        format!("{:.2} KB", total_size as f64 / 1024.0)
    } else {
        format!("{total_size} B")
    };

    eprintln!("    Total size: {size_str}");
    eprintln!("    Entry count: {entry_count}");

    Ok(())
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

fn count_entries(path: &std::path::Path) -> u64 {
    let mut count = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            count += 1;
            let path = entry.path();
            if path.is_dir() {
                count += count_entries(&path);
            }
        }
    }
    count
}
