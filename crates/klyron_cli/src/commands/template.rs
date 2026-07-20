use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use dialoguer::{Select, theme::ColorfulTheme, console::Term};

use crate::anim::{StepAnim, success_banner};
use crate::color::Color;

// ── Adapter directory scanner ─────────────────────────────────────────────

// ── Public API ─────────────────────────────────────────────────────────────

pub fn template_exists(name: &str) -> bool {
  scan_adapters().iter().any(|t| t.name == name)
}

pub fn find_template(name: &str) -> Option<TemplateInfo> {
  scan_adapters().into_iter().find(|t| t.name == name)
}

// ── Internal types ─────────────────────────────────────────────────────────

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
  // 1. Explicit env var
  if let Ok(dir) = std::env::var("KLYRON_ADAPTERS_DIR") {
    let p = std::path::PathBuf::from(&dir);
    if has_adapters(&p) { return p; }
  }

  // 2. Compile-time path (works when running from klyron source tree)
  let src_root = Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent().and_then(|p| p.parent())
    .map(|p| p.join("adapters"));
  if let Some(p) = src_root {
    if has_adapters(&p) { return p; }
  }

  // 3. Walk up from CWD looking for adapters/
  let cwd = std::env::current_dir().unwrap_or_default();
  let mut current = Some(cwd.as_path());
  while let Some(dir) = current {
    let candidate = dir.join("adapters");
    if has_adapters(&candidate) {
      return candidate;
    }
    current = dir.parent();
  }

  // 4. Relative to the binary, walking up (handles global installs)
  if let Ok(exe) = std::env::current_exe() {
    let mut current = exe.parent();
    while let Some(dir) = current {
      let candidate = dir.join("adapters");
      if has_adapters(&candidate) {
        return candidate;
      }
      let share = dir.join("share").join("klyron").join("adapters");
      if has_adapters(&share) {
        return share;
      }
      current = dir.parent();
    }
  }

  // 5. User home data directory
  if let Some(home) = dirs::data_dir() {
    let candidate = home.join("klyron").join("adapters");
    if has_adapters(&candidate) {
      return candidate;
    }
  }

  cwd.join("adapters") // fallback — will fail with a clear error downstream
}

