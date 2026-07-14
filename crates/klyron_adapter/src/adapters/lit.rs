use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct LitAdapter;

#[async_trait]
impl FrameworkAdapter for LitAdapter {
    fn name(&self) -> &'static str { "lit" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"lit\"")).unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["3.2"] }
    fn default_version(&self) -> &'static str { "3.2" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Frontend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.arg("vite").current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["vite", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("npx").args(["web-test-runner"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("npx").args(["eslint", "."]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        tokio::process::Command::new("npx")
            .args(if write { ["prettier", "--write", "."] } else { ["prettier", "--check", "."] })
            .current_dir(dir).status().await?;
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("npx".into(), vec!["create-vite@latest".into(), name.into(), "--template".into(), "lit-ts".into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "test": "web-test-runner",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": { "lit": "^3.2.0" },
  "devDependencies": {
    "vite": "^6.0.0",
    "typescript": "^5.6.0",
    "@web/test-runner": "^0.19.0",
    "eslint": "^9.0.0",
    "prettier": "^3.4.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("vite.config.ts"),
            r#"import { defineConfig } from 'vite'
export default defineConfig({ server: { port: 5173, host: true } })"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ES2021",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "noEmit": true,
    "experimentalDecorators": true,
    "useDefineForClassFields": false,
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}"#)?;

        std::fs::write(project_dir.join("index.html"),
            klyron_template::TemplateEngine::render_static(r#"<!doctype html>
<html lang="en">
  <head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>{{ name }}</title></head>
  <body><my-app></my-app><script type="module" src="/src/my-app.ts"></script></body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("src/my-app.ts"),
            r#"import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'

@customElement('my-app')
export class MyApp extends LitElement {
  render() {
    return html`<my-element></my-element>`
  }
}
"#)?;

        std::fs::write(project_dir.join("src/my-element.ts"),
            r#"import { LitElement, html, css } from 'lit'
import { customElement, property } from 'lit/decorators.js'

@customElement('my-element')
export class MyElement extends LitElement {
  static styles = css`p { color: blue }`

  @property() name = 'World'

  render() {
    return html`<p>Hello, ${this.name}!</p>`
  }
}
"#)?;

        std::fs::write(project_dir.join("src/vite-env.d.ts"),
            r#"/// <reference types="vite/client" />"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'
import tseslint from 'typescript-eslint'
export default tseslint.config(js.configs.recommended, ...tseslint.configs.recommended, { ignores: ['dist'] })"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Lit project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
