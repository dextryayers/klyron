use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct SvelteKitAdapter;

#[async_trait]
impl FrameworkAdapter for SvelteKitAdapter {
    fn name(&self) -> &'static str { "sveltekit" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("svelte.config.js").exists() || dir.join("svelte.config.ts").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["2.0"] }
    fn default_version(&self) -> &'static str { "2.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Fullstack }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["vite", "dev"]).current_dir(dir);
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
        Some(("npm".into(), vec!["create".into(), "svelte@latest".into(), name.into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        if options.external {
            if let Some((cmd, args)) = self.external_scaffold_command(name, options.version.as_deref()) {
                let status = std::process::Command::new(&cmd).args(&args).current_dir(&options.dir).status()?;
                if !status.success() { anyhow::bail!("External scaffolding failed"); }
                return Ok(());
            }
        }
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src/routes"))?;
        std::fs::create_dir_all(project_dir.join("src/lib"))?;
        std::fs::create_dir_all(project_dir.join("src/params"))?;
        std::fs::create_dir_all(project_dir.join("static"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "@sveltejs/kit": "^2.0.0",
    "@sveltejs/adapter-auto": "^3.0.0",
    "svelte": "^5.0.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "vite": "^6.0.0",
    "vitest": "^2.1.0",
    "eslint": "^9.0.0",
    "eslint-plugin-svelte": "^2.0.0",
    "prettier": "^3.4.0",
    "prettier-plugin-svelte": "^3.0.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("svelte.config.js"),
            r#"import adapter from '@sveltejs/adapter-auto'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

export default {
  kit: { adapter: adapter() },
  preprocess: vitePreprocess(),
}"#)?;

        std::fs::write(project_dir.join("vite.config.ts"),
            r#"import { sveltekit } from '@sveltejs/kit/vite'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [sveltekit()],
})"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": { "strict": true }
}"#)?;

        std::fs::write(project_dir.join("src/app.html"),
            klyron_template::TemplateEngine::render_static(r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <link rel="icon" href="%sveltekit.assets%/favicon.png" />
    <meta name="viewport" content="width=device-width" />
    %sveltekit.head%
  </head>
  <body>
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
"#, vars))?;

        std::fs::write(project_dir.join("src/routes/+layout.svelte"),
            r#"<script lang="ts">
  import './app.css'
</script>

<slot />
"#)?;

        std::fs::write(project_dir.join("src/routes/+page.svelte"),
            klyron_template::TemplateEngine::render_static(r#"<script lang="ts">
  let name = '{{ name }}'
</script>

<h1>Welcome to {name}</h1>
"#, vars))?;

        std::fs::write(project_dir.join("src/routes/about/+page.svelte"),
            r#"<h1>About</h1>
"#)?;

        std::fs::write(project_dir.join("src/app.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: system-ui, sans-serif; }
body { min-height: 100vh; }
"#)?;

        std::fs::write(project_dir.join("src/app.d.ts"),
            r#"/// <reference types="@sveltejs/kit" />

declare namespace App {}
"#)?;

        std::fs::write(project_dir.join("tailwind.config.js"),
            r#"export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: { extend: {} },
  plugins: [],
}"#)?;

        std::fs::write(project_dir.join("postcss.config.js"),
            r#"export default { plugins: { tailwindcss: {}, autoprefixer: {} } }"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.svelte-kit\nbuild\n.DS_Store\n")?;
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
  { ignores: ['.svelte-kit', 'build'] },
)"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

SvelteKit project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
