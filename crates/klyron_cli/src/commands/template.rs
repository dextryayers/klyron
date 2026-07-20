use std::collections::BTreeMap;
use std::path::Path;

use crate::anim::{StepAnim, success_banner};
use crate::color::Color;

// ── Adapter directory scanner ─────────────────────────────────────────────

#[derive(Debug)]
pub struct TemplateInfo {
  pub name: String,
  pub category: String,
  pub versions: Vec<String>,
  pub default_version: String,
  pub kind: String,
  pub description: String,
}

fn adapters_dir() -> std::path::PathBuf {
  let cwd = std::env::current_dir().unwrap_or_default();
  let candidates = [
    cwd.join("adapters"),
    cwd.join("../../adapters"),
    std::env::current_exe()
      .ok()
      .and_then(|p| p.parent().map(|d| d.join("../adapters")))
      .unwrap_or_default(),
  ];
  for candidate in &candidates {
    if candidate.join("backend").exists() || candidate.join("frontend").exists() {
      return candidate.clone();
    }
  }
  let env_dir = std::env::var("KLYRON_ADAPTERS_DIR").ok();
  env_dir.map(std::path::PathBuf::from).unwrap_or(cwd.join("adapters"))
}

fn scan_adapters() -> Vec<TemplateInfo> {
  let adapter_dir = adapters_dir();
  let mut templates = Vec::new();

  let categories = ["backend", "frontend", "laravel"];
  for cat in &categories {
    let cat_dir = adapter_dir.join(cat);
    if !cat_dir.exists() || !cat_dir.is_dir() {
      continue;
    }
    let entries = match std::fs::read_dir(&cat_dir) {
      Ok(e) => e,
      Err(_) => continue,
    };
    for entry in entries.flatten() {
      let path = entry.path();
      if !path.is_dir() {
        continue;
      }
      let name = path.file_name().unwrap().to_string_lossy().to_string();

      let mut versions: Vec<String> = Vec::new();
      let mut has_pkg = false;
      if let Ok(ver_entries) = std::fs::read_dir(&path) {
        for ve in ver_entries.flatten() {
          let vpath = ve.path();
          if vpath.is_dir() {
            let vname = vpath.file_name().unwrap().to_string_lossy().to_string();
            if vname.starts_with('v') || vname.chars().next().map_or(false, |c| c.is_numeric()) {
              versions.push(vname);
            }
          } else if vpath.is_file() && vpath.file_name().unwrap() == "package.json" {
            has_pkg = true;
          }
        }
      }
      if versions.is_empty() && has_pkg {
        versions.push("latest".to_string());
      }
      versions.sort_by(|a, b| {
        let a_ver = a.trim_start_matches('v').to_string();
        let b_ver = b.trim_start_matches('v').to_string();
        nat_sort(&a_ver, &b_ver)
      });
      let default_version = versions.last().cloned().unwrap_or_else(|| "latest".to_string());
      let kind = match *cat {
        "backend" => "Backend",
        "frontend" => "Frontend",
        "laravel" => "Fullstack",
        _ => "Unknown",
      };
      let cat_owned = cat.to_string();
      let description = read_description(&path.join(&default_version));
      templates.push(TemplateInfo {
        name,
        category: cat_owned,
        versions,
        default_version,
        kind: kind.to_string(),
        description,
      });
    }
  }
  templates
}

fn nat_sort(a: &str, b: &str) -> std::cmp::Ordering {
  let a_parts: Vec<&str> = a.split(|c: char| !c.is_ascii_digit()).collect();
  let b_parts: Vec<&str> = b.split(|c: char| !c.is_ascii_digit()).collect();
  for (pa, pb) in a_parts.iter().zip(b_parts.iter()) {
    let cmp = if let (Ok(na), Ok(nb)) = (pa.parse::<u64>(), pb.parse::<u64>()) {
      na.cmp(&nb)
    } else {
      pa.cmp(pb)
    };
    if cmp != std::cmp::Ordering::Equal {
      return cmp;
    }
  }
  a.len().cmp(&b.len())
}

