use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct AngularAdapter;

#[async_trait]
impl FrameworkAdapter for AngularAdapter {
    fn name(&self) -> &'static str { "angular" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("angular.json").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["17.0", "18.0"] }
    fn default_version(&self) -> &'static str { "18.0" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Frontend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["ng", "serve"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["ng", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("npx").args(["ng", "test"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn lint(&self, dir: &Path, _fix: bool) -> Result<()> {
        tokio::process::Command::new("npx").args(["ng", "lint"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn format(&self, dir: &Path, write: bool) -> Result<()> {
        tokio::process::Command::new("npx")
            .args(if write { ["prettier", "--write", "."] } else { ["prettier", "--check", "."] })
            .current_dir(dir).status().await?;
        Ok(())
    }

    fn external_scaffold_command(&self, name: &str, _version: Option<&str>) -> Option<(String, Vec<String>)> {
        Some(("npx".into(), vec!["@angular/cli@latest".into(), "new".into(), name.into()]))
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
        std::fs::create_dir_all(project_dir.join("src/app/pages/home"))?;
        std::fs::create_dir_all(project_dir.join("src/app/pages/about"))?;
        std::fs::create_dir_all(project_dir.join("src/environments"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("angular.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "$schema": "./node_modules/@angular/cli/lib/config/schema.json",
  "version": 1,
  "newProjectRoot": "projects",
  "projects": {
    "{{ name }}": {
      "projectType": "application",
      "schematics": {},
      "root": "",
      "sourceRoot": "src",
      "prefix": "app",
      "architect": {
        "build": {
          "builder": "@angular-devkit/build-angular:application",
          "options": {
            "outputPath": "dist",
            "index": "src/index.html",
            "browser": "src/main.ts",
            "polyfills": ["zone.js"],
            "tsConfig": "tsconfig.app.json",
            "assets": ["public"],
            "styles": ["src/styles.css"],
            "scripts": []
          },
          "configurations": {
            "production": { "budgets": [], "outputHashing": "all" },
            "development": { "optimization": false, "extractLicenses": false, "sourceMap": true }
          },
          "defaultConfiguration": "production"
        },
        "serve": {
          "builder": "@angular-devkit/build-angular:dev-server",
          "configurations": { "production": { "buildTarget": "{{ name }}:build:production" }, "development": { "buildTarget": "{{ name }}:build:development" } },
          "defaultConfiguration": "development"
        },
        "test": { "builder": "@angular-devkit/build-angular:karma", "options": { "polyfills": ["zone.js", "zone.js/testing"], "tsConfig": "tsconfig.spec.json", "assets": ["public"], "styles": ["src/styles.css"], "scripts": [] } },
        "lint": { "builder": "@angular-eslint/builder:lint", "options": { "lintFilePatterns": ["src/**/*.ts", "src/**/*.html"] } }
      }
    }
  },
  "cli": { "analytics": false }
}"#, vars))?;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "ng": "ng",
    "dev": "ng serve",
    "build": "ng build",
    "test": "ng test",
    "lint": "ng lint",
    "format": "prettier --write ."
  },
  "dependencies": {
    "@angular/animations": "^18.0.0",
    "@angular/common": "^18.0.0",
    "@angular/compiler": "^18.0.0",
    "@angular/core": "^18.0.0",
    "@angular/forms": "^18.0.0",
    "@angular/platform-browser": "^18.0.0",
    "@angular/platform-browser-dynamic": "^18.0.0",
    "@angular/router": "^18.0.0",
    "rxjs": "^7.8.0",
    "zone.js": "^0.14.0"
  },
  "devDependencies": {
    "@angular-devkit/build-angular": "^18.0.0",
    "@angular/cli": "^18.0.0",
    "@angular/compiler-cli": "^18.0.0",
    "typescript": "^5.5.0",
    "eslint": "^9.0.0",
    "@angular-eslint/eslint-plugin": "^18.0.0",
    "@angular-eslint/eslint-plugin-template": "^18.0.0",
    "@angular-eslint/template-parser": "^18.0.0",
    "prettier": "^3.4.0",
    "jasmine-core": "^5.0.0",
    "karma": "^6.4.0",
    "karma-chrome-launcher": "^3.2.0",
    "karma-jasmine": "^5.1.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compileOnSave": false,
  "compilerOptions": {
    "baseUrl": "./",
    "outDir": "./dist/out-tsc",
    "forceConsistentCasingInFileNames": true,
    "strict": true,
    "noImplicitOverride": true,
    "noPropertyAccessFromIndexSignature": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "sourceMap": true,
    "declaration": false,
    "downlevelIteration": true,
    "experimentalDecorators": true,
    "moduleResolution": "bundler",
    "importHelpers": true,
    "target": "ES2022",
    "module": "ES2022",
    "useDefineForClassFields": false,
    "lib": ["ES2022", "dom"]
  },
  "angularCompilerOptions": {
    "enableI18nLegacyMessageIdFormat": false,
    "strictInjectionParameters": true,
    "strictInputAccessModifiers": true,
    "strictTemplates": true
  }
}"#)?;

        std::fs::write(project_dir.join("tsconfig.app.json"),
            r#"{
  "extends": "./tsconfig.json",
  "compilerOptions": { "outDir": "./out-tsc/app", "types": [] },
  "files": ["src/main.ts"],
  "include": ["src/**/*.d.ts"]
}"#)?;

        std::fs::write(project_dir.join("tsconfig.spec.json"),
            r#"{
  "extends": "./tsconfig.json",
  "compilerOptions": { "outDir": "./out-tsc/spec", "types": ["jasmine"] },
  "include": ["src/**/*.spec.ts", "src/**/*.d.ts"]
}"#)?;

        std::fs::write(project_dir.join("src/index.html"),
            klyron_template::TemplateEngine::render_static(r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>{{ name }}</title>
  <base href="/">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="icon" type="image/x-icon" href="favicon.ico">
</head>
<body>
  <app-root></app-root>
</body>
</html>
"#, vars))?;

        std::fs::write(project_dir.join("src/main.ts"),
            r#"import { bootstrapApplication } from '@angular/platform-browser'
import { appConfig } from './app/app.config'
import { AppComponent } from './app/app.component'

bootstrapApplication(AppComponent, appConfig).catch((err) => console.error(err))
"#)?;

        std::fs::write(project_dir.join("src/styles.css"),
            r#"* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: system-ui, sans-serif; min-height: 100vh; }
"#)?;

        std::fs::write(project_dir.join("src/app/app.config.ts"),
            r#"import { ApplicationConfig, provideZoneChangeDetection } from '@angular/core'
import { provideRouter } from '@angular/router'
import { routes } from './app.routes'

export const appConfig: ApplicationConfig = {
  providers: [provideZoneChangeDetection({ eventCoalescing: true }), provideRouter(routes)],
}
"#)?;

        std::fs::write(project_dir.join("src/app/app.routes.ts"),
            r#"import { Routes } from '@angular/router'
import { HomeComponent } from './pages/home/home.component'

export const routes: Routes = [
  { path: '', component: HomeComponent },
  { path: 'about', loadComponent: () => import('./pages/about/about.component').then(m => m.AboutComponent) },
]
"#)?;

        std::fs::write(project_dir.join("src/app/app.component.ts"),
            r#"import { Component } from '@angular/core'
import { RouterOutlet } from '@angular/router'

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet],
  template: `<router-outlet />`,
})
export class AppComponent {}
"#)?;

        std::fs::write(project_dir.join("src/app/pages/home/home.component.ts"),
            klyron_template::TemplateEngine::render_static(r#"import { Component } from '@angular/core'

@Component({
  selector: 'app-home',
  standalone: true,
  template: `<h1>Welcome to {{ name }}</h1>`,
})
export class HomeComponent {}
"#, vars))?;

        std::fs::write(project_dir.join("src/app/pages/about/about.component.ts"),
            r#"import { Component } from '@angular/core'

@Component({
  selector: 'app-about',
  standalone: true,
  template: `<h1>About</h1>`,
})
export class AboutComponent {}
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n*.log\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": true, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import tseslint from 'typescript-eslint'
import angular from '@angular-eslint/eslint-plugin'
import angularTemplate from '@angular-eslint/eslint-plugin-template'

export default tseslint.config(
  { files: ['**/*.ts'], plugins: { '@angular-eslint': angular }, rules: {} },
  { files: ['**/*.html'], plugins: { '@angular-eslint/template': angularTemplate }, rules: {} },
  { ignores: ['dist'] },
)"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Angular project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
