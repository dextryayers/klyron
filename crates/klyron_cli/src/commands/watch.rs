use std::path::{Path, PathBuf};
use std::process::{Child, Command as StdCommand};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use clap::Args;

#[derive(Args)]
pub struct WatchArgs {
    pub script: String,
    #[arg(long)]
    pub dir: Option<PathBuf>,
    #[arg(long)]
    pub ignore: Vec<String>,
}

pub fn run_watch(args: WatchArgs) -> anyhow::Result<()> {
    let dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    let script = args.script.clone();

    crate::load_dotenv(&dir);

    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    let runner = detect_script_runner(&script);
    let parts = shell_split(&script);

    eprintln!(
        "{} klyron watch — watching {} for changes",
        crate::Color::CYAN.paint("\u{1F50D}"),
        dir.display()
    );
    eprintln!("  {} Running: {}", crate::Color::GREEN.paint("\u{25B6}"), script);
    eprintln!("  {} Press Ctrl+C to stop", crate::Color::DIM.paint("i"));

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| anyhow::anyhow!("Failed to set Ctrl+C handler: {e}"))?;

    let watch_dir = dir.clone();
    let running_watch = running.clone();

    let watcher_handle = std::thread::spawn(move || {
        let mut builder = klyron_watcher::WatcherBuilder::new()
            .add_path(watch_dir.to_str().unwrap_or("."))
            .recursive(true)
            .debounce(500);

        for pat in &[
            "node_modules/*", ".git/*", "target/*", ".klyron/*",
            "*.lock", ".DS_Store", "*.swp", "*.swo",
            ".vscode/*", ".idea/*", "dist/*", "build/*",
        ] {
            builder = builder.ignore(pat);
        }

        for pat in &args.ignore {
            builder = builder.ignore(pat);
        }

        if let Ok(watcher) = builder.build() {
            let changed = Arc::new(AtomicBool::new(false));
            let c = changed.clone();
            let _ = watcher.start(move |_event| {
                c.store(true, Ordering::SeqCst);
            });

            while running_watch.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    });

    let running_child = running.clone();
    while running.load(Ordering::SeqCst) {
        let mut child = run_script_detached(&runner, &parts, &dir)?;

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        eprintln!(
                            "{} Script exited with code {}",
                            crate::Color::YELLOW.paint("\u{26A0}"),
                            status
                        );
                    }
                    break;
                }
                Ok(None) => {
                    if !running.load(Ordering::SeqCst) {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Ok(());
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }
                Err(e) => anyhow::bail!("Failed to wait for script: {e}"),
            }
        }

        if running.load(Ordering::SeqCst) {
            eprintln!(
                "\n{} {}",
                crate::Color::YELLOW.paint("\u{1F504}"),
                crate::Color::BOLD.paint("File change detected. Restarting...")
            );
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    Ok(())
}

fn run_script_detached(program: &str, args: &[String], dir: &Path) -> std::io::Result<Child> {
    StdCommand::new(program)
        .args(args)
        .current_dir(dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
}

fn detect_script_runner(script: &str) -> String {
    if script.ends_with(".js") || script.ends_with(".mjs") {
        "node".to_string()
    } else if script.ends_with(".ts") || script.ends_with(".tsx") {
        if cfg!(target_os = "windows") {
            "npx".to_string()
        } else {
            if StdCommand::new("tsx").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
                "tsx".to_string()
            } else if StdCommand::new("bun").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
                "bun".to_string()
            } else {
                "npx".to_string()
            }
        }
    } else if script.ends_with(".py") {
        if StdCommand::new("python3").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
            "python3".to_string()
        } else {
            "python".to_string()
        }
    } else if script.ends_with(".rs") {
        "cargo".to_string()
    } else if script.ends_with(".go") {
        "go".to_string()
    } else if script.ends_with(".rb") {
        "ruby".to_string()
    } else if script.ends_with(".php") {
        "php".to_string()
    } else if script.ends_with(".zig") {
        "zig".to_string()
    } else if script.ends_with(".sh") {
        "sh".to_string()
    } else {
        "sh".to_string()
    }
}

fn shell_split(script: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';

    for c in script.chars() {
        if in_quote {
            if c == quote_char {
                in_quote = false;
            } else {
                current.push(c);
            }
        } else {
            match c {
                '\'' | '"' => {
                    in_quote = true;
                    quote_char = c;
                }
                ' ' | '\t' => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(c),
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}