fn read_description(dir: &Path) -> String {
  let readme = dir.join("README.md");
  if readme.exists() {
    if let Ok(content) = std::fs::read_to_string(&readme) {
      for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
          return trimmed.to_string();
        }
      }
    }
  }
  let pkg = dir.join("package.json");
  if pkg.exists() {
    if let Ok(content) = std::fs::read_to_string(&pkg) {
      if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(desc) = json.get("description").and_then(|d| d.as_str()) {
          return desc.to_string();
        }
      }
    }
  }
  String::new()
}

// ── List Command ──────────────────────────────────────────────────────────

pub fn list_templates() {
  list_templates_with_filter(None);
}

pub fn list_templates_with_filter(category_filter: Option<&str>) {
  let all_templates = scan_adapters();
  if all_templates.is_empty() {
    println!("  {} No templates found in adapters/ directory", Color::YELLOW.paint("!"));
    println!("  {}", Color::DIM.paint("Run from the Klyron project root or set KLYRON_ADAPTERS_DIR"));
    return;
  }

  let templates: Vec<&TemplateInfo> = match category_filter {
    Some(cat) => all_templates.iter().filter(|t| t.category == cat).collect(),
    None => all_templates.iter().collect(),
  };

  if templates.is_empty() {
    println!("  {} No templates found for category '{}'", Color::YELLOW.paint("!"), category_filter.unwrap());
    println!("  {}", Color::DIM.paint("Available categories: backend, frontend, laravel"));
    return;
  }

  let cat_labels = [
    ("backend", "Backend"),
    ("frontend", "Frontend"),
    ("laravel", "Laravel / Fullstack"),
  ];

  let total = all_templates.len();
  let shown = templates.len();
  let count_suffix = if shown == total { format!("({} found)", total) } else { format!("({} shown of {} total)", shown, total) };
  println!();
  let header = format!("{} Available Templates {}", Color::BRIGHT_YELLOW.bold("Template"), Color::DIM.paint(&count_suffix));
  println!("  {}", header);
  println!();

  for (cat_key, cat_label) in &cat_labels {
    let cat_templates: Vec<&&TemplateInfo> = templates.iter().filter(|t| t.category == *cat_key).collect();
    if cat_templates.is_empty() {
      continue;
    }
    println!("  {}", Color::BRIGHT_YELLOW.bold(cat_label));
    for t in &cat_templates {
      let versions_str = if t.versions.len() > 3 {
        let shown: Vec<&str> = t.versions.iter().rev().take(3).map(|s| s.as_str()).collect();
        format!("{} … ({} ver)", shown.join(", "), t.versions.len())
      } else {
        t.versions.join(", ")
      };
      let label = format!("{}", t.name);
      println!("    {}  {:<24}{}",
        Color::GREEN.paint("▶"),
        Color::BRIGHT_CYAN.paint(format!("{:<24}", label)),
        Color::WHITE.paint(versions_str),
      );
      if !t.description.is_empty() {
        println!("    {:>2}  {:<24}{}",
          "",
          "",
          Color::DIM.paint(&t.description),
        );
      }
    }
    println!();
  }

  println!("  {}", Color::BRIGHT_YELLOW.bold("Usage"));
  println!("    {}  {}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint(format!("{:<24}", "template list")),
    Color::WHITE.paint("List all available templates"),
  );
  println!("    {}  {}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint(format!("{:<24}", "template show <name>")),
    Color::WHITE.paint("Show template details & versions"),
  );
  println!("    {}  {}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint(format!("{:<24}", "template create <name> <project>")),
    Color::WHITE.paint("Create project from template"),
  );
  println!("    {}  {}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint(format!("{:<24}", "template create <name> <project> --version <ver>")),
    Color::WHITE.paint("Create with specific version"),
  );
}

// ── Show Command ──────────────────────────────────────────────────────────

