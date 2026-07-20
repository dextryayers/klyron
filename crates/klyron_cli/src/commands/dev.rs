use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use axum::response::sse::{Event, KeepAlive, Sse};
use clap::Args;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

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
    #[arg(long, default_value_t = false)]
    pub no_hmr_inject: bool,
}

pub fn run_dev(args: DevArgs) -> anyhow::Result<()> {
    crate::anim::cmd_header("dev", "Starting development server");
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
        let _ = crate::generate_klyron_config(&dir, true);
    }

    let project_type = crate::detect_project_type(&dir);
    let hmr_enabled = (args.hot || args.watch) && !args.no_hmr_inject;

    if hmr_enabled {
        crate::log_info(format!(
            "{} {} {} [HMR]",
            crate::Color::GREEN.paint("\u{25C9}"),
            format!("Klyron Dev Server \u{2014} {project_type} on http://{host}:{port}"),
            crate::Color::DIM.paint("(hot module replacement)")
        ));
    } else {
        crate::log_info(format!(
            "{} {}",
            crate::Color::GREEN.paint("\u{25C9}"),
            format!("Klyron Dev Server \u{2014} {project_type} on http://{host}:{port}")
        ));
    }

    crate::log_info(format!(
        "{} {}",
        crate::Color::DIM.paint("\u{2502}"),
        crate::Color::CYAN.paint(format!("serving: {}", dir.display()))
    ));

    match project_type {
        "node" => {
            let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists() || dir.join("vite.config.mjs").exists();
            let has_next = dir.join("next.config.ts").exists() || dir.join("next.config.mjs").exists() || dir.join("next.config.js").exists();
            let local_bin = |name: &str| -> String {
                let path = dir.join("node_modules").join(".bin").join(name);
                if path.exists() { path.to_string_lossy().to_string() } else { name.to_string() }
            };
            if has_next {
                let next_bin = local_bin("next");
                if hmr_enabled {
                    watch_dev(&next_bin, &["dev", "-p", &port.to_string()], &dir)
                } else {
                    crate::run_cmd(&next_bin, &["dev", "-p", &port.to_string()], &dir)
                }
            } else if has_vite {
                let port_str = port.to_string();
                let host_str = host.clone();
                let vite_bin = local_bin("vite");
                let args_vite: Vec<&str> = {
                    let mut v = vec!["--port", &port_str, "--host", &host_str];
                    if hmr_enabled { v.push("--hmr"); }
                    v
                };
                if hmr_enabled {
                    watch_dev(&vite_bin, &args_vite, &dir)
                } else {
                    crate::run_cmd(&vite_bin, &args_vite, &dir)
                }
            } else if dir.join("package.json").exists() {
                // Fallback: try npm run dev if package.json has a dev script
                let pkg = std::fs::read_to_string(dir.join("package.json")).unwrap_or_default();
                if pkg.contains("\"dev\"") || pkg.contains("'dev'") {
                    crate::log_info("Detected npm dev script, running npm run dev ...");
                    crate::run_cmd("npm", &["run", "dev"], &dir)
                } else if hmr_enabled {
                    run_klyron_hmr_server(&dir, port, host)
                } else {
                    run_klyron_static_server(&dir, port, host)
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
    let addr = format!("{host}:{port}");
    crate::anim::success_banner(&format!("Dev server ready at http://{addr}"));
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let service = tower_http::services::ServeDir::new(dir)
            .append_index_html_on_directories(true);
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| anyhow::anyhow!("Cannot bind {addr}: {e}"))?;
        crate::log_info(format!("Listening on http://{addr}"));
        axum::serve(listener, axum::routing::any_service(service))
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {e}"))?;
        Ok(())
    })
}

fn run_klyron_hmr_server(dir: &Path, port: u16, host: String) -> anyhow::Result<()> {
    let addr = format!("{host}:{port}");
    crate::anim::success_banner(&format!("Dev server ready at http://{addr}"));
    let rt = tokio::runtime::Runtime::new()?;
    let dir = dir.to_path_buf();
    let shutdown = Arc::new(AtomicBool::new(false));
    let connected_clients = Arc::new(AtomicUsize::new(0));

    rt.block_on(async {
        let (tx, _rx) = broadcast::channel::<String>(200);
        let tx_clone = tx.clone();
        let dir_clone = dir.clone();

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
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0);
                let time_str = format_time(now);

                for path in &update.changed {
                    let rel = path.strip_prefix(&dir_clone).unwrap_or(path);
                    let msg = format!("changed:{}", rel.display());
                    let _ = tx_clone.send(msg);
                    crate::log_info(format!(
                        "{} {} {} {}",
                        crate::Color::DIM.paint(format!("[{time_str}]")),
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
                        "{} {} {} {}",
                        crate::Color::DIM.paint(format!("[{time_str}]")),
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
                        "{} {} {} {}",
                        crate::Color::DIM.paint(format!("[{time_str}]")),
                        crate::Color::RED.paint("\u{2796}"),
                        crate::Color::BOLD.paint("[HMR]"),
                        crate::Color::CYAN.paint(format!("{} removed", rel.display()))
                    ));
                }
            });
        });

        let app = axum::Router::new()
            .route("/__klyron_hmr", axum::routing::get(hmr_sse_handler))
            .route("/__klyron_hmr.js", axum::routing::get(hmr_client_js))
            .with_state(HmrState {
                tx: tx.clone(),
                clients: connected_clients.clone(),
            })
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
            crate::Color::DIM.paint("(watcher active)")
        ));

        crate::log_info(format!(
            "{} {}",
            crate::Color::BLUE.paint("\u{2139}"),
            crate::Color::DIM.paint("SSE: /__klyron_hmr | Client: /__klyron_hmr.js")
        ));

        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {e}"))?;

        shutdown.store(true, Ordering::SeqCst);
        drop(watch_handle);
        Ok(())
    })
}

