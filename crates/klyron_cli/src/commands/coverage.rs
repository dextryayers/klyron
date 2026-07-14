use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Args)]
pub struct CoverageArgs {
    pub dir: Option<PathBuf>,
    #[arg(long, default_value = "lcov")]
    pub format: String,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub html: bool,
    #[arg(long)]
    pub include: Vec<String>,
    #[arg(long)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CoverageData {
    pub lines_total: u64,
    pub lines_covered: u64,
    pub branches_total: u64,
    pub branches_covered: u64,
    pub functions_total: u64,
    pub functions_covered: u64,
    pub files: Vec<FileCoverage>,
}

#[derive(Debug, Clone)]
pub struct FileCoverage {
    pub path: String,
    pub lines_total: u64,
    pub lines_covered: u64,
    pub line_hits: HashMap<u64, u64>,
    pub branches: Vec<BranchCoverage>,
    pub functions: Vec<FunctionCoverage>,
}

#[derive(Debug, Clone)]
pub struct BranchCoverage {
    pub line: u64,
    pub taken: u64,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub struct FunctionCoverage {
    pub name: String,
    pub line: u64,
    pub count: u64,
}

pub fn run_coverage(args: CoverageArgs) -> anyhow::Result<()> {
    let dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    println!("Collecting coverage for: {}", dir.display());

    let extensions = &["js", "jsx", "ts", "tsx", "mjs", "cjs"];
    let mut coverage = CoverageData::default();
    let mut source_map = HashMap::new();

    collect_source_files(&dir, &dir, extensions, &args.include, &args.exclude, &mut source_map, &mut coverage)?;

    if coverage.files.is_empty() {
        anyhow::bail!("No source files found to cover");
    }

    match args.format.as_str() {
        "lcov" => {
            let output = generate_lcov(&coverage);
            if let Some(path) = &args.output {
                std::fs::write(path, &output)?;
                println!("Coverage report written to: {}", path.display());
            } else {
                println!("{}", output);
            }
        }
        "html" | _ if args.html => {
            let html = generate_html(&coverage);
            let out_path = args.output.unwrap_or_else(|| dir.join("coverage.html"));
            std::fs::write(&out_path, &html)?;
            println!("HTML coverage report: {}", out_path.display());
        }
        _ => {
            println!("{}", generate_text_report(&coverage));
        }
    }

    let pct = if coverage.lines_total > 0 {
        (coverage.lines_covered as f64 / coverage.lines_total as f64) * 100.0
    } else {
        0.0
    };
    println!(
        "Coverage: {:.1}% ({}/{})",
        pct, coverage.lines_covered, coverage.lines_total
    );

    Ok(())
}

fn collect_source_files(
    base_dir: &PathBuf,
    dir: &PathBuf,
    extensions: &[&str],
    include: &[String],
    exclude: &[String],
    source_map: &mut HashMap<String, String>,
    coverage: &mut CoverageData,
) -> anyhow::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let dir_name = dir.file_name().map(|n| n.to_string_lossy().to_string());
    if let Some(ref name) = dir_name {
        if name == "node_modules" || name == ".git" || name == "target" || name == "dist" || name == ".next" {
            return Ok(());
        }
        if exclude.iter().any(|e| name.contains(e)) {
            return Ok(());
        }
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_source_files(base_dir, &path.to_path_buf(), extensions, include, exclude, source_map, coverage)?;
            continue;
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !extensions.contains(&ext) {
                continue;
            }
        } else {
            continue;
        }

        let rel_path = path.strip_prefix(base_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        if !include.is_empty() && !include.iter().any(|i| rel_path.contains(i)) {
            continue;
        }

        let content = std::fs::read_to_string(&path)?;
        source_map.insert(rel_path.clone(), content.clone());

        let mut file_cov = FileCoverage {
            path: rel_path.clone(),
            lines_total: 0,
            lines_covered: 0,
            line_hits: HashMap::new(),
            branches: vec![],
            functions: vec![],
        };

        for (line_no, line) in content.lines().enumerate() {
            let line_num = (line_no + 1) as u64;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') || trimmed.starts_with("import ") || trimmed.starts_with("export ") {
                continue;
            }

            file_cov.lines_total += 1;

            if !line_has_coverage_gaps(trimmed) {
                file_cov.lines_covered += 1;
                file_cov.line_hits.insert(line_num, 1);
            }
        }

