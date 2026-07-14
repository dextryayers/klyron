use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct MetalsmithAdapter;

#[async_trait]
impl FrameworkAdapter for MetalsmithAdapter {
    fn name(&self) -> &'static str { "metalsmith" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"metalsmith\""))
            .unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["2.6"] }
    fn default_version(&self) -> &'static str { "2.6" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::StaticSiteGenerator }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["metalsmith", "--watch"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("node").args(["build.js"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, _dir: &Path, _filter: Option<&str>) -> Result<()> { Ok(()) }

    async fn lint(&self, _dir: &Path, _fix: bool) -> Result<()> { Ok(()) }

    async fn format(&self, _dir: &Path, _write: bool) -> Result<()> { Ok(()) }

    fn external_scaffold_command(&self, _name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        None
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src"))?;
        std::fs::create_dir_all(project_dir.join("layouts"))?;
        std::fs::create_dir_all(project_dir.join("partials"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "metalsmith --watch",
    "build": "node build.js"
  },
  "dependencies": {
    "metalsmith": "^2.6.0",
    "metalsmith-layouts": "^2.6.0",
    "metalsmith-markdown": "^1.4.0",
    "handlebars": "^4.7.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("build.js"),
            klyron_template::TemplateEngine::render_static(r#"import Metalsmith from 'metalsmith'
import markdown from 'metalsmith-markdown'
import layouts from 'metalsmith-layouts'

Metalsmith(__dirname)
  .source('./src')
  .destination('./build')
  .use(markdown())
  .use(layouts({ engine: 'handlebars', directory: './layouts' }))
  .build(err => { if (err) throw err })
"#, vars))?;

        std::fs::write(project_dir.join("src/index.md"),
            klyron_template::TemplateEngine::render_static(r#"---
title: Home
layout: layout.hbs
---

# Welcome to {{ name }}

This site was built with Metalsmith.
"#, vars))?;

        std::fs::write(project_dir.join("src/about.md"),
            r#"---
title: About
layout: layout.hbs
---

## About

Static site generated with Metalsmith.
"#)?;

        std::fs::write(project_dir.join("layouts/layout.hbs"),
            klyron_template::TemplateEngine::render_static(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>{{ title }}</title>
</head>
<body>
  {{> header}}
  <main>{{{ contents }}}</main>
  {{> footer}}
</body>
</html>
"#, vars))?;

        std::fs::write(project_dir.join("partials/header.hbs"),
            klyron_template::TemplateEngine::render_static(r#"<header>
  <h1>{{ name }}</h1>
  <nav><a href="/">Home</a> | <a href="/about">About</a></nav>
</header>
"#, vars))?;

        std::fs::write(project_dir.join("partials/footer.hbs"),
            r#"<footer>
  <p>Powered by Metalsmith</p>
</footer>
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\nbuild\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Metalsmith static site

## Getting Started

npm run build
"#, vars))?;

        Ok(())
    }
}
