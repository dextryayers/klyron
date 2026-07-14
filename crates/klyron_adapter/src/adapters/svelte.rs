use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct SvelteAdapter;

#[async_trait]
impl FrameworkAdapter for SvelteAdapter {
    fn name(&self) -> &'static str { "svelte" }

    fn detect(&self, dir: &Path) -> bool {
        let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
        let has_svelte = dir.join("package.json").exists().then(|| {
            std::fs::read_to_string(dir.join("package.json")).ok()
                .map(|c| c.contains("\"svelte\"") && !c.contains("\"@sveltejs/kit\""))
                .unwrap_or(false)
        }).unwrap_or(false);
        has_vite && has_svelte
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.2", "5.0"] }
    fn default_version(&self) -> &'static str { "5.0" }
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
        tokio::process::Command::new("npx").args(["vitest", "run"]).current_dir(dir).status().await?;
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
        Some(("npx".into(), vec!["create-vite@latest".into(), name.into(), "--template".into(), "svelte-ts".into()]))
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
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": { "svelte": "^5.0.0" },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^5.0.0",
    "vite": "^6.0.0",
    "vitest": "^2.1.0",
    "eslint": "^9.0.0",
    "eslint-plugin-svelte": "^2.0.0",
    "prettier": "^3.4.0",
    "prettier-plugin-svelte": "^3.0.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("vite.config.ts"),
            r#"import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
  server: { port: 5173, host: true },
})"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "noEmit": true,
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}"#)?;

        std::fs::write(project_dir.join("index.html"),
            klyron_template::TemplateEngine::render_static(r#"<!doctype html>
<html lang="en">
  <head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>{{ name }}</title></head>
  <body><div id="app"></div><script type="module" src="/src/main.js"></script></body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("src/main.js"),
            r#"import App from './App.svelte'
import { mount } from 'svelte'

const app = mount(App, { target: document.getElementById('app')! })

export default app
"#)?;

        std::fs::write(project_dir.join("src/App.svelte"),
            klyron_template::TemplateEngine::render_static(r#"<script>
  let name = '{{ name }}'
</script>

<h1>Welcome to {name}</h1>
"#, vars))?;

        std::fs::write(project_dir.join("src/app.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; min-height: 100vh; }
#app { max-width: 1280px; margin: 0 auto; padding: 2rem; }
"#)?;

        std::fs::write(project_dir.join("src/vite-env.d.ts"),
            r#"/// <reference types="svelte" />
/// <reference types="vite/client" />"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100, "plugins": ["prettier-plugin-svelte"]}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import svelte from 'eslint-plugin-svelte'

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...svelte.configs.recommended,
  { ignores: ['dist'] },
)"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Svelte project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
