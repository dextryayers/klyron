use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use clap::Args;
use tokio::sync::broadcast;

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

    crate::load_dotenv(&dir);

    if let Some(tsconfig) = crate::detect_tsconfig(&dir) {
        let opts = crate::apply_tsconfig_compiler_options(&tsconfig);
        if !opts.is_empty() {
            crate::log_info(format!("Detected tsconfig.json options: {}", opts.join(" ")));
        }
    }

    if !dir.join("klyron.json").exists() {
        let _ = crate::auto_create_klyron_config(&dir);
    }

    let project_type = crate::detect_project_type(&dir);
    let hmr_enabled = args.hot || args.watch;

    if hmr_enabled {
        crate::log_info(format!(
            "{} Klyron Dev Server — {} on http://{}:{} [HMR]",
            crate::Color::GREEN.paint("\u{25C9}"),
            project_type, host, port
        ));
    } else {
        crate::log_info(format!(
            "{} Klyron Dev Server — {} on http://{}:{}",
            crate::Color::GREEN.paint("\u{25C9}"),
            project_type, host, port
        ));
    }

    match project_type {
        "node" => {
            let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
            let has_next = dir.join("next.config.mjs").exists() || dir.join("next.config.js").exists();
            if has_next {
                if hmr_enabled {
                    watch_dev("npx", &["next", "dev", "-p", &port.to_string()], &dir)
                } else {
                    crate::run_cmd("npx", &["next", "dev", "-p", &port.to_string()], &dir)
                }
            } else if has_vite {
                let mut args_vite = vec!["vite", "--port", &port.to_string(), "--host", &host];
                if hmr_enabled { args_vite.push("--hmr"); }
                if hmr_enabled {
                    watch_dev("npx", &args_vite, &dir)
                } else {
                    crate::run_cmd("npx", &args_vite, &dir)
                }
            } else if hmr_enabled {
                run_klyron_hmr_server(&dir, port, host)
            } else {
                run_klyron_static_server(&dir, port, host)
            }
        }
        "laravel" => {
            if hmr_enabled {
                watch_dev("php", &["artisan", "serve", "--port", &port.to_string(), "--host", &host], &dir)
            } else {
                crate::run_cmd("php", &["artisan", "serve", "--port", &port.to_string(), "--host", &host], &dir)
            }
        }
        "python" => {
            if dir.join("manage.py").exists() {
                if hmr_enabled {
                    watch_dev("python3", &["manage.py", "runserver", &format!("{}:{}", host, port)], &dir)
                } else {
                    crate::run_cmd("python3", &["manage.py", "runserver", &format!("{}:{}", host, port)], &dir)
                }
            } else {
                anyhow::bail!("No dev server configuration found")
            }
        }
        "rust" => {
            if hmr_enabled {
                watch_dev("cargo", &["run"], &dir)
            } else {
                crate::run_cmd("cargo", &["run"], &dir)
            }
        }
        "go" => {
            if hmr_enabled {
                watch_dev("go", &["run", "."], &dir)
            } else {
                crate::run_cmd("go", &["run", "."], &dir)
            }
        }
        _ => {
            if hmr_enabled {
                run_klyron_hmr_server(&dir, port, host)
            } else {
                run_klyron_static_server(&dir, port, host)
            }
        }
    }
}

fn run_klyron_static_server(dir: &Path, port: u16, host: String) -> anyhow::Result<()> {
    crate::start_dev_server(&host, port, dir)
}

