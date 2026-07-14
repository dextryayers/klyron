use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct DocPadAdapter;

#[async_trait]
impl FrameworkAdapter for DocPadAdapter {
    fn name(&self) -> &'static str { "docpad" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"docpad\""))
            .unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["6.82"] }
    fn default_version(&self) -> &'static str { "6.82" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::StaticSiteGenerator }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["docpad", "run"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["docpad", "generate"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, _dir: &Path, _filter: Option<&str>) -> Result<()> { Ok(()) }

    async fn lint(&self, _dir: &Path, _fix: bool) -> Result<()> { Ok(()) }

    async fn format(&self, _dir: &Path, _write: bool) -> Result<()> { Ok(()) }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src/documents"))?;
        std::fs::create_dir_all(project_dir.join("src/layouts"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "dev": "docpad run",
    "build": "docpad generate"
  },
  "dependencies": { "docpad": "^6.82.0" }
}"#, vars))?;

        std::fs::write(project_dir.join("docpad.coffee"),
            klyron_template::TemplateEngine::render_static(r#"# DocPad Configuration
docpadConfig = {
  templateData:
    site:
      title: "{{ name }}"
      description: "A DocPad site"
}
module.exports = docpadConfig
"#, vars))?;

        std::fs::write(project_dir.join("src/documents/index.html.md"),
            klyron_template::TemplateEngine::render_static(r#"---
layout: default
title: Home
---

# Welcome to {{ name }}

This site was built with DocPad.
"#, vars))?;

        std::fs::write(project_dir.join("src/documents/about.html.md"),
            r#"---
layout: default
title: About
---

## About

Static site generated with DocPad.
"#)?;

        std::fs::write(project_dir.join("src/layouts/default.html.eco"),
            klyron_template::TemplateEngine::render_static(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title><%= @site.title %></title>
</head>
<body>
  <header><h1><%= @site.title %></h1></header>
  <main><%- @content %></main>
</body>
</html>
"#, vars))?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\nout\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

DocPad static site

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
