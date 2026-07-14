use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct TrpcNextAdapter;

#[async_trait]
impl FrameworkAdapter for TrpcNextAdapter {
    fn name(&self) -> &'static str { "trpc_next" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"@trpc/next\""))
            .unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["10.45", "11.0"] }
    fn default_version(&self) -> &'static str { "10.45" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Fullstack }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["next", "dev"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
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
        std::fs::create_dir_all(project_dir.join("src/app/api/trpc/[trpc]"))?;
        std::fs::create_dir_all(project_dir.join("src/server/routers"))?;
        std::fs::create_dir_all(project_dir.join("src/trpc"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "next": "^15.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@trpc/server": "^10.45.0",
    "@trpc/client": "^10.45.0",
    "@trpc/next": "^10.45.0",
    "@trpc/react-query": "^10.45.0",
    "zod": "^3.23.0",
    "@tanstack/react-query": "^5.0.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "vitest": "^2.1.0",
    "eslint": "^9.0.0",
    "prettier": "^3.4.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2023", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "skipLibCheck": true,
    "paths": { "@/*": ["./src/*"] }
  },
  "include": ["next-env.d.ts", "src"]
}"#)?;

        std::fs::write(project_dir.join("next.config.js"),
            r#"/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: { serverActions: true },
}
module.exports = nextConfig"#)?;

        std::fs::write(project_dir.join("src/app/page.tsx"),
            klyron_template::TemplateEngine::render_static(r#""use client"
import { trpc } from '@/trpc/client'

export default function Home() {
  const { data } = trpc.greeting.useQuery({ name: '{{ name }}' })
  return <h1>{data?.message ?? 'Loading...'}</h1>
}
"#, vars))?;

        std::fs::write(project_dir.join("src/app/api/trpc/[trpc]/route.ts"),
            r#"import { fetchRequestHandler } from '@trpc/server/adapters/fetch'
import { appRouter } from '@/server/routers/_app'
import { createContext } from '@/server/context'

const handler = (req: Request) =>
  fetchRequestHandler({ endpoint: '/api/trpc', req, router: appRouter, createContext })

export { handler as GET, handler as POST }"#)?;

        std::fs::write(project_dir.join("src/server/trpc.ts"),
            r#"import { initTRPC } from '@trpc/server'
import { Context } from './context'

const t = initTRPC.context<Context>().create()

export const router = t.router
export const publicProcedure = t.procedure"#)?;

        std::fs::write(project_dir.join("src/server/routers/_app.ts"),
            r#"import { router } from '@/server/trpc'
import { postRouter } from './post'

export const appRouter = router({ post: postRouter })
export type AppRouter = typeof appRouter"#)?;

        std::fs::write(project_dir.join("src/server/routers/post.ts"),
            r#"import { z } from 'zod'
import { publicProcedure, router } from '@/server/trpc'

export const postRouter = router({
  greeting: publicProcedure.input(z.object({ name: z.string() })).query(({ input }) => {
    return { message: `Hello, ${input.name}!` }
  }),
})"#)?;

        std::fs::write(project_dir.join("src/server/context.ts"),
            r#"import { inferAsyncReturnType } from '@trpc/server'
import { CreateNextContextOptions } from '@trpc/server/adapters/next'

export async function createContext(opts: CreateNextContextOptions) {
  return { req: opts.req, res: opts.res }
}

export type Context = inferAsyncReturnType<typeof createContext>"#)?;

        std::fs::write(project_dir.join("src/trpc/server.ts"),
            r#"import { createTRPCNext } from '@trpc/next'
import { httpBatchLink } from '@trpc/client'
import { AppRouter } from '@/server/routers/_app'

export const trpc = createTRPCNext<AppRouter>({
  config() {
    return { links: [httpBatchLink({ url: '/api/trpc' })] }
  },
  ssr: true,
})"#)?;

        std::fs::write(project_dir.join("src/trpc/client.ts"),
            r#""use client"
import { createTRPCReact } from '@trpc/react-query'
import { AppRouter } from '@/server/routers/_app'

export const trpc = createTRPCReact<AppRouter>()"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.next\n.DS_Store\n*.tsbuildinfo\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

tRPC + Next.js project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
