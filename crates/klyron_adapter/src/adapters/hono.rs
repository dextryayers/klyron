use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct HonoAdapter;

#[async_trait]
impl FrameworkAdapter for HonoAdapter {
    fn name(&self) -> &'static str { "hono" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"hono\"")).unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.6", "4.7"] }
    fn default_version(&self) -> &'static str { "4.7" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["tsx", "watch", "src/index.ts"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

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
        Some(("npm".into(), vec!["create".into(), "hono@latest".into(), name.into()]))
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

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}", "version": "1.0.0", "private": true, "type": "module",
  "scripts": { "dev": "tsx watch src/index.ts", "start": "tsx src/index.ts", "test": "vitest run", "lint": "eslint .", "format": "prettier --write ." },
  "dependencies": { "hono": "^4.7.0" },
  "devDependencies": { "typescript": "^5.7.0", "tsx": "^4.19.0", "vitest": "^3.0.0", "eslint": "^9.20.0", "prettier": "^3.5.0" }
}"#, vars))?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{"compilerOptions":{"target":"ES2022","module":"ESNext","moduleResolution":"bundler","strict":true,"skipLibCheck":true,"noEmit":true,"isolatedModules":true},"include":["src"]}"#)?;

        std::fs::write(project_dir.join("src/index.ts"),
            klyron_template::TemplateEngine::render_static(r#"import { Hono } from 'hono'
import routes from './routes/index.js'
const app = new Hono()
app.route('/', routes)
export default { port: process.env.PORT || 3000, fetch: app.fetch }
console.log(`{{ name }} running on http://localhost:${process.env.PORT || 3000}`)
"#, vars))?;

        std::fs::write(project_dir.join("src/routes/index.ts"),
            r#"import { Hono } from 'hono'
const router = new Hono()
router.get('/', (c) => c.json({ message: 'Hello World' }))
export default router"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"), r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'\nimport tseslint from 'typescript-eslint'\nexport default tseslint.config(js.configs.recommended, ...tseslint.configs.recommended, { ignores: ['node_modules'] })"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}\nHono API\nnpm run dev"#, vars))?;

        Ok(())
    }
}
