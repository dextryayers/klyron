use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct Svelte5Adapter;

#[async_trait]
impl FrameworkAdapter for Svelte5Adapter {
    fn name(&self) -> &'static str { "svelte5" }

    fn detect(&self, dir: &Path) -> bool {
        let has_config = dir.join("svelte.config.js").exists() || dir.join("svelte.config.mjs").exists() || dir.join("svelte.config.ts").exists();
        let has_plugin = dir.join("package.json").exists().then(|| {
            std::fs::read_to_string(dir.join("package.json")).ok()
                .map(|c| c.contains("\"@sveltejs/vite-plugin-svelte\""))
                .unwrap_or(false)
        }).unwrap_or(false);
        has_config && has_plugin
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["5.0"] }
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
        std::fs::create_dir_all(project_dir.join("src/lib"))?;

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

        std::fs::write(project_dir.join("svelte.config.js"),
            r#"import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

export default {
  preprocess: vitePreprocess(),
}"#)?;

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
  <body><div id="app"></div><script type="module" src="/src/main.ts"></script></body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("src/main.ts"),
            r#"import { mount } from 'svelte'
import App from './App.svelte'
import './app.css'

const app = mount(App, { target: document.getElementById('app')! })
export default app
"#)?;

        std::fs::write(project_dir.join("src/App.svelte"),
            klyron_template::TemplateEngine::render_static(r#"<script lang="ts">
  let name = $state('{{ name }}')
  let count = $state(0)

  let doubled = $derived(count * 2)

  $effect(() => {
    document.title = `Count: ${count}`
  })

  function increment() {
    count += 1
  }
</script>

<h1>Welcome to {name}</h1>
<p>Count: {count}</p>
<p>Doubled: {doubled}</p>
<button onclick={increment}>Increment</button>
"#, vars))?;

        std::fs::write(project_dir.join("src/app.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; min-height: 100vh; }
#app { max-width: 1280px; margin: 0 auto; padding: 2rem; text-align: center; }
button { border-radius: 8px; border: 1px solid #ccc; padding: 0.6em 1.2em; font-size: 1em; cursor: pointer; }
button:hover { border-color: #646cff; }
"#)?;

        std::fs::write(project_dir.join("src/vite-env.d.ts"),
            r#"/// <reference types="svelte" />
/// <reference types="vite/client" />"#)?;

        std::fs::write(project_dir.join("src/lib/counter.svelte"),
            r#"<script lang="ts">
  let { initial = 0 }: { initial?: number } = $props()
  let count = $state(initial)
  const increment = () => count++
</script>

<button onclick={increment}>Count: {count}</button>
"#)?;

        std::fs::write(project_dir.join("src/lib/user.svelte"),
            r#"<script lang="ts">
  interface User {
    name: string
    email: string
  }
  let { user }: { user: User } = $props()
</script>

<div>
  <p>Name: {user.name}</p>
  <p>Email: {user.email}</p>
</div>
"#)?;

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

Svelte 5 project with runes

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
