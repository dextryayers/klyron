use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct ReactAdapter;

#[async_trait]
impl FrameworkAdapter for ReactAdapter {
    fn name(&self) -> &'static str { "react" }

    fn detect(&self, dir: &Path) -> bool {
        let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists() || dir.join("vite.config.mjs").exists();
        let has_react = dir.join("package.json").exists().then(|| {
            std::fs::read_to_string(dir.join("package.json")).ok()
                .map(|c| c.contains("\"react\"") || c.contains("\"react-dom\"")).unwrap_or(false)
        }).unwrap_or(false);
        has_vite && has_react
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["18.0", "19.0", "19.1"] }
    fn default_version(&self) -> &'static str { "19.1" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Frontend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.arg("vite").current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?;
        Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["vite", "build"]).current_dir(dir).status().await?;
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
        Some(("npx".into(), vec!["create-vite@latest".into(), name.into(), "--template".into(), "react-ts".into()]))
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
        std::fs::create_dir_all(project_dir.join("src/pages"))?;
        std::fs::create_dir_all(project_dir.join("src/assets"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "private": true,
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "react": "^19.1.0",
    "react-dom": "^19.1.0",
    "react-router-dom": "^7.1.0"
  },
  "devDependencies": {
    "@vitejs/plugin-react": "^4.4.0",
    "vite": "^6.1.0",
    "typescript": "^5.7.0",
    "@types/react": "^19.1.0",
    "@types/react-dom": "^19.1.0",
    "vitest": "^3.0.0",
    "@testing-library/react": "^16.2.0",
    "jsdom": "^26.0.0",
    "eslint": "^9.20.0",
    "@eslint/js": "^9.20.0",
    "typescript-eslint": "^8.24.0",
    "eslint-plugin-react-hooks": "^5.2.0",
    "eslint-plugin-react-refresh": "^0.4.19",
    "prettier": "^3.5.0",
    "prettier-plugin-tailwindcss": "^0.6.11"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("tsconfig.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "lib": ["ES2023", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src"]
}"#, vars))?;

        std::fs::write(project_dir.join("vite.config.ts"),
            r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: { port: 5173, host: true },
})"#)?;

        std::fs::write(project_dir.join("vitest.config.ts"),
            r#"import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: { globals: true, environment: 'jsdom', setupFiles: './src/test/setup.ts', css: true },
})"#)?;

        std::fs::write(project_dir.join("index.html"),
            klyron_template::TemplateEngine::render_static(r#"<!doctype html>
<html lang="en">
  <head><meta charset="UTF-8" /><link rel="icon" type="image/svg+xml" href="/vite.svg" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>{{ name }}</title></head>
  <body><div id="root"></div><script type="module" src="/src/main.tsx"></script></body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("src/main.tsx"),
            r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserRouter><App /></BrowserRouter>
  </React.StrictMode>,
)"#)?;

        std::fs::write(project_dir.join("src/App.tsx"),
            r#"import { Routes, Route } from 'react-router-dom'
import Home from './pages/Home'

function App() {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
    </Routes>
  )
}
export default App"#)?;

        std::fs::write(project_dir.join("src/pages/Home.tsx"),
            klyron_template::TemplateEngine::render_static(r#"function Home() {
  return <div><h1>Welcome to {{ name }}</h1></div>
}
export default Home"#, vars))?;

        std::fs::write(project_dir.join("src/index.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
:root { font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif; line-height: 1.5; font-weight: 400; color: #213547; background-color: #fff; }
body { min-height: 100vh; }
a { font-weight: 500; color: #646cff; text-decoration: inherit; }
a:hover { color: #535bf2; }
h1 { font-size: 3.2em; line-height: 1.1; }
button { border-radius: 8px; border: 1px solid transparent; padding: 0.6em 1.2em; font-size: 1em; font-weight: 500; font-family: inherit; cursor: pointer; transition: border-color 0.25s; }
button:hover { border-color: #646cff; }
"#)?;

        std::fs::write(project_dir.join("src/App.css"), "#root { max-width: 1280px; margin: 0 auto; padding: 2rem; text-align: center; }\n")?;
        std::fs::write(project_dir.join("src/vite-env.d.ts"), r#"/// <reference types="vite/client" />"#)?;
        std::fs::write(project_dir.join("src/test/setup.ts"), r#"import '@testing-library/jest-dom'"#)?;
        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n*.local\n")?;
        std::fs::write(project_dir.join(".prettierrc"), r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;

        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'

export default tseslint.config(
  { ignores: ['dist'] },
  { extends: [js.configs.recommended, ...tseslint.configs.recommended], files: ['**/*.{ts,tsx}'], plugins: { 'react-hooks': reactHooks, 'react-refresh': reactRefresh }, rules: { ...reactHooks.configs.recommended.rules, 'react-refresh/only-export-components': ['warn', { allowConstantExport: true }] } },
)"#)?;

        std::fs::write(project_dir.join("public/vite.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><title>Vite</title><path d="M29.883 6.146L16.741 29.645a.423.423 0 01-.744 0L2.117 6.146a.423.423 0 01.508-.606L16 9.678l13.375-4.138a.423.423 0 01.508.606z" fill="#646CFF"/></svg>"##)?;

        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}
React + Vite + TypeScript
npm run dev
"#, vars))?;

        Ok(())
    }
}
