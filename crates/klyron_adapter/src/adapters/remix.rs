use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct RemixAdapter;

#[async_trait]
impl FrameworkAdapter for RemixAdapter {
    fn name(&self) -> &'static str { "remix" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("remix.config.ts").exists() || dir.join("remix.config.js").exists() || dir.join("vite.config.ts").exists()
            .then(|| {
                std::fs::read_to_string(dir.join("vite.config.ts")).ok()
                    .map(|c| c.contains("remix")).unwrap_or(false)
            }).unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["2.14"] }
    fn default_version(&self) -> &'static str { "2.14" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Fullstack }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["remix", "dev"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["remix", "build"]).current_dir(dir).status().await?;
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

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("npx".into(), vec!["create-remix@latest".into(), name.into()]))
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
        std::fs::create_dir_all(project_dir.join("app/routes"))?;
        std::fs::create_dir_all(project_dir.join("app/components"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;
        std::fs::create_dir_all(project_dir.join("styles"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "remix dev",
    "build": "remix build",
    "start": "remix-serve build/server/index.js",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "@remix-run/node": "^2.14.0",
    "@remix-run/react": "^2.14.0",
    "@remix-run/serve": "^2.14.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "isbot": "^5.0.0"
  },
  "devDependencies": {
    "@remix-run/dev": "^2.14.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0",
    "vitest": "^2.1.0",
    "eslint": "^9.0.0",
    "eslint-plugin-react": "^7.0.0",
    "prettier": "^3.4.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("remix.config.ts"),
            r#"import { defineConfig } from '@remix-run/dev'

export default defineConfig({
  appDirectory: 'app',
  browserBuildDirectory: 'public/build',
  serverBuildPath: 'build/server/index.js',
})"#)?;

        std::fs::write(project_dir.join("vite.config.ts"),
            r#"import { vitePlugin as remix } from '@remix-run/dev'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [remix()],
})"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "lib": ["DOM", "DOM.Iterable", "ES2022"],
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "allowJs": true,
    "strict": true,
    "noEmit": true,
    "isolatedModules": true,
    "jsx": "react-jsx",
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "baseUrl": ".",
    "paths": { "~/*": ["./app/*"] }
  },
  "include": ["remix.env.d.ts", "**/*.ts", "**/*.tsx"]
}"#)?;

        std::fs::write(project_dir.join("app/root.tsx"),
            klyron_template::TemplateEngine::render_static(r#"import { Links, LiveReload, Meta, Outlet, Scripts, ScrollRestoration } from '@remix-run/react'
import './tailwind.css'

export default function App() {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <body>
        <Outlet />
        <ScrollRestoration />
        <Scripts />
        <LiveReload />
      </body>
    </html>
  )
}
"#, vars))?;

        std::fs::write(project_dir.join("app/routes/_index.tsx"),
            klyron_template::TemplateEngine::render_static(r#"export default function Index() {
  return (
    <main>
      <h1>Welcome to {{ name }}</h1>
    </main>
  )
}
"#, vars))?;

        std::fs::write(project_dir.join("app/routes/about.tsx"),
            r#"export default function About() {
  return <main><h1>About</h1></main>
}
"#)?;

        std::fs::write(project_dir.join("app/entry.client.tsx"),
            r#"import { RemixBrowser } from '@remix-run/react'
import { startTransition, StrictMode } from 'react'
import { hydrateRoot } from 'react-dom/client'

startTransition(() => {
  hydrateRoot(document, <StrictMode><RemixBrowser /></StrictMode>)
})"#)?;

        std::fs::write(project_dir.join("app/entry.server.tsx"),
            r#"import { RemixServer } from '@remix-run/react'
import { handleRequest } from '@remix-run/node'
import { renderToString } from 'react-dom/server'

export default function handleRequest(request, responseStatusCode, responseHeaders, remixContext) {
  const html = renderToString(<RemixServer context={remixContext} url={request.url} />)
  responseHeaders.set('Content-Type', 'text/html')
  return new Response(html, { status: responseStatusCode, headers: responseHeaders })
}"#)?;

        std::fs::write(project_dir.join("app/tailwind.css"),
            r#"@tailwind base;
@tailwind components;
@tailwind utilities;
"#)?;

        std::fs::write(project_dir.join("tailwind.config.ts"),
            r#"import type { Config } from 'tailwindcss'
export default { content: ['./app/**/*.{ts,tsx}'], theme: { extend: {} }, plugins: [] } satisfies Config"#)?;

        std::fs::write(project_dir.join("postcss.config.js"),
            r#"export default { plugins: { tailwindcss: {}, autoprefixer: {} } }"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\nbuild\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import react from 'eslint-plugin-react'

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  react.configs.flat.recommended,
  { ignores: ['build'] },
)"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Remix project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
