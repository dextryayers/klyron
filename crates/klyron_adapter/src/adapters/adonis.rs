use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct AdonisAdapter;

#[async_trait]
impl FrameworkAdapter for AdonisAdapter {
    fn name(&self) -> &'static str { "adonis" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join(".adonisrc.json").exists() || dir.join(".adonisrc.yaml").exists() || dir.join(".adonisrc.ts").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["6.0"] }
    fn default_version(&self) -> &'static str { "6.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Backend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("node");
        cmd.args(["ace", "serve", "--watch"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("node").args(["ace", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("node").args(["ace", "test"]).current_dir(dir).status().await?;
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
        Some(("npm".into(), vec!["create".into(), "adonisjs@latest".into(), name.into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("start"))?;
        std::fs::create_dir_all(project_dir.join("app/Controllers/Http"))?;
        std::fs::create_dir_all(project_dir.join("app/Middleware"))?;
        std::fs::create_dir_all(project_dir.join("resources/views"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join(".adonisrc.json"),
            r#"{
  "typescript": true,
  "directories": { "controllers": "app/Controllers", "middleware": "app/Middleware" },
  "commands": ["@adonisjs/core/build/commands"]
}"#)?;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "node ace serve --watch",
    "build": "node ace build",
    "start": "node bin/server.js",
    "test": "node ace test",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "@adonisjs/core": "^6.0.0",
    "@adonisjs/lucid": "^20.0.0",
    "@adonisjs/session": "^7.0.0",
    "@adonisjs/shield": "^8.0.0",
    "@adonisjs/static": "^2.0.0",
    "@adonisjs/view": "^7.0.0",
    "edge.js": "^6.0.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "eslint": "^9.0.0",
    "@adonisjs/eslint-config": "^1.0.0",
    "prettier": "^3.4.0",
    "pino-pretty": "^13.0.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true,
    "isolatedModules": true,
    "experimentalDecorators": true,
    "emitDecoratorMetadata": true
  },
  "include": ["start", "app", "config"]
}"#)?;

        std::fs::write(project_dir.join("start/routes.ts"),
            r#"import router from '@adonisjs/core/services/router'

router.get('/', async () => {
  return { hello: 'world' }
})
"#)?;

        std::fs::write(project_dir.join("start/kernel.ts"),
            r#"import server from '@adonisjs/core/services/server'
server.use(['@adonisjs/session', '@adonisjs/shield'])
"#)?;

        std::fs::write(project_dir.join("env.ts"),
            r#"import Env from '@adonisjs/core/services/env'

export default Env.rules({
  PORT: Env.schema.number(),
  HOST: Env.schema.string(),
  APP_KEY: Env.schema.string(),
  NODE_ENV: Env.schema.enum(['development', 'production'] as const),
})
"#)?;

        std::fs::write(project_dir.join("app/Controllers/Http/HelloController.ts"),
            r#"import { HttpContext } from '@adonisjs/core/http'

export default class HelloController {
  async index({ response }: HttpContext) {
    return response.json({ message: 'Hello World' })
  }
}
"#)?;

        std::fs::write(project_dir.join("app/Middleware/Auth.ts"),
            r#"import { HttpContext } from '@adonisjs/core/http'

export default class AuthMiddleware {
  async handle(ctx: HttpContext, next: () => Promise<void>) {
    await next()
  }
}
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\nbuild\n.DS_Store\n*.log\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import adonis from '@adonisjs/eslint-config'
export default adonis.configs.recommended
"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

AdonisJS project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