pub fn show_template(name: &str) {
  let templates = scan_adapters();
  let t = match templates.iter().find(|t| t.name == name) {
    Some(t) => t,
    None => {
      println!("  {} Template '{}' not found", Color::YELLOW.paint("!"), name);
      println!("  {}", Color::DIM.paint("Run 'klyron template list' to see available templates"));
      return;
    }
  };

  let cat_label = match t.category.as_str() {
    "backend" => "Backend",
    "frontend" => "Frontend",
    "laravel" => "Laravel / Fullstack",
    _ => &t.category,
  };

  println!();
  println!("  {} {}  {}",
    Color::GREEN.paint("◆"),
    Color::BRIGHT_CYAN.bold(&t.name),
    Color::DIM.paint(cat_label),
  );
  if !t.description.is_empty() {
    println!("  {}", Color::WHITE.paint(&t.description));
  }
  println!();

  println!("  {}", Color::BRIGHT_YELLOW.bold("Details"));
  println!("    {}  {:<18}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint("Category"),
    Color::WHITE.paint(&t.category),
  );
  println!("    {}  {:<18}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint("Kind"),
    Color::WHITE.paint(&t.kind),
  );
  println!("    {}  {:<18}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint("Default version"),
    Color::WHITE.paint(&t.default_version),
  );
  println!("    {}  {:<18}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint("Available versions"),
    Color::WHITE.paint(format!("{}", t.versions.len())),
  );
  println!();

  println!("  {}", Color::BRIGHT_YELLOW.bold("Versions"));
  for v in &t.versions {
    let marker = if *v == t.default_version {
      format!("{}  (default)", Color::GREEN.paint("●"))
    } else {
      format!("{}", Color::DIM.paint("○"))
    };
    println!("    {}  {:<18}{}",
      marker,
      Color::BRIGHT_CYAN.paint(v),
      if *v == t.default_version { Color::DIM.paint("latest recommended") } else { Color::DIM.paint("") },
    );
  }
  println!();

  println!("  {}", Color::BRIGHT_YELLOW.bold("Create"));
  println!("    {}  {:<24}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint(format!("template create {} <project>", name)),
    Color::WHITE.paint(format!("Uses default version ({})", t.default_version)),
  );
  println!("    {}  {:<24}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint(format!("template create {} <project> --version <ver>", name)),
    Color::WHITE.paint("Use a specific version"),
  );
}

// ── Create Command ────────────────────────────────────────────────────────

fn resolve_version_dir(framework_dir: &Path, version: &str) -> Option<std::path::PathBuf> {
  let direct = framework_dir.join(version);
  if direct.is_dir() {
    return Some(direct);
  }
  let with_v = framework_dir.join(format!("v{}", version.trim_start_matches('v')));
  if with_v.is_dir() {
    return Some(with_v);
  }
  None
}

