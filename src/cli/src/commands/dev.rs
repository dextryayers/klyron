use std::path::PathBuf;
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
    if args.hot { println!("  HMR: enabled"); }

    match project_type {
        "node" => {
            let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
            let has_next = dir.join("next.config.mjs").exists() || dir.join("next.config.js").exists();
            if has_next {
                crate::run_cmd("npx", &["next", "dev", "-p", &port.to_string()], &dir)
            } else if has_vite {
                let port_str = port.to_string();
            let mut args_vite = vec!["vite", "--port", &port_str, "--host", &host];
                crate::run_cmd("npx", &args_vite, &dir)
            } else {
                crate::run_cmd("npx", &["vite", "--port", &port.to_string(), "--host", &host], &dir)
            }
        }
        "laravel" => {
            crate::run_cmd("php", &["artisan", "serve", "--port", &port.to_string(), "--host", &host], &dir)
        }
        "python" => {
            if dir.join("manage.py").exists() {
                crate::run_cmd("python3", &["manage.py", "runserver", &format!("{}:{}", host, port)], &dir)
            } else {
                anyhow::bail!("No dev server configuration found")
            }
        }
        "rust" => {
            crate::run_cmd("cargo", &["run"], &dir)
        }
        "go" => {
            crate::run_cmd("go", &["run", "."], &dir)
        }
        _ => anyhow::bail!("Dev server not configured for {project_type}"),
    }
}
