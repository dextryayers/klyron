use std::path::{Path, PathBuf};
use clap::Args;

#[derive(Args)]
pub struct BuildArgs {
    #[arg(long, value_parser = ["web", "wasm", "napi", "binary"], required = true)]
    pub target: String,
    #[arg(long, default_value = "dist")]
    pub out_dir: PathBuf,
    #[arg(long)]
    pub minify: bool,
    #[arg(long)]
    pub sourcemap: bool,
    #[arg(long, default_value = "esm", value_parser = ["esm", "cjs", "iife"])]
    pub format: String,
    #[arg(default_value = ".")]
    pub entry: PathBuf,
}

pub fn run_build(args: BuildArgs) -> anyhow::Result<()> {
    let out_dir = &args.out_dir;
    std::fs::create_dir_all(out_dir)?;

    let target_str = args.target.as_str();
    crate::anim::cmd_header("build", &format!("Building for target: {}", target_str));

    let mut bar = crate::anim::GradientBar::new(100, &format!("Building {}...", target_str));

    let result = match target_str {
        "web" => build_web(&args),
        "wasm" => build_wasm(&args),
        "napi" => build_napi(&args),
        "binary" => build_binary(&args),
        _ => anyhow::bail!("Unknown target: {}", args.target),
    };

    if result.is_ok() {
        bar.finish_with(&format!("Build complete → {}", out_dir.display()));
        crate::anim::success_banner("Build successful");
    }
    result
}

fn build_web(args: &BuildArgs) -> anyhow::Result<()> {
    let entry = &args.entry;
    let source = if entry.is_file() {
        std::fs::read_to_string(entry)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", entry.display()))?
    } else {
        anyhow::bail!("Entry file not found: {}", entry.display())
    };

    let js = if entry.to_str().map(|s| s.ends_with(".ts")).unwrap_or(false) {
        klyron_transpiler::transpile_ts_to_js(&source)?
    } else {
        source
    };

    let js = if args.minify {
        minify_js(&js)
    } else {
        js
    };

    let bundle_path = out_dir(&args.out_dir, "bundle.js");
    std::fs::write(&bundle_path, &js)?;
    eprintln!("  {} Written: {}", crate::Color::GREEN.paint("\u{2713}"), bundle_path.display());

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Klyron App</title>
<style>*{{margin:0;padding:0;box-sizing:border-box}}body{{font-family:system-ui,-apple-system,sans-serif}}</style>
</head>
<body>
<div id="root"></div>
<script src="bundle.js"></script>
</body>
</html>
"#
    );
    let index_path = args.out_dir.join("index.html");
    std::fs::write(&index_path, &html)?;
    eprintln!("  {} Written: {}", crate::Color::GREEN.paint("\u{2713}"), index_path.display());

    if args.sourcemap {
        let sourcemap_content = format!("//# sourceMappingURL=bundle.js.map\n");
        std::fs::write(&bundle_path, format!("{js}\n{sourcemap_content}"))?;
        let sm_path = out_dir(&args.out_dir, "bundle.js.map");
        let sm = serde_json::json!({
            "version": 3,
            "file": "bundle.js",
            "sources": [entry.to_str().unwrap_or("entry.js")],
            "mappings": ""
        });
        std::fs::write(&sm_path, serde_json::to_string_pretty(&sm)?)?;
        eprintln!("  {} Sourcemap: {}", crate::Color::GREEN.paint("\u{2713}"), sm_path.display());
    }

    Ok(())
}

fn build_wasm(args: &BuildArgs) -> anyhow::Result<()> {
    let entry = &args.entry;
    let source = std::fs::read_to_string(entry)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", entry.display()))?;

    if source.trim().starts_with('(') || source.trim().starts_with("(module") {
        let wasm_path = out_dir(&args.out_dir, "module.wasm");
        let bytes = wat::parse_str(&source)
            .map_err(|e| anyhow::anyhow!("Failed to parse .wat: {e}"))?;
        std::fs::write(&wasm_path, &bytes)?;
        eprintln!("  {} Compiled .wat to {}", crate::Color::GREEN.paint("\u{2713}"), wasm_path.display());
    } else {
        let status = std::process::Command::new("wasm-pack")
            .args(["build", "--target", "web", "--out-dir"])
            .arg(args.out_dir.to_str().unwrap_or("dist"))
            .arg("--")
            .arg(entry.to_str().unwrap_or("."))
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run wasm-pack: {e}"))?;
        if !status.success() {
            anyhow::bail!("wasm-pack build failed");
        }
        eprintln!("  {} wasm-pack build complete", crate::Color::GREEN.paint("\u{2713}"));
    }

    Ok(())
}

