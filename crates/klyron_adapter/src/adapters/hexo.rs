use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct HexoAdapter;

#[async_trait]
impl FrameworkAdapter for HexoAdapter {
    fn name(&self) -> &'static str { "hexo" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"hexo\""))
            .unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["7.0"] }
    fn default_version(&self) -> &'static str { "7.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::StaticSiteGenerator }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["hexo", "server"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["hexo", "generate"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, _dir: &Path, _filter: Option<&str>) -> Result<()> { Ok(()) }

    async fn lint(&self, _dir: &Path, _fix: bool) -> Result<()> { Ok(()) }

    async fn format(&self, _dir: &Path, _write: bool) -> Result<()> { Ok(()) }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("npx".into(), vec!["hexo-cli".into(), "init".into(), name.into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("source/_posts"))?;
        std::fs::create_dir_all(project_dir.join("source/about"))?;
        std::fs::create_dir_all(project_dir.join("scaffolds"))?;
        std::fs::create_dir_all(project_dir.join("themes/default/layout"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "dev": "hexo server",
    "build": "hexo generate",
    "clean": "hexo clean"
  },
  "dependencies": {
    "hexo": "^7.0.0",
    "hexo-server": "^3.0.0",
    "hexo-generator-archive": "^2.0.0",
    "hexo-generator-tag": "^2.0.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("_config.yml"),
            klyron_template::TemplateEngine::render_static(r#"# Hexo Configuration
title: {{ name }}
subtitle: ""
description: ""
author: Anonymous
language: en
timezone: ""
url: http://localhost:4000
permalink: :year/:month/:day/:title/
theme: default
"#, vars))?;

        std::fs::write(project_dir.join("source/_posts/hello-world.md"),
            klyron_template::TemplateEngine::render_static(r#"---
title: Hello World
date: 2026-01-01 00:00:00
tags:
---

Welcome to {{ name }}! This is your first post.
"#, vars))?;

        std::fs::write(project_dir.join("source/about/index.md"),
            r#"---
title: About
date: 2026-01-01 00:00:00
---

## About

This blog is powered by Hexo.
"#)?;

        std::fs::write(project_dir.join("scaffolds/post.md"),
            r#"---
title: {{ title }}
date: {{ date }}
tags:
---
"#)?;

        std::fs::write(project_dir.join("scaffolds/page.md"),
            r#"---
title: {{ title }}
date: {{ date }}
---
"#)?;

        std::fs::write(project_dir.join("themes/default/layout/layout.ejs"),
            klyron_template::TemplateEngine::render_static(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title><%= config.title %></title>
  <link rel="stylesheet" href="/style.css" />
</head>
<body>
  <header><h1><%= config.title %></h1></header>
  <main><%- body %></main>
</body>
</html>
"#, vars))?;

        std::fs::write(project_dir.join("themes/default/style.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 1rem; }
header { border-bottom: 2px solid #333; margin-bottom: 2rem; padding-bottom: 1rem; }
h1 { font-size: 2em; }
article { margin-bottom: 2rem; }
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\npublic\n.DS_Store\ndb.json\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Hexo blog

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
