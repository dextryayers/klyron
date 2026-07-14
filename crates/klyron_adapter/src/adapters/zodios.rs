use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct ZodiosAdapter;

#[async_trait]
impl FrameworkAdapter for ZodiosAdapter {
    fn name(&self) -> &'static str { "zodios" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"zodios\""))
            .unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["10.0"] }
    fn default_version(&self) -> &'static str { "10.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::ApiFramework }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["tsx", "watch", "src/server.ts"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["tsc"]).current_dir(dir).status().await?;
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
        std::fs::create_dir_all(project_dir.join("src/api"))?;
        std::fs::create_dir_all(project_dir.join("src/schemas"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/server.ts",
    "build": "tsc",
    "start": "tsx src/server.ts",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "zodios": "^10.0.0",
    "zod": "^3.23.0",
    "express": "^4.21.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "tsx": "^4.19.0",
    "vitest": "^2.1.0",
    "eslint": "^9.0.0",
    "prettier": "^3.4.0"
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
    "esModuleInterop": true
  },
  "include": ["src"]
}"#)?;

        std::fs::write(project_dir.join("src/index.ts"),
            klyron_template::TemplateEngine::render_static(r#"import { createApiClient } from 'zodios'
import { userApi } from './api/users'
import { postApi } from './api/posts'

export const client = createApiClient('http://localhost:3000', ...userApi, ...postApi)

console.log('{{ name }} API client ready')
"#, vars))?;

        std::fs::write(project_dir.join("src/server.ts"),
            klyron_template::TemplateEngine::render_static(r#"import express from 'express'
import { createApiRouter } from 'zodios'
import { userApi } from './api/users'
import { postApi } from './api/posts'

const app = express()
app.use(express.json())

app.use('/api', createApiRouter(userApi))
app.use('/api', createApiRouter(postApi))

app.listen(3000, () => console.log('{{ name }} running on http://localhost:3000'))
"#, vars))?;

        std::fs::write(project_dir.join("src/client.ts"),
            r#"import { createApiClient } from 'zodios'
import { userApi } from './api/users'
import { postApi } from './api/posts'

export const apiClient = createApiClient('/api', ...userApi, ...postApi)
"#)?;

        std::fs::write(project_dir.join("src/api/users.ts"),
            r#"import { z } from 'zod'
import { apiBuilder } from 'zodios'

const userSchema = z.object({ id: z.number(), name: z.string(), email: z.string().email() })

export const userApi = apiBuilder({
  method: 'get',
  path: '/users',
  response: z.array(userSchema),
})"#)?;

        std::fs::write(project_dir.join("src/api/posts.ts"),
            r#"import { z } from 'zod'
import { apiBuilder } from 'zodios'

const postSchema = z.object({ id: z.number(), title: z.string(), body: z.string() })

export const postApi = apiBuilder({
  method: 'get',
  path: '/posts',
  response: z.array(postSchema),
})"#)?;

        std::fs::write(project_dir.join("src/schemas/index.ts"),
            r#"import { z } from 'zod'

export const UserSchema = z.object({
  id: z.number(),
  name: z.string().min(1),
  email: z.string().email(),
})

export const PostSchema = z.object({
  id: z.number(),
  title: z.string().min(1),
  body: z.string(),
})

export type User = z.infer<typeof UserSchema>
export type Post = z.infer<typeof PostSchema>
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Zodios API project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