fn build_napi(args: &BuildArgs) -> anyhow::Result<()> {
    let header_path = out_dir(&args.out_dir, "klyron_napi.h");
    let source_path = out_dir(&args.out_dir, "klyron_napi.c");

    let h_content = r#"#ifndef KLYRON_NAPI_H
#define KLYRON_NAPI_H

#include <node_api.h>

napi_value klyron_init(napi_env env, napi_value exports);

#endif
"#;
    let c_content = r#"#include "klyron_napi.h"

napi_value klyron_init(napi_env env, napi_value exports) {
    return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, klyron_init)
"#;

    std::fs::write(&header_path, h_content)?;
    eprintln!("  {} Generated: {}", crate::Color::GREEN.paint("\u{2713}"), header_path.display());
    std::fs::write(&source_path, c_content)?;
    eprintln!("  {} Generated: {}", crate::Color::GREEN.paint("\u{2713}"), source_path.display());
    eprintln!("  {} N-API bindings written to {}", crate::Color::CYAN.paint("i"), args.out_dir.display());
    eprintln!("  {} Build with: node-gyp configure build", crate::Color::DIM.paint("  "));

    let binding_gyp = r#"{
  "targets": [{
    "target_name": "klyron_napi",
    "sources": [ "klyron_napi.c" ],
    "include_dirs": ["<!(node -e \"require('node-api-headers')\")"]
  }]
}
"#;
    let gyp_path = out_dir(&args.out_dir, "binding.gyp");
    std::fs::write(&gyp_path, binding_gyp)?;
    eprintln!("  {} Generated: {}", crate::Color::GREEN.paint("\u{2713}"), gyp_path.display());

    Ok(())
}

fn build_binary(args: &BuildArgs) -> anyhow::Result<()> {
    let entry = &args.entry;
    if !entry.exists() {
        anyhow::bail!("Entry file not found: {}", entry.display());
    }

    let klyron_bin = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Cannot determine current executable: {e}"))?;
    let target_bin = out_dir(&args.out_dir, "klyron_app");
    std::fs::copy(&klyron_bin, &target_bin)?;

    let script_name = entry
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("script.js");
    let script_dest = out_dir(&args.out_dir, script_name);
    std::fs::copy(entry, &script_dest)?;

    let runner = if cfg!(unix) {
        r#"#!/bin/sh
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
exec "$SCRIPT_DIR/klyron_app" run "$SCRIPT_DIR/""#.to_string() + script_name + "\" \"$@\""
    } else {
        format!(
            "@echo off\r\n\"%~dp0klyron_app\" run \"%~dp0{}\" %*\r\n",
            script_name
        )
    };

    let runner_path = if cfg!(unix) {
        out_dir(&args.out_dir, "run.sh")
    } else {
        out_dir(&args.out_dir, "run.bat")
    };
    std::fs::write(&runner_path, &runner)?;

        if cfg!(unix) {
            std::fs::set_permissions(&runner_path, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;
            std::fs::set_permissions(&target_bin, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;
        }

    eprintln!("  {} Copied klyron binary to {}", crate::Color::GREEN.paint("\u{2713}"), target_bin.display());
    eprintln!("  {} Script: {}", crate::Color::GREEN.paint("\u{2713}"), script_dest.display());
    eprintln!("  {} Runner: {}", crate::Color::GREEN.paint("\u{2713}"), runner_path.display());

    Ok(())
}

fn out_dir(base: &Path, name: &str) -> PathBuf {
    base.join(name)
}

fn minify_js(code: &str) -> String {
    let mut result = String::with_capacity(code.len());
    let mut in_string = false;
    let mut string_char = ' ';
    let mut prev_char = ' ';
    let mut in_block_comment = false;
    let mut in_line_comment = false;
    let mut chars = code.chars().peekable();

    while let Some(c) = chars.next() {
        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
            }
            continue;
        }
        if in_string {
            result.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else if c == string_char {
                in_string = false;
            }
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            in_line_comment = true;
            continue;
        }
        if c == '/' && chars.peek() == Some(&'*') {
            in_block_comment = true;
            chars.next();
            continue;
        }
        if c == '"' || c == '\'' || c == '`' {
            in_string = true;
            string_char = c;
            result.push(c);
            continue;
        }
        if c.is_whitespace() {
            let next = chars.peek().copied();
            let is_separator = prev_char.is_ascii_punctuation()
                || next.map_or(true, |n| n.is_ascii_punctuation() || n == '\n');
            if !is_separator && prev_char != ' ' {
                result.push(' ');
            }
            prev_char = ' ';
            continue;
        }
        result.push(c);
        prev_char = c;
    }
    result
}
