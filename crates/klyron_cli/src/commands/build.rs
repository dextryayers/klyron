use std::path::PathBuf;
use clap::Args;

#[derive(Args)]
pub struct BuildArgs {
    #[arg(default_value = ".")]
    pub dir: PathBuf,
    #[arg(long)]
    pub release: bool,
    #[arg(long)]
    pub minify: bool,
    #[arg(long)]
    pub sourcemap: bool,
    #[arg(long)]
    pub target: Option<String>,
}

pub fn run_build(args: BuildArgs) -> anyhow::Result<()> {
    let dir = args.dir;
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    let project_type = crate::detect_project_type(&dir);
    println!("Building {} project in {}", project_type, dir.display());
    if args.minify { println!("  Minify: enabled"); }
    if args.sourcemap { println!("  Sourcemap: enabled"); }
    if let Some(ref t) = args.target { println!("  Target: {}", t); }

    match project_type {
        "node" => {
            let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
            let has_next = dir.join("next.config.mjs").exists() || dir.join("next.config.js").exists();
            if has_next {
                crate::run_cmd("npx", &["next", "build"], &dir)
            } else if has_vite {
                let mut vite_args = vec!["vite".to_string(), "build".to_string()];
                if args.minify { vite_args.push("--minify".into()); }
                if args.sourcemap { vite_args.push("--sourcemap".into()); }
                if let Some(ref t) = args.target { vite_args.push("--target".into()); vite_args.push(t.clone()); }
                crate::run_cmd_str("npx", &vite_args, &dir)
            } else {
                anyhow::bail!("No build configuration found. Use a framework like Next.js or Vite.")
            }
        }
        "laravel" => {
            crate::run_cmd("php", &["artisan", "build"], &dir)
        }
        "rust" => {
            if args.release {
                crate::run_cmd("cargo", &["build", "--release"], &dir)
            } else {
                crate::run_cmd("cargo", &["build"], &dir)
            }
        }
        "go" => {
            crate::run_cmd("go", &["build", "-o", "dist/"], &dir)
        }
        _ => anyhow::bail!("Build not configured for {project_type}"),
    }
}
