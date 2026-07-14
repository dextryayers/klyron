use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct TrpcAdapter;

#[async_trait]
impl FrameworkAdapter for TrpcAdapter {
    fn name(&self) -> &'static str { "trpc" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"@trpc/server\"")).unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["10.45", "11.0"] }
    fn default_version(&self) -> &'static str { "10.45" }
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

    fn external_scaffold_command(&self, _name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        None
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "start": "tsx src/index.ts",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "@trpc/server": "^10.45.0",
    "@trpc/client": "^10.45.0",
    "zod": "^3.23.0"
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
    "isolatedModules": true
  },
  "include": ["src"]
}"#)?;

        std::fs::write(project_dir.join("src/index.ts"),
            klyron_template::TemplateEngine::render_static(r#"import { createHTTPServer } from '@trpc/server/adapters/standalone'
import { appRouter } from './router'
import { createContext } from './context'

const server = createHTTPServer({
  router: appRouter,
  createContext,
})

server.listen(3000)
console.log(`{{ name }} running on http://localhost:3000`)
"#, vars))?;

        std::fs::write(project_dir.join("src/router.ts"),
            r#"import { z } from 'zod'
import { publicProcedure, router } from '@trpc/server'

export const appRouter = router({
  hello: publicProcedure.input(z.object({ name: z.string() })).query(({ input }) => {
    return { greeting: `Hello, ${input.name}!` }
  }),
})

export type AppRouter = typeof appRouter
"#)?;

        std::fs::write(project_dir.join("src/context.ts"),
            r#"import { inferAsyncReturnType } from '@trpc/server'
import { CreateHTTPContextOptions } from '@trpc/server/adapters/standalone'

export async function createContext(opts: CreateHTTPContextOptions) {
  return { req: opts.req, res: opts.res }
}

export type Context = inferAsyncReturnType<typeof createContext>
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'
import tseslint from 'typescript-eslint'
export default tseslint.config(js.configs.recommended, ...tseslint.configs.recommended, { ignores: ['node_modules'] })"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

tRPC API

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