fn has_adapters(dir: &std::path::Path) -> bool {
  dir.join("backend").exists() || dir.join("frontend").exists() || dir.join("laravel").exists()
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

fn clean_text(s: &str) -> String {
  let s = s.trim();
  // Strip HTML tags
  let s = strip_html(s);
  // Replace markdown links [text](url) → text
  let s = replace_md_links(&s);

  // Strip remaining markdown formatting
  let s = s.replace("```", "").replace('`', "");
  let s = s.replace("**", "").replace("__", "");
  let s = s.replace('*', "").replace('_', "");
  // Collapse whitespace
  let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
  s.trim().to_string()
}

fn strip_html(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut in_tag = false;
  for ch in s.chars() {
    match ch {
      '<' => in_tag = true,
      '>' => in_tag = false,
      _ if !in_tag => out.push(ch),
      _ => {}
    }
  }
  out
}

fn replace_md_links(s: &str) -> String {
  // Replace [text](url) with just text — handles nested parens
  let mut out = String::with_capacity(s.len());
  let chars: Vec<char> = s.chars().collect();
  let mut i = 0;
  while i < chars.len() {
    if chars[i] == '[' {
      // Find matching ]
      let mut depth = 1;
      let mut text_end = None;
      for j in (i + 1)..chars.len() {
        if chars[j] == ']' { text_end = Some(j); break; }
        if chars[j] == '[' { depth += 1; }
      }
      if let Some(end) = text_end {
        if end + 1 < chars.len() && chars[end + 1] == '(' {
          // Find matching ) handling nested parens
          let mut paren_depth = 1;
          let mut url_end = None;
          for j in (end + 2)..chars.len() {
            if chars[j] == ')' && paren_depth == 1 { url_end = Some(j); break; }
            if chars[j] == '(' { paren_depth += 1; }
            if chars[j] == ')' { paren_depth -= 1; }
          }
          if let Some(url_e) = url_end {
            // Output just the text, trim spaces
            let text: String = chars[i + 1..end].iter().collect();
            out.push_str(text.trim());
            i = url_e + 1;
            continue;
          }
        }
      }
    }
    out.push(chars[i]);
    i += 1;
  }
  out
}

fn is_bad_line(s: &str) -> bool {
  let s = s.trim();
  s.is_empty()
    || s.starts_with('#')
    || s.starts_with("```")
    || s.starts_with('[') // reference-style links
    || s.starts_with("npm")
    || s.starts_with("pnpm")
    || s.starts_with("yarn")
    || s.starts_with("bun")
    || s.starts_with('$')
    || s.starts_with("<!--")
    || s.starts_with("<img")
    || s.starts_with("<a ")
    || s.starts_with("<p ")
    || s.starts_with("<div")
    || s.starts_with("<center")
    || s.contains("://img.")
    || s.contains("badge")
    || s.contains("circleci")
    || s.contains("http") && s.len() < 20
    || s.len() > 200
}

fn read_description(dir: &Path) -> String {
  // Try package.json first — cleanest source
  let pkg = dir.join("package.json");
  if pkg.exists() {
    if let Ok(content) = std::fs::read_to_string(&pkg) {
      if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(desc) = json.get("description").and_then(|d| d.as_str()) {
          let cleaned = clean_text(desc);
          if !cleaned.is_empty() && cleaned.len() < 120 && !is_bad_line(&cleaned) {
            return cleaned;
          }
        }
      }
    }
  }
  // Fallback: README.md — first good paragraph
  let readme = dir.join("README.md");
  if readme.exists() {
    if let Ok(content) = std::fs::read_to_string(&readme) {
      for line in content.lines() {
        let trimmed = line.trim();
        if is_bad_line(trimmed) { continue; }
        let cleaned = clean_text(trimmed);
        if !cleaned.is_empty() && cleaned.len() < 120 && !is_bad_line(&cleaned) {
          return cleaned;
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
        let desc = &t.description;
        if desc.len() > 80 {
          // truncate with ellipsis
          let truncated: String = desc.chars().take(77).collect();
          println!("    {:>2}  {:<24}{}",
            "",
            "",
            Color::DIM.paint(format!("{}...", truncated)),
          );
        } else {
          println!("    {:>2}  {:<24}{}",
            "",
            "",
            Color::DIM.paint(desc),
          );
        }
      }
    }
    println!();
  }

  println!("  {}", Color::BRIGHT_YELLOW.bold("Usage"));
  println!("    {}  {}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint(format!("{:<24}", "template show <template>")),
    Color::WHITE.paint("Show template details & versions"),
  );
  println!("    {}  {}{}",
    Color::GREEN.paint("▶"),
    Color::BRIGHT_CYAN.paint(format!("{:<24}", "template create <template> <project>")),
    Color::WHITE.paint("Create project with interactive version picker (↑↓)"),
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
    Color::WHITE.paint("Create project (interactive version picker)"),
  );
}

// ── Version helpers ───────────────────────────────────────────────────────

fn scan_versions(framework_dir: &Path) -> Vec<String> {
  let entries = std::fs::read_dir(framework_dir).ok();
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
  vers
}

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

fn pick_version_interactive(versions: &[String], default: &str) -> dialoguer::Result<String> {
  let term = Term::stderr();
  let _ = term.write_line("");
  let _ = term.write_str(&format!("  {} {}\n", Color::BRIGHT_YELLOW.bold("Select version"), Color::DIM.paint("(↑↓ arrows, Enter to confirm)")));

  let default_idx = versions.iter().position(|v| v == default).unwrap_or(versions.len().saturating_sub(1));

  let styled_items: Vec<String> = versions.iter().map(|v| {
    if v == default {
      format!("{}  {}", v, Color::GREEN.paint("● default"))
    } else {
      v.clone()
    }
  }).collect();

  let selection = Select::with_theme(&ColorfulTheme::default())
    .items(&styled_items)
    .default(default_idx)
    .interact_on_opt(&Term::stderr())?;

  Ok(selection.map(|i| versions[i].clone()).unwrap_or_else(|| default.to_string()))
}

// ── Create Command ────────────────────────────────────────────────────────

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

  // Determine version — interactive picker if not specified
  let versions = scan_versions(&framework_dir);
  let version = match version {
    Some(v) => v.to_string(),
    None if versions.is_empty() => "latest".to_string(),
    None => {
      let default = versions.last().cloned().unwrap_or_else(|| "latest".to_string());
      if versions.len() == 1 {
        default
      } else {
        match pick_version_interactive(&versions, &default) {
          Ok(v) => v,
          Err(_) => {
            steps.step_fail("Version selection cancelled");
            anyhow::bail!("Version selection cancelled")
          }
        }
      }
    }
  };

  let version_dir = resolve_version_dir(&framework_dir, &version)
    .ok_or_else(|| {
      steps.step_fail(&format!("Version '{}' not found for {}", version, name));
      anyhow::anyhow!("Version '{version}' not found for '{name}'. List versions with 'klyron template show {name}'")
    })?;

  steps.step_done();

  // Determine the target directory: dir is the parent, project_name is the subfolder
  let cwd = std::env::current_dir().unwrap_or_default();
  let parent = dir.unwrap_or(&cwd);
  let target_dir = parent.join(project_name);

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