#[derive(Clone)]
struct HmrState {
    tx: broadcast::Sender<String>,
    clients: Arc<AtomicUsize>,
}

async fn hmr_sse_handler(
    axum::extract::State(state): axum::extract::State<HmrState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    state.clients.fetch_add(1, Ordering::SeqCst);
    let clients = state.clients.load(Ordering::SeqCst);
    crate::log_info(format!(
        "{} {} {} {}",
        crate::Color::DIM.paint("[SSE]"),
        crate::Color::GREEN.paint("\u{2795}"),
        "Client connected",
        crate::Color::DIM.paint(format!("({clients} total)"))
    ));

    let rx = state.tx.subscribe();

    let init_event = Ok::<Event, Infallible>(
        Event::default().data(serde_json::json!({
            "type": "connected",
            "hmr": true,
            "clientCount": clients
        }).to_string())
    );

    let stream = BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(msg) => {
                let parts: Vec<&str> = msg.splitn(2, ':').collect();
                let (event_type, path) = if parts.len() == 2 {
                    (parts[0], parts[1])
                } else {
                    ("update", msg.as_str())
                };
                let event = Event::default().data(serde_json::json!({
                    "type": event_type,
                    "path": path,
                    "timestamp": SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs_f64())
                        .unwrap_or(0.0)
                }).to_string());
                Some(Ok::<Event, Infallible>(event))
            }
            Err(_) => None,
        }
    });

    let full_stream = tokio_stream::once(init_event).chain(stream);

    Sse::new(full_stream)
        .keep_alive(KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"))
}

async fn hmr_client_js() -> impl axum::response::IntoResponse {
    let hmr_client = r#"(function() {
    'use strict';
    const evtSource = new EventSource('/__klyron_hmr');
    evtSource.onmessage = function(evt) {
        try {
            const data = JSON.parse(evt.data);
            if (data.type === 'connected') {
                console.log('[Klyron HMR] Connected', data.clientCount ? '(' + data.clientCount + ' clients)' : '');
                return;
            }
            if (data.type === 'changed' || data.type === 'update') {
                const path = data.path || '';
                console.log('[Klyron HMR] Update:', path);
                if (path.endsWith('.css')) {
                    const links = document.querySelectorAll('link[rel="stylesheet"]');
                    for (const link of links) {
                        const href = link.getAttribute('href');
                        if (href && href.includes(path.split('/').pop())) {
                            link.href = href.split('?')[0] + '?t=' + Date.now();
                            return;
                        }
                    }
                }
                setTimeout(function() { window.location.reload(); }, 100);
            }
        } catch(e) { console.warn('[Klyron HMR] Parse error:', e); }
    };
    evtSource.onerror = function() {
        console.warn('[Klyron HMR] Disconnected, retrying...');
    };
})();"#;
    axum::response::Response::builder()
        .header("Content-Type", "application/javascript; charset=utf-8")
        .body(axum::body::Body::from(hmr_client))
        .unwrap()
}

fn format_time(unix_secs: f64) -> String {
    let total_secs = unix_secs as u64;
    let secs_of_day = total_secs % 86400;
    let hours = secs_of_day / 3600;
    let minutes = (secs_of_day % 3600) / 60;
    let seconds = secs_of_day % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
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
            let _ = watcher.start_hmr(move |_update| {
                let _ = tx.send(());
            });
        }
    });

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    }).ok();

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        let mut child = match StdCommand::new(program)
            .args(args)
            .current_dir(&dir)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to start dev server: {e}");
                break;
            }
        };

        loop {
            if !running.load(std::sync::atomic::Ordering::SeqCst) {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(());
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        eprintln!("Dev server exited with code {}", status);
                    }
                    break;
                }
                Ok(None) => {
                    if rx.try_recv().is_ok() {
                        crate::log_info(format!("\n{} File change detected. Restarting...",
                            crate::Color::YELLOW.paint("\u{1F504}")));
                        let _ = child.kill();
                        let _ = child.wait();
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    eprintln!("Failed to wait: {e}");
                    break;
                }
            }
        }
    }
    Ok(())
}
