use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct QwikAdapter;

#[async_trait]
impl FrameworkAdapter for QwikAdapter {
    fn name(&self) -> &'static str { "qwik" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("qwik.config.ts").exists() || dir.join("qwik.config.js").exists()
            || dir.join("package.json").exists().then(|| {
                std::fs::read_to_string(dir.join("package.json")).ok()
                    .map(|c| c.contains("\"@builder.io/qwik\"")).unwrap_or(false)
            }).unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["1.9"] }
    fn default_version(&self) -> &'static str { "1.9" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Frontend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["qwik", "dev"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["qwik", "build"]).current_dir(dir).status().await?;
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

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src/routes"))?;
        std::fs::create_dir_all(project_dir.join("src/components/router-head"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "qwik dev",
    "build": "qwik build",
    "preview": "qwik preview",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "@builder.io/qwik": "^1.9.0",
    "@builder.io/qwik-city": "^1.9.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "vite": "^6.0.0",
    "vitest": "^2.1.0",
    "eslint": "^9.0.0",
    "eslint-plugin-qwik": "^1.9.0",
    "prettier": "^3.4.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("qwik.config.ts"),
            r#"import { defineConfig } from '@builder.io/qwik-city'

export default defineConfig({
  server: { port: 5173, host: true },
})"#)?;

        std::fs::write(project_dir.join("vite.config.ts"),
            r#"import { defineConfig } from 'vite'
import { qwikVite } from '@builder.io/qwik/optimizer'
import { qwikCity } from '@builder.io/qwik-city/vite'

export default defineConfig({
  plugins: [qwikCity(), qwikVite()],
})"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "jsxImportSource": "@builder.io/qwik",
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}"#)?;

        std::fs::write(project_dir.join("src/root.tsx"),
            r#"import { component$ } from '@builder.io/qwik'

export default component$(() => {
  return <p>Hello Qwik</p>
})
"#)?;

        std::fs::write(project_dir.join("src/routes/index.tsx"),
            klyron_template::TemplateEngine::render(r#"import { component$ } from '@builder.io/qwik'
import type { DocumentHead } from '@builder.io/qwik-city'

export default component$(() => {
  return <h1>Welcome to {{ name }}</h1>
})

export const head: DocumentHead = { title: '{{ name }}' }
"#, vars))?;

        std::fs::write(project_dir.join("src/routes/layout.tsx"),
            r#"import { component$, Slot } from '@builder.io/qwik'

export default component$(() => {
  return <Slot />
})
"#)?;

        std::fs::write(project_dir.join("src/components/router-head/router-head.tsx"),
            r#"import { component$ } from '@builder.io/qwik'

export default component$(() => {
  return <head><meta charSet="utf-8" /></head>
})
"#)?;

        std::fs::write(project_dir.join("tailwind.config.js"),
            r#"export default { content: ['./src/**/*.{ts,tsx}'], theme: { extend: {} }, plugins: [] }"#)?;

        std::fs::write(project_dir.join("postcss.config.js"),
            r#"export default { plugins: { tailwindcss: {}, autoprefixer: {} } }"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\nserver\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'
import qwik from 'eslint-plugin-qwik'

export default [
  js.configs.recommended,
  { plugins: { qwik }, rules: { 'qwik/valid-lexical-scope': 'error' }, ignores: ['dist', 'server'] },
]"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render(r#"# {{ name }}

Qwik project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