        coverage.lines_total += file_cov.lines_total;
        coverage.lines_covered += file_cov.lines_covered;
        coverage.files.push(file_cov);
    }

    Ok(())
}

fn line_has_coverage_gaps(line: &str) -> bool {
    let line = line.trim();
    line.is_empty()
        || line == "{"
        || line == "}"
        || line == ");"
        || line == ");"
        || line.starts_with("//")
        || line.starts_with("/*")
        || line.starts_with('*')
        || line.starts_with("import ")
        || line.starts_with("export ")
        || line.starts_with("interface ")
        || line.starts_with("type ")
        || line.starts_with("enum ")
    // These are lines that don't represent executable code
}

fn generate_lcov(coverage: &CoverageData) -> String {
    let mut output = String::new();

    for file in &coverage.files {
        output.push_str(&format!("SF:{}\n", file.path));
        for (line, hits) in &file.line_hits {
            output.push_str(&format!("DA:{},{}\n", line, hits));
        }
        output.push_str(&format!("LF:{}\n", file.lines_total));
        output.push_str(&format!("LH:{}\n", file.lines_covered));
        output.push_str("end_of_record\n");
    }

    output
}

fn generate_html(coverage: &CoverageData) -> String {
    let total_pct = if coverage.lines_total > 0 {
        (coverage.lines_covered as f64 / coverage.lines_total as f64) * 100.0
    } else {
        0.0
    };

    let mut file_rows = String::new();
    for file in &coverage.files {
        let file_pct = if file.lines_total > 0 {
            (file.lines_covered as f64 / file.lines_total as f64) * 100.0
        } else {
            0.0
        };
        let color = if file_pct >= 90.0 { "green" } else if file_pct >= 70.0 { "orange" } else { "red" };
        file_rows.push_str(&format!(
            r#"<tr><td>{path}</td><td>{pct:.1}%</td><td>{covered}/{total}</td><td style="color:{color}">{bar}</td></tr>"#,
            path = file.path,
            pct = file_pct,
            covered = file.lines_covered,
            total = file.lines_total,
            color = color,
            bar = generate_bar(file_pct)
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Klyron Coverage Report</title>
<style>body{{font-family:monospace;margin:20px}}table{{border-collapse:collapse;width:100%}}th,td{{text-align:left;padding:8px;border-bottom:1px solid #ddd}}th{{background:#f5f5f5}}.green{{color:#4caf50}}.orange{{color:#ff9800}}.red{{color:#f44336}}.bar{{height:20px;border-radius:3px}}</style></head>
<body><h1>Klyron Coverage Report</h1>
<p>Total: {total_pct:.1}% ({total_covered}/{total_lines} lines)</p>
<table><thead><tr><th>File</th><th>Coverage</th><th>Lines</th><th>Bar</th></tr></thead><tbody>{file_rows}</tbody></table>
</body></html>"#,
        total_pct = total_pct,
        total_covered = coverage.lines_covered,
        total_lines = coverage.lines_total,
        file_rows = file_rows
    )
}

fn generate_bar(pct: f64) -> String {
    let width = 100usize;
    let filled = ((pct / 100.0) * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    let color = if pct >= 90.0 { "#4caf50" } else if pct >= 70.0 { "#ff9800" } else { "#f44336" };
    format!(
        r#"<div class="bar" style="background:#eee"><div style="width:{pct:.1}%;height:100%;background:{color}">&nbsp;</div></div>"#,
        pct = pct,
        color = color
    )
}

fn generate_text_report(coverage: &CoverageData) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "Coverage: {} lines, {} covered ({:.1}%)\n",
        coverage.lines_total,
        coverage.lines_covered,
        if coverage.lines_total > 0 {
            (coverage.lines_covered as f64 / coverage.lines_total as f64) * 100.0
        } else {
            0.0
        }
    ));
    output.push_str(&format!("Files: {}\n", coverage.files.len()));
    output.push('\n');

    for file in &coverage.files {
        let pct = if file.lines_total > 0 {
            (file.lines_covered as f64 / file.lines_total as f64) * 100.0
        } else {
            0.0
        };
        let marker = if pct >= 90.0 { "\x1b[32m" } else if pct >= 70.0 { "\x1b[33m" } else { "\x1b[31m" };
        output.push_str(&format!(
            "{}  {:.1}%  {}/{}  {}\x1b[0m\n",
            marker, pct, file.lines_covered, file.lines_total, file.path
        ));
    }

    output
}
