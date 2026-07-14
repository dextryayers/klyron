use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct AstroAdapter;

#[async_trait]
impl FrameworkAdapter for AstroAdapter {
    fn name(&self) -> &'static str { "astro" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("astro.config.mjs").exists() || dir.join("astro.config.ts").exists() || dir.join("astro.config.js").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.0", "5.0", "5.4"] }
    fn default_version(&self) -> &'static str { "5.4" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::StaticSiteGenerator }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["astro", "dev"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["astro", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("npx").args(["astro", "check"]).current_dir(dir).status().await?;
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
        Some(("npm".into(), vec!["create".into(), "astro@latest".into(), name.into()]))
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
        std::fs::create_dir_all(project_dir.join("src/pages"))?;
        std::fs::create_dir_all(project_dir.join("src/layouts"))?;
        std::fs::create_dir_all(project_dir.join("src/components"))?;
        std::fs::create_dir_all(project_dir.join("src/styles"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}", "version": "1.0.0", "private": true, "type": "module",
  "scripts": { "dev": "astro dev", "build": "astro build", "preview": "astro preview", "check": "astro check", "lint": "eslint .", "format": "prettier --write ." },
  "dependencies": { "astro": "^5.4.0" },
  "devDependencies": { "typescript": "^5.7.0", "@astrojs/check": "^0.9.4", "eslint": "^9.20.0", "eslint-plugin-astro": "^1.3.0", "prettier": "^3.5.0", "prettier-plugin-astro": "^0.14.1" }
}"#, vars))?;

        std::fs::write(project_dir.join("astro.config.mjs"),
            r#"import { defineConfig } from 'astro/config'
export default defineConfig({ site: 'https://example.com', server: { port: 4321, host: true } })"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{"compilerOptions":{"target":"ESNext","module":"ESNext","moduleResolution":"bundler","allowImportingTsExtensions":true,"isolatedModules":true,"noEmit":true,"strict":true,"skipLibCheck":true},"include":["src"]}"#)?;

        std::fs::write(project_dir.join("src/pages/index.astro"),
            klyron_template::TemplateEngine::render_static(r#"---
import Layout from '../layouts/Layout.astro'
---
<Layout title="{{ name }}"><main><h1>Welcome to {{ name }}</h1></main></Layout>"#, vars))?;

        std::fs::write(project_dir.join("src/layouts/Layout.astro"),
            r#"---
interface Props { title: string }
const { title } = Astro.props
---
<!doctype html><html lang="en"><head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width" /><title>{title}</title><link rel="icon" type="image/svg+xml" href="/favicon.svg" /></head><body><slot /></body></html>"#)?;

        std::fs::write(project_dir.join("src/components/Header.astro"), r#"<header><nav><a href="/">Home</a></nav></header>"#)?;
        std::fs::write(project_dir.join("src/styles/global.css"), r#"* { margin: 0; padding: 0; box-sizing: border-box; }\nbody { font-family: system-ui, sans-serif; }"#)?;
        std::fs::write(project_dir.join("public/robots.txt"), "User-agent: *\nAllow: /\n")?;
        std::fs::write(project_dir.join("public/favicon.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><circle cx="16" cy="16" r="16" fill="#333"/></svg>"##)?;
        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"), r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100, "plugins": ["prettier-plugin-astro"]}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'
import astro from 'eslint-plugin-astro'
export default [js.configs.recommended, ...astro.configs.recommended, { ignores: ['dist'] }]"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}\nAstro project\nnpm run dev"#, vars))?;

        Ok(())
    }
}
