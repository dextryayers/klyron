use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct AnalogAdapter;

#[async_trait]
impl FrameworkAdapter for AnalogAdapter {
    fn name(&self) -> &'static str { "analog" }

    fn detect(&self, dir: &Path) -> bool {
        if !dir.join("package.json").exists() { return false; }
        std::fs::read_to_string(dir.join("package.json")).ok()
            .map(|c| c.contains("\"analog\"") || c.contains("\"@analogjs/"))
            .unwrap_or(false)
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["1.0"] }
    fn default_version(&self) -> &'static str { "1.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Fullstack }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["analog", "dev"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["analog", "build"]).current_dir(dir).status().await?;
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
        Some(("npx".into(), vec!["create-analog@latest".into(), name.into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src/app/pages"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "analog dev",
    "build": "analog build",
    "preview": "analog preview",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "@angular/core": "^19.0.0",
    "@angular/router": "^19.0.0",
    "@angular/platform-browser": "^19.0.0",
    "analog": "^1.0.0"
  },
  "devDependencies": {
    "vite": "^6.0.0",
    "typescript": "^5.6.0",
    "vitest": "^2.1.0",
    "eslint": "^9.0.0",
    "prettier": "^3.4.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("vite.config.ts"),
            r#"import { defineConfig } from 'vite'
import analog from 'analog'

export default defineConfig({
  plugins: [analog()],
  server: { port: 5173, host: true },
})"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "noEmit": true,
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}"#)?;

        std::fs::write(project_dir.join("tsconfig.app.json"),
            r#"{
  "extends": "./tsconfig.json",
  "compilerOptions": { "outDir": "./out-tsc/app", "types": [] },
  "files": ["src/main.ts"],
  "include": ["src/**/*.ts"]
}"#)?;

        std::fs::write(project_dir.join("tsconfig.spec.json"),
            r#"{
  "extends": "./tsconfig.json",
  "compilerOptions": { "outDir": "./out-tsc/spec", "types": ["vitest/globals"] },
  "include": ["src/**/*.spec.ts"]
}"#)?;

        std::fs::write(project_dir.join("angular.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "$schema": "./node_modules/@angular/cli/lib/config/schema.json",
  "version": 1,
  "newProjectRoot": "projects",
  "projects": {
    "{{ name }}": {
      "projectType": "application",
      "root": "",
      "sourceRoot": "src",
      "architect": {
        "build": { "builder": "analog:build" },
        "serve": { "builder": "analog:dev-server" }
      }
    }
  }
}"#, vars))?;

        std::fs::write(project_dir.join("src/index.html"),
            klyron_template::TemplateEngine::render_static(r#"<!doctype html>
<html lang="en">
  <head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>{{ name }}</title></head>
  <body><app-root></app-root><script type="module" src="/src/main.ts"></script></body>
</html>"#, vars))?;

        std::fs::write(project_dir.join("src/main.ts"),
            r#"import 'zone.js'
import { bootstrapApplication } from '@angular/platform-browser'
import { AppComponent } from './app/app.component'
import { appConfig } from './app/app.config'

bootstrapApplication(AppComponent, appConfig).catch(console.error)"#)?;

        std::fs::write(project_dir.join("src/app/app.component.ts"),
            klyron_template::TemplateEngine::render_static(r#"import { Component } from '@angular/core'
import { RouterOutlet } from '@angular/router'

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet],
  template: '<router-outlet></router-outlet>',
})
export class AppComponent {}"#, vars))?;

        std::fs::write(project_dir.join("src/app/app.config.ts"),
            r#"import { ApplicationConfig } from '@angular/core'
import { provideRouter } from '@angular/router'
import { routes } from './routes'

export const appConfig: ApplicationConfig = {
  providers: [provideRouter(routes)],
}"#)?;

        std::fs::write(project_dir.join("src/app/routes.ts"),
            r#"import { Route } from '@angular/router'

export const routes: Route[] = [
  { path: '', loadComponent: () => import('./pages/home.component').then(m => m.HomeComponent) },
]"#)?;

        std::fs::write(project_dir.join("src/app/pages/home.component.ts"),
            klyron_template::TemplateEngine::render_static(r#"import { Component } from '@angular/core'

@Component({
  selector: 'app-home',
  standalone: true,
  template: '<h1>Welcome to {{ name }}</h1>',
})
export class HomeComponent {}"#, vars))?;

        std::fs::write(project_dir.join("src/styles.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; min-height: 100vh; }
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n.angular\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Analog (Angular + Vite) project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