pub fn create_template(name: &str, project_name: &str, version: Option<&str>, dir: Option<&Path>) -> anyhow::Result<()> {
  let mut steps = StepAnim::new(vec![
    "Resolve template".into(),
    "Copy files".into(),
    "Install dependencies".into(),
  ]);
  steps.begin("Creating project");

  let adapter_dir = adapters_dir();
  let cat_dirs = ["backend", "frontend", "laravel"];
  let mut source_dir = None;

  for cat in &cat_dirs {
    let candidate = adapter_dir.join(cat).join(name);
    if candidate.exists() && candidate.is_dir() {
      source_dir = Some((cat.to_string(), candidate));
      break;
    }
  }

  let (_category, framework_dir) = match source_dir {
    Some(s) => s,
    None => {
      steps.step_fail(&format!("Template '{}' not found", name));
      anyhow::bail!("Template '{name}' not found. Run 'klyron template list' to see available templates.")
    }
  };

  // Determine version
  let version = match version {
    Some(v) => v.to_string(),
    None => {
      let entries = std::fs::read_dir(&framework_dir).ok();
      let mut vers: Vec<String> = entries
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
          let v = e.file_name().to_string_lossy().to_string();
          if v.starts_with('v') || v.chars().next().map_or(false, |c| c.is_numeric()) {
            Some(v)
          } else {
            None
          }
        })
        .collect();
      vers.sort_by(|a, b| nat_sort(
        a.trim_start_matches('v'),
        b.trim_start_matches('v'),
      ));
      vers.last().cloned().unwrap_or_else(|| "latest".to_string())
    }
  };

  let version_dir = resolve_version_dir(&framework_dir, &version)
    .ok_or_else(|| {
      steps.step_fail(&format!("Version '{}' not found for {}", version, name));
      anyhow::anyhow!("Version '{version}' not found for '{name}'. List versions with 'klyron template show {name}'")
    })?;

  steps.step_done();

  // Determine the target directory
  let target_dir = match dir {
    Some(d) => d.to_path_buf(),
    None => std::env::current_dir()?.join(project_name),
  };

  steps.step_begin("Copying template files");
  copy_template_dir(&version_dir, &target_dir)?;
  steps.step_ok("Copied");

  if target_dir.join("package.json").exists() {
    steps.step_begin("Installing dependencies");
    install_template_deps(&target_dir)?;
    steps.step_ok("Dependencies installed");
  } else {
    steps.step_done();
  }

  steps.done();

  println!();
  success_banner(&format!("Created {} project at {}", name, target_dir.display()));
  println!();
  println!("  {} Next steps:", Color::BOLD.paint("→"));
  println!("    cd {}", target_dir.display());
  let pm = detect_package_manager(&target_dir);
  if target_dir.join(".env.example").exists() && target_dir.join("artisan").exists() {
    println!("    cp .env.example .env");
    println!("    php artisan key:generate");
    println!("    php artisan serve");
  } else if target_dir.join("package.json").exists() {
    println!("    {} run dev", pm);
  }
  if target_dir.join("Cargo.toml").exists() {
    println!("    cargo run");
  }

  Ok(())
}

fn detect_package_manager(dir: &Path) -> &'static str {
  if dir.join("bun.lock").exists() || dir.join("bun.lockb").exists() { "bun" }
  else if dir.join("pnpm-lock.yaml").exists() { "pnpm" }
  else if dir.join("yarn.lock").exists() { "yarn" }
  else { "npm" }
}

fn copy_template_dir(src: &Path, dst: &Path) -> anyhow::Result<()> {
  if !dst.exists() {
    std::fs::create_dir_all(dst)?;
  }
  copy_dir_recursive(src, dst, src)
}

fn copy_dir_recursive(src_root: &Path, dst_root: &Path, current: &Path) -> anyhow::Result<()> {
  for entry in std::fs::read_dir(current)? {
    let entry = entry?;
    let path = entry.path();
    let rel = path.strip_prefix(src_root).unwrap();
    let target = dst_root.join(rel);

    let fname = entry.file_name();
    let name = fname.to_string_lossy();
    if name.starts_with('.') && name != ".gitignore" && name != ".env.example" && name != ".editorconfig" && name != ".env" {
      continue;
    }

    if entry.file_type()?.is_dir() {
      std::fs::create_dir_all(&target)?;
      copy_dir_recursive(src_root, dst_root, &path)?;
    } else {
      if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
      }
      std::fs::copy(&path, &target)?;
    }
  }
  Ok(())
}

fn install_template_deps(dir: &Path) -> anyhow::Result<()> {
  use std::process::Command;
  let pm = detect_package_manager(dir);

  let mut cmd = Command::new(pm);
  cmd.arg("install").current_dir(dir);

  let output = cmd.output().map_err(|e| anyhow::anyhow!("Failed to run {pm} install: {e}"))?;
  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("  {} Dependency install warning: {}", Color::YELLOW.paint("!"), stderr.lines().next().unwrap_or("unknown error"));
  }
  Ok(())
}