fn run_klyron_hmr_server(dir: &Path, port: u16, host: String) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let dir = dir.to_path_buf();
    let shutdown = Arc::new(AtomicBool::new(false));

    rt.block_on(async {
        let (tx, _rx) = broadcast::channel::<String>(100);
        let tx_clone = tx.clone();
        let dir_clone = dir.clone();
        let shutdown_clone = shutdown.clone();

        let watch_handle = std::thread::spawn(move || {
            let watch_dir = dir_clone.clone();
            let watcher = match klyron_watcher::WatcherBuilder::new()
                .add_path(watch_dir.to_str().unwrap_or("."))
                .debounce(300)
                .build()
            {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("Failed to start watcher: {e}");
                    return;
                }
            };

            let _ = watcher.start_hmr(move |update| {
                for path in &update.changed {
                    let rel = path.strip_prefix(&dir_clone).unwrap_or(path);
                    let msg = format!("changed:{}", rel.display());
                    let _ = tx_clone.send(msg);
                    crate::log_info(format!(
                        "{} {} {}",
                        crate::Color::YELLOW.paint("\u{26A1}"),
                        crate::Color::BOLD.paint("[HMR]"),
                        crate::Color::CYAN.paint(format!("{} changed, recompiling...", rel.display()))
                    ));
                }
                for path in &update.added {
                    let rel = path.strip_prefix(&dir_clone).unwrap_or(path);
                    let msg = format!("added:{}", rel.display());
                    let _ = tx_clone.send(msg);
                    crate::log_info(format!(
                        "{} {} {}",
                        crate::Color::GREEN.paint("\u{2795}"),
                        crate::Color::BOLD.paint("[HMR]"),
                        crate::Color::CYAN.paint(format!("{} added", rel.display()))
                    ));
                }
                for path in &update.removed {
                    let rel = path.strip_prefix(&dir_clone).unwrap_or(path);
                    let msg = format!("removed:{}", rel.display());
                    let _ = tx_clone.send(msg);
                    crate::log_info(format!(
                        "{} {} {}",
                        crate::Color::RED.paint("\u{2796}"),
                        crate::Color::BOLD.paint("[HMR]"),
                        crate::Color::CYAN.paint(format!("{} removed", rel.display()))
                    ));
                }
            });
        });

        let app = axum::Router::new()
            .route("/__klyron_hmr", axum::routing::get(hmr_sse_handler))
            .with_state(tx)
            .fallback_service(tower_http::services::ServeDir::new(&dir)
                .append_index_html_on_directories(true)
                .not_found_service(tower_http::services::ServeDir::new(&dir)
                    .precompressed_br()
                    .precompressed_gzip()
                    .append_index_html_on_directories(true))
            );

        let addr = format!("{host}:{port}");
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                anyhow::bail!("Cannot bind {addr}: {e}");
            }
        };

        crate::log_info(format!(
            "{} {} {} {}",
            crate::Color::GREEN.paint("\u{25C9}"),
            crate::Color::BOLD.paint("Klyron HMR Dev Server"),
            crate::Color::CYAN.paint(format!("http://{addr}")),
            crate::Color::DIM.paint(format!("(watcher: {} dirs)", 1))
        ));

        crate::log_info(format!(
            "{} {}",
            crate::Color::BLUE.paint("\u{2139}"),
            crate::Color::DIM.paint("SSE endpoint: /__klyron_hmr")
        ));

        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {e}"))?;

        shutdown_clone.store(true, Ordering::SeqCst);
        drop(watch_handle);
        Ok(())
    })
}

async fn hmr_sse_handler(
    axum::extract::State(tx): axum::extract::State<broadcast::Sender<String>>,
) -> axum::response::Sse<impl tokio_stream::Stream<Item = Result<String, std::convert::Infallible>>> {
    let mut rx = tx.subscribe();

    let initial = vec![
        Ok::<_, std::convert::Infallible>(format!("data: {}\n\n", serde_json::json!({"type": "connected", "hmr": true}))),
    ];

    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(|result| async move {
            match result {
                Ok(msg) => Some(Ok::<_, std::convert::Infallible>(format!("data: {}\n\n", serde_json::json!({"type": "update", "body": msg})))),
                Err(_) => None,
            }
        });

    let full_stream = tokio_stream::iter(initial).chain(stream);

    axum::response::Sse::new(full_stream)
        .keep_alive(axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"))
}

fn watch_dev(program: &str, args: &[&str], dir: &Path) -> anyhow::Result<()> {
    let dir = dir.to_path_buf();
    let (tx, rx) = std::sync::mpsc::channel();
    let watch_dir = dir.clone();

    let watcher_dir = watch_dir.clone();
    std::thread::spawn(move || {
        let watcher = klyron_watcher::WatcherBuilder::new()
            .add_path(watcher_dir.to_str().unwrap_or("."))
            .debounce(300)
            .build();
        if let Ok(watcher) = watcher {
            let _ = watcher.start(move |_event| {
                let _ = tx.send(());
            });
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
                        crate::log_info(format!("\n{} File change detected. Restarting...", crate::Color::YELLOW.paint("\u{1F504}")));
                        let _ = child.kill();
                        let _ = child.wait();
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => anyhow::bail!("Failed to wait for dev server: {e}"),
            }
        }
    }
}
