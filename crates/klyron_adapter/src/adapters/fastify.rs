use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct FastifyAdapter;

#[async_trait]
impl FrameworkAdapter for FastifyAdapter {
    fn name(&self) -> &'static str { "fastify" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"fastify\"")).unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.28", "5.0", "5.2"] }
    fn default_version(&self) -> &'static str { "5.2" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["node", "--watch", "src/index.js"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("npx").args(["tap"]).current_dir(dir).status().await?;
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
        if options.template_vars.get("external").map(|s| s == "true").unwrap_or(false) {
            let status = std::process::Command::new("npx")
                .args(["create-fastify@latest", name]).current_dir(&options.dir).status()?;
            if !status.success() { anyhow::bail!("External scaffolding failed"); }
            return Ok(());
        }
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src/routes"))?;
        std::fs::create_dir_all(project_dir.join("src/plugins"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render(r#"{
  "name": "{{ name }}", "version": "1.0.0", "private": true, "type": "module",
  "scripts": { "dev": "node --watch src/index.js", "start": "node src/index.js", "test": "tap", "lint": "eslint .", "format": "prettier --write ." },
  "dependencies": { "fastify": "^5.2.0", "@fastify/cors": "^11.0.0" },
  "devDependencies": { "tap": "^21.0.0", "eslint": "^9.20.0", "prettier": "^3.5.0" }
}"#, vars))?;

        std::fs::write(project_dir.join("src/index.js"),
            klyron_template::TemplateEngine::render(r#"import Fastify from 'fastify'
import routes from './routes/index.js'
const app = Fastify({ logger: true })
const port = process.env.PORT || 3000
app.register(routes)
app.listen({ port }, () => console.log(`{{ name }} running on http://localhost:${port}`))
"#, vars))?;

        std::fs::write(project_dir.join("src/routes/index.js"),
            r#"export default async function (fastify, opts) { fastify.get('/', async (request, reply) => { return { message: 'Hello World' } }) }"#)?;

        std::fs::write(project_dir.join("src/plugins/support.js"),
            r#"import fp from 'fastify-plugin'
export default fp(async function (fastify, opts) { fastify.decorate('support', { name: 'fastify-support' }) })"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"), r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"), r#"import js from '@eslint/js'\nexport default [js.configs.recommended, { ignores: ['node_modules'] }]"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render(r#"# {{ name }}\nFastify API\nnpm run dev"#, vars))?;

        Ok(())
    }
}
