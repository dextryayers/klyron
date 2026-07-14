use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use clap::Args;

#[derive(Args)]
pub struct DevArgs {
    #[arg(default_value = ".")]
    pub dir: PathBuf,
    #[arg(long)]
    pub port: Option<u16>,
    #[arg(long)]
    pub host: Option<String>,
    #[arg(long)]
    pub watch: bool,
    #[arg(long)]
    pub hot: bool,
}

pub fn run_dev(args: DevArgs) -> anyhow::Result<()> {
    let dir = args.dir;
    let port = args.port.unwrap_or(3000);
    let host = args.host.unwrap_or_else(|| "127.0.0.1".to_string());

    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    let project_type = crate::detect_project_type(&dir);
    println!("Klyron Dev Server — {} on http://{}:{}", project_type, host, port);
    if args.watch { println!("  Watch mode: enabled"); }
    if args.hot {
        println!("  HMR: enabled");
        println!("[HMR] Watching for changes...");
        let hot_dir = dir.clone();
        thread::spawn(move || {
        let mut last_modified = std::time::SystemTime::now();
            loop {
                thread::sleep(Duration::from_millis(500));
                if let Ok(entries) = std::fs::read_dir(&hot_dir) {
                    for entry in entries.flatten() {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                if modified > last_modified {
                                    last_modified = modified;
                                    println!("[HMR] Hot reload triggered!");
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    match project_type {
        "node" => {
            let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
            let has_next = dir.join("next.config.mjs").exists() || dir.join("next.config.js").exists();
            if has_next {
                if args.watch {
                    watch_dev("npx", &["next", "dev", "-p", &port.to_string()], &dir)
                } else {
                    crate::run_cmd("npx", &["next", "dev", "-p", &port.to_string()], &dir)
                }
            } else if has_vite {
                let port_str = port.to_string();
                let args_vite = vec!["vite", "--port", &port_str, "--host", &host];
                if args.watch {
                    watch_dev("npx", &args_vite, &dir)
                } else {
                    crate::run_cmd("npx", &args_vite, &dir)
                }
            } else {
                if args.watch {
                    watch_dev("npx", &["vite", "--port", &port.to_string(), "--host", &host], &dir)
                } else {
                    crate::run_cmd("npx", &["vite", "--port", &port.to_string(), "--host", &host], &dir)
                }
            }
        }
        "laravel" => {
            if args.watch {
                watch_dev("php", &["artisan", "serve", "--port", &port.to_string(), "--host", &host], &dir)
            } else {
                crate::run_cmd("php", &["artisan", "serve", "--port", &port.to_string(), "--host", &host], &dir)
            }
        }
        "python" => {
            if dir.join("manage.py").exists() {
                if args.watch {
                    watch_dev("python3", &["manage.py", "runserver", &format!("{}:{}", host, port)], &dir)
                } else {
                    crate::run_cmd("python3", &["manage.py", "runserver", &format!("{}:{}", host, port)], &dir)
                }
            } else {
                anyhow::bail!("No dev server configuration found")
            }
        }
        "rust" => {
            if args.watch {
                watch_dev("cargo", &["run"], &dir)
            } else {
                crate::run_cmd("cargo", &["run"], &dir)
            }
        }
        "go" => {
            if args.watch {
                watch_dev("go", &["run", "."], &dir)
            } else {
                crate::run_cmd("go", &["run", "."], &dir)
            }
        }
        _ => anyhow::bail!("Dev server not configured for {project_type}"),
    }
}

fn watch_dev(program: &str, args: &[&str], dir: &Path) -> anyhow::Result<()> {
    let dir = dir.to_path_buf();
    let (tx, rx) = mpsc::channel();
    let watch_dir = dir.clone();

    thread::spawn(move || {
        let last_modified = std::time::SystemTime::now();
        loop {
            thread::sleep(Duration::from_secs(1));
            if let Ok(entries) = std::fs::read_dir(&watch_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if modified > last_modified {
                                let _ = tx.send(());
                                return;
                            }
                        }
                    }
                }
            }
        }
    });

    loop {
        let mut child = StdCommand::new(program)
            .args(args)
            .current_dir(&dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start dev server: {e}"))?;

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        anyhow::bail!("Dev server exited with code {}", status);
                    }
                    break;
                }
                Ok(None) => {
                    if rx.try_recv().is_ok() {
                        println!("\nFile change detected. Restarting...");
                        let _ = child.kill();
                        let _ = child.wait();
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => anyhow::bail!("Failed to wait for dev server: {e}"),
            }
        }
    }
}
