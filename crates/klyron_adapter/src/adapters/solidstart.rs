use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct SolidStartAdapter;

#[async_trait]
impl FrameworkAdapter for SolidStartAdapter {
    fn name(&self) -> &'static str { "solidstart" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"solid-start\"") || c.contains("\"@solidjs/start\""))
            .unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["1.0"] }
    fn default_version(&self) -> &'static str { "1.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Fullstack }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["solid", "start"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["solid", "build"]).current_dir(dir).status().await?;
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
        Some(("npm".into(), vec!["create".into(), "solid@latest".into(), name.into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src/routes"))?;
        std::fs::create_dir_all(project_dir.join("src/components"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "solid start",
    "build": "solid build",
    "preview": "solid preview",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "solid-js": "^1.9.0",
    "solid-start": "^1.0.0",
    "@solidjs/router": "^0.15.0"
  },
  "devDependencies": {
    "vite": "^6.0.0",
    "typescript": "^5.6.0",
    "vitest": "^2.1.0",
    "eslint": "^9.0.0",
    "prettier": "^3.4.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("app.config.ts"),
            r#"import { defineConfig } from 'solid-start'

export default defineConfig({})"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "jsxImportSource": "solid-js",
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}"#)?;

        std::fs::write(project_dir.join("vite.config.ts"),
            r#"import solid from 'solid-start/vite'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [solid()],
})"#)?;

        std::fs::write(project_dir.join("src/entry-server.tsx"),
            r#"import { createHandler, StartServer } from 'solid-start/entry-server'

export default createHandler(StartServer)"#)?;

        std::fs::write(project_dir.join("src/entry-client.tsx"),
            r#"import { mount, StartClient } from 'solid-start/entry-client'

mount(() => <StartClient />, document)"#)?;

        std::fs::write(project_dir.join("src/root.tsx"),
            klyron_template::TemplateEngine::render_static(r#"import { Routes, FileRoutes } from 'solid-start'
import { ErrorBoundary } from 'solid-start/error-boundary'
import { Suspense } from 'solid-js'
import './app.css'

export default function Root() {
  return (
    <html lang="en">
      <head><meta charset="utf-8" /><meta name="viewport" content="width=device-width, initial-scale=1" /><title>{{ name }}</title></head>
      <body>
        <ErrorBoundary>
          <Suspense fallback={<div>Loading...</div>}>
            <Routes><FileRoutes /></Routes>
          </Suspense>
        </ErrorBoundary>
      </body>
    </html>
  )
}
"#, vars))?;

        std::fs::write(project_dir.join("src/routes/index.tsx"),
            klyron_template::TemplateEngine::render_static(r#"export default function Home() {
  return <h1>Welcome to {{ name }}</h1>
}
"#, vars))?;

        std::fs::write(project_dir.join("src/routes/about.tsx"),
            r#"export default function About() {
  return <h1>About</h1>
}
"#)?;

        std::fs::write(project_dir.join("src/components/Counter.tsx"),
            r#"import { createSignal } from 'solid-js'

export default function Counter() {
  const [count, setCount] = createSignal(0)
  return <button onClick={() => setCount(c => c + 1)}>Count: {count()}</button>
}
"#)?;

        std::fs::write(project_dir.join("src/app.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; min-height: 100vh; }
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n.output\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

SolidStart project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
