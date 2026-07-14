use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct WintersmithAdapter;

#[async_trait]
impl FrameworkAdapter for WintersmithAdapter {
    fn name(&self) -> &'static str { "wintersmith" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"wintersmith\""))
            .unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["2.5"] }
    fn default_version(&self) -> &'static str { "2.5" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::StaticSiteGenerator }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["wintersmith", "preview"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["wintersmith", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, _dir: &Path, _filter: Option<&str>) -> Result<()> { Ok(()) }

    async fn lint(&self, _dir: &Path, _fix: bool) -> Result<()> { Ok(()) }

    async fn format(&self, _dir: &Path, _write: bool) -> Result<()> { Ok(()) }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("contents"))?;
        std::fs::create_dir_all(project_dir.join("templates"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "dev": "wintersmith preview",
    "build": "wintersmith build"
  },
  "dependencies": { "wintersmith": "^2.5.0" }
}"#, vars))?;

        std::fs::write(project_dir.join("config.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "locals": { "url": "http://localhost:8080", "name": "{{ name }}", "owner": "Anonymous" },
  "plugins": [],
  "require": {}
}"#, vars))?;

        std::fs::write(project_dir.join("contents/index.md"),
            klyron_template::TemplateEngine::render_static(r#"---
title: Home
template: index.jade
---

# Welcome to {{ name }}

This site was built with Wintersmith.
"#, vars))?;

        std::fs::write(project_dir.join("contents/about.md"),
            r#"---
title: About
template: index.jade
---

## About

Static site generated with Wintersmith.
"#)?;

        std::fs::write(project_dir.join("templates/index.jade"),
            klyron_template::TemplateEngine::render_static(r#"extends layout

block content
  article
    != page.html
"#, vars))?;

        std::fs::write(project_dir.join("templates/layout.jade"),
            klyron_template::TemplateEngine::render_static(r#"doctype html
html(lang="en")
  head
    title= locals.name
    meta(charset="utf-8")
  body
    header
      h1= locals.name
    main
      block content
    footer
      p &copy; #{ locals.owner }
"#, vars))?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\nbuild\n.DS_Store\n")?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Wintersmith static site

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
