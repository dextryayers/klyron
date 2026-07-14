use clap::Args;

#[derive(Args)]
pub struct CheckArgs {
    pub target: Option<String>,
}

pub fn run_check(args: CheckArgs) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match args.target.as_deref() {
        None | Some("all") | Some("project") => {
            check_types(&dir)?;
            Ok(())
        }
        Some("types") => check_types(&dir),
        Some(t) => anyhow::bail!("Unknown check target: {t}. Use: types, project"),
    }
}

fn check_types(dir: &std::path::Path) -> anyhow::Result<()> {
    let project = crate::detect_project_type(dir);
    match project {
        "node" => {
            let tsconfig = dir.join("tsconfig.json");
            if tsconfig.exists() {
                crate::run_cmd("npx", &["tsc", "--noEmit"], dir)
            } else {
                println!("No tsconfig.json found. Skipping type check.");
                Ok(())
            }
        }
        "rust" => crate::run_cmd("cargo", &["check"], dir),
        _ => {
            println!("Type checking not configured for {project}");
            Ok(())
        }
    }
}
