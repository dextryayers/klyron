use clap::Args;

#[derive(Args)]
pub struct CompatArgs {
    pub target: Option<String>,
}

pub fn run_compat(args: CompatArgs) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match args.target.as_deref() {
        None | Some("check") => check_compat(&dir),
        Some("react") => check_framework_compat(&dir, "react"),
        Some("next") => check_framework_compat(&dir, "next"),
        Some("astro") => check_framework_compat(&dir, "astro"),
        Some("nest") => check_framework_compat(&dir, "nest"),
        Some("prisma") => check_framework_compat(&dir, "prisma"),
        Some(t) => anyhow::bail!("Unknown compat target: {t}"),
    }
}

fn check_compat(dir: &std::path::Path) -> anyhow::Result<()> {
    println!("🔍 Node.js Compatibility Check");
    println!("  Directory: {}", dir.display());
    let pkg = dir.join("package.json");
    if pkg.exists() {
        println!("  ✅ package.json found");
        let content = std::fs::read_to_string(&pkg)?;
        if content.contains("\"type\": \"module\"") {
            println!("  ✅ ESM modules supported");
        }
        if content.contains("next") { println!("  ℹ️  Next.js detected — needs Node.js runtime"); }
        if content.contains("express") { println!("  ✅ Express.js compatible"); }
        if content.contains("prisma") { println!("  ✅ Prisma compatible (runs via npx)"); }
    } else {
        println!("  ⚠️  No package.json found");
    }
    Ok(())
}

fn check_framework_compat(dir: &std::path::Path, framework: &str) -> anyhow::Result<()> {
    let pkg = dir.join("package.json");
    if !pkg.exists() {
        println!("❌ No package.json found");
        return Ok(());
    }
    let content = std::fs::read_to_string(&pkg)?;
    println!("🔍 {} Compatibility Check", framework);
    if content.contains(framework) {
        println!("  ✅ {} detected in dependencies", framework);
        println!("  ⚠️  Full compatibility requires Node.js runtime (Phase 3)");
        println!("  ℹ️  Scaffold via: klyron create {}", framework);
    } else {
        println!("  ❌ {} not found in dependencies", framework);
    }
    Ok(())
}
