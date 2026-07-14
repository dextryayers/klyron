use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct ExpressAdapter;

#[async_trait]
impl FrameworkAdapter for ExpressAdapter {
    fn name(&self) -> &'static str { "express" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"express\"")).unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["4.18", "5.0", "5.1"] }
    fn default_version(&self) -> &'static str { "5.1" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        if dir.join("tsconfig.json").exists() {
            cmd.args(["tsx", "watch", "src/index.ts"]);
        } else {
            cmd.args(["node", "--watch", "src/index.js"]);
        }
        cmd.current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        if dir.join("tsconfig.json").exists() {
            tokio::process::Command::new("npx").args(["tsc"]).current_dir(dir).status().await?;
        }
        Ok(())
    }

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

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        if options.template_vars.get("external").map(|s| s == "true").unwrap_or(false) {
            let status = std::process::Command::new("npx")
                .args(["express-generator@latest", "--no-view", name]).current_dir(&options.dir).status()?;
            if !status.success() { anyhow::bail!("External scaffolding failed"); }
            return Ok(());
        }
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src/routes"))?;
        std::fs::create_dir_all(project_dir.join("src/middleware"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render(r#"{
  "name": "{{ name }}", "version": "1.0.0", "private": true, "type": "module",
  "scripts": { "dev": "node --watch src/index.js", "start": "node src/index.js", "test": "jest", "lint": "eslint .", "format": "prettier --write ." },
  "dependencies": { "express": "^5.1.0", "cors": "^2.8.5", "morgan": "^1.10.0" },
  "devDependencies": { "jest": "^30.0.0", "eslint": "^9.20.0", "prettier": "^3.5.0" }
}"#, vars))?;

        std::fs::write(project_dir.join("src/index.js"),
            klyron_template::TemplateEngine::render(r#"import express from 'express'
import cors from 'cors'
import morgan from 'morgan'
import routes from './routes/index.js'
const app = express()
const port = process.env.PORT || 3000
app.use(cors()); app.use(morgan('dev')); app.use(express.json())
app.use('/', routes)
app.listen(port, () => console.log(`{{ name }} running on http://localhost:${port}`))
"#, vars))?;

        std::fs::write(project_dir.join("src/routes/index.js"),
            r#"import { Router } from 'express'
const router = Router()
router.get('/', (req, res) => res.json({ message: 'Hello World' }))
export default router"#)?;

        std::fs::write(project_dir.join("src/middleware/error.js"),
            r#"export function errorHandler(err, req, res, next) { console.error(err.stack); res.status(500).json({ error: err.message }) }"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"), r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"), r#"import js from '@eslint/js'\nexport default [js.configs.recommended, { ignores: ['node_modules'] }]"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render(r#"# {{ name }}\nExpress.js API\nnpm run dev"#, vars))?;

        Ok(())
    }
}
