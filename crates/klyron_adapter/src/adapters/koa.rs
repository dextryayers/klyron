use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct KoaAdapter;

#[async_trait]
impl FrameworkAdapter for KoaAdapter {
    fn name(&self) -> &'static str { "koa" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"koa\"")).unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["2.15"] }
    fn default_version(&self) -> &'static str { "2.15" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["node", "--watch", "src/index.js"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, _dir: &Path, _opts: BuildOptions) -> Result<()> { Ok(()) }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("npx").args(["jest"]).current_dir(dir).status().await?;
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

    fn external_scaffold_command(&self, _name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        None
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src/routes"))?;
        std::fs::create_dir_all(project_dir.join("src/middleware"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "node --watch src/index.js",
    "start": "node src/index.js",
    "test": "jest",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": { "koa": "^2.15.0", "koa-router": "^12.0.0", "koa-bodyparser": "^4.4.0" },
  "devDependencies": { "jest": "^29.7.0", "eslint": "^9.0.0", "prettier": "^3.4.0" }
}"#, vars))?;

        std::fs::write(project_dir.join("src/index.js"),
            klyron_template::TemplateEngine::render_static(r#"import Koa from 'koa'
import bodyParser from 'koa-bodyparser'
import router from './routes/index.js'

const app = new Koa()
const port = process.env.PORT || 3000

app.use(bodyParser())
app.use(router.routes())

app.listen(port, () => {
  console.log(`{{ name }} running on http://localhost:${port}`)
})
"#, vars))?;

        std::fs::write(project_dir.join("src/routes/index.js"),
            r#"import Router from 'koa-router'
const router = new Router()

router.get('/', (ctx) => { ctx.body = { message: 'Hello World' } })

export default router
"#)?;

        std::fs::write(project_dir.join("src/middleware/error.js"),
            r#"export default async function errorHandler(ctx, next) {
  try { await next() } catch (err) { ctx.status = 500; ctx.body = { error: err.message } }
}
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'
export default [js.configs.recommended, { ignores: ['node_modules'] }]"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Koa API

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
