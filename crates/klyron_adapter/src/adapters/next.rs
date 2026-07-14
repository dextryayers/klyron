use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct NextAdapter;

#[async_trait]
impl FrameworkAdapter for NextAdapter {
    fn name(&self) -> &'static str { "next" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("next.config.ts").exists() || dir.join("next.config.js").exists() || dir.join("next.config.mjs").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["14.0", "15.0", "15.1", "15.2", "15.3"] }
    fn default_version(&self) -> &'static str { "15.3" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Fullstack }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["next", "dev"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); } else { cmd.env("PORT", "3000"); }
        cmd.env("NEXT_TELEMETRY_DISABLED", "1");
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["next", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("npx").args(["vitest", "run"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("npx").args(["next", "lint"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        tokio::process::Command::new("npx")
            .args(if write { ["prettier", "--write", "."] } else { ["prettier", "--check", "."] })
            .current_dir(dir).status().await?;
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("npx".into(), vec!["create-next-app@latest".into(), name.into()]))
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
        std::fs::create_dir_all(project_dir.join("app"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;
        std::fs::create_dir_all(project_dir.join("components"))?;
        std::fs::create_dir_all(project_dir.join("lib"))?;
        std::fs::create_dir_all(project_dir.join("styles"))?;
        std::fs::create_dir_all(project_dir.join("types"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}", "version": "1.0.0", "private": true,
  "scripts": { "dev": "next dev", "build": "next build", "start": "next start", "lint": "next lint", "format": "prettier --write .", "test": "vitest run" },
  "dependencies": { "next": "^15.3.0", "react": "^19.1.0", "react-dom": "^19.1.0" },
  "devDependencies": { "typescript": "^5.7.0", "@types/node": "^22.13.0", "@types/react": "^19.1.0", "@types/react-dom": "^19.1.0", "tailwindcss": "^4.0.0", "postcss": "^8.5.0", "autoprefixer": "^10.4.20", "eslint": "^9.20.0", "eslint-config-next": "^15.3.0", "prettier": "^3.5.0", "vitest": "^3.0.0" }
}"#, vars))?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{"compilerOptions":{"target":"ES2017","lib":["dom","dom.iterable","esnext"],"allowJs":true,"skipLibCheck":true,"strict":true,"noEmit":true,"esModuleInterop":true,"module":"esnext","moduleResolution":"bundler","resolveJsonModule":true,"isolatedModules":true,"jsx":"preserve","incremental":true,"plugins":[{"name":"next"}],"paths":{"@/*":["./*"]}},"include":["next-env.d.ts","**/*.ts","**/*.tsx",".next/types/**/*.ts"],"exclude":["node_modules"]}"#)?;

        std::fs::write(project_dir.join("next.config.ts"),
            r#"import type { NextConfig } from 'next'
const nextConfig: NextConfig = { experimental: { serverActions: true } }
export default nextConfig"#)?;

        std::fs::write(project_dir.join("tailwind.config.ts"),
            r#"import type { Config } from 'tailwindcss'
const config: Config = { content: ['./app/**/*.{ts,tsx}', './components/**/*.{ts,tsx}'], theme: { extend: {} }, plugins: [] }
export default config"#)?;

        std::fs::write(project_dir.join("postcss.config.js"),
            r#"module.exports = { plugins: { tailwindcss: {}, autoprefixer: {} } }"#)?;

        std::fs::write(project_dir.join("app/layout.tsx"),
            r#"import type { Metadata } from 'next'
import './globals.css'
export const metadata: Metadata = { title: 'My App', description: 'Generated by Klyron' }
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return <html lang="en"><body>{children}</body></html>
}"#)?;

        std::fs::write(project_dir.join("app/page.tsx"),
            klyron_template::TemplateEngine::render_static(r#"export default function Home() {
  return <main><h1>Welcome to {{ name }}</h1></main>
}"#, vars))?;

        std::fs::write(project_dir.join("app/globals.css"), r#"@tailwind base; @tailwind components; @tailwind utilities;
:root { --foreground: #000; --background: #fff; }"#)?;

        std::fs::write(project_dir.join("app/not-found.tsx"), r#"export default function NotFound() { return <div><h1>404 - Page Not Found</h1></div> }"#)?;

        std::fs::write(project_dir.join("app/error.tsx"), r##""use client"
export default function Error({ error, reset }: { error: Error & { digest?: string }; reset: () => void }) {
  return <div><h2>Something went wrong!</h2><button onClick={() => reset()}>Try again</button></div>
}"##)?;

        std::fs::write(project_dir.join("app/loading.tsx"), r#"export default function Loading() { return <div>Loading...</div> }"#)?;
        std::fs::write(project_dir.join("next-env.d.ts"), r#"/// <reference types="next" />\n/// <reference types="next/image-types/global" />"#)?;
        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.next\n*.tsbuildinfo\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"), r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;

        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import { dirname } from 'path'
import { fileURLToPath } from 'url'
import { FlatCompat } from '@eslint/eslintrc'
const __filename = fileURLToPath(import.meta.url); const __dirname = dirname(__filename)
const compat = new FlatCompat({ baseDirectory: __dirname })
const eslintConfig = [...compat.extends('next/core-web-vitals')]
export default eslintConfig"#)?;

        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}\nNext.js App Router\nnpm run dev"#, vars))?;

        Ok(())
    }
}
