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

    // Check for Vue project
    if dir.join("vue.config.ts").exists() || dir.join("vue.config.js").exists()
        || dir.join("vite.config.ts").exists()
    {
        if dir.join("tsconfig.json").exists() {
            if dir.join("node_modules/.bin/vue-tsc").exists() {
                return crate::run_cmd("npx", &["vue-tsc", "--noEmit"], dir);
            }
            return crate::run_cmd("npx", &["tsc", "--noEmit"], dir);
        }
        // Vue without tsconfig is OK (JS project)
        println!("Vue project detected (no tsconfig). Skipping type check.");
        return Ok(());
    }

    // Check for Svelte project
    if dir.join("svelte.config.js").exists() || dir.join("svelte.config.ts").exists() {
        if dir.join("node_modules/.bin/svelte-check").exists() {
            return crate::run_cmd("npx", &["svelte-check"], dir);
        }
        if dir.join("tsconfig.json").exists() {
            return crate::run_cmd("npx", &["tsc", "--noEmit"], dir);
        }
        println!("Svelte project detected. Install svelte-check for better type checking.");
        return Ok(());
    }

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
