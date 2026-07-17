use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct VueAdapter;

#[async_trait]
impl FrameworkAdapter for VueAdapter {
    fn name(&self) -> &'static str { "vue" }

    fn detect(&self, dir: &Path) -> bool {
        let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
        let has_vue = dir.join("package.json").exists().then(|| {
            std::fs::read_to_string(dir.join("package.json")).ok()
                .map(|c| c.contains("\"vue\"") || c.contains("\"vue-router\"") || c.contains("\"pinia\""))
                .unwrap_or(false)
        }).unwrap_or(false);
        has_vite && has_vue
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["3.4", "3.5", "3.6"] }
    fn default_version(&self) -> &'static str { "3.6" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Frontend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.arg("vite").current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
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
        Some(("npx".into(), vec!["create-vue@latest".into(), name.into()]))
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
        std::fs::create_dir_all(project_dir.join("src/stores"))?;
        std::fs::create_dir_all(project_dir.join("src/router"))?;
        std::fs::create_dir_all(project_dir.join("src/assets"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}", "private": true, "version": "1.0.0", "type": "module",
  "scripts": { "dev": "vite", "build": "vite build", "preview": "vite preview", "test": "vitest run", "lint": "eslint .", "format": "prettier --write ." },
  "dependencies": { "vue": "^3.5.0", "vue-router": "^4.0.0", "pinia": "^4.0.0" },
  "devDependencies": { "@vitejs/plugin-vue": "^5.2.0", "vite": "^6.1.0", "typescript": "^5.7.0", "vitest": "^3.0.0", "@vue/test-utils": "^2.4.6", "jsdom": "^26.0.0", "eslint": "^9.20.0", "@eslint/js": "^9.20.0", "typescript-eslint": "^8.24.0", "eslint-plugin-vue": "^9.32.0", "prettier": "^3.5.0" }
}"#, vars))?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{"compilerOptions":{"target":"ES2022","useDefineForClassFields":true,"lib":["ES2023","DOM","DOM.Iterable"],"module":"ESNext","skipLibCheck":true,"moduleResolution":"bundler","allowImportingTsExtensions":true,"isolatedModules":true,"moduleDetection":"force","noEmit":true,"jsx":"preserve","strict":true,"noUnusedLocals":true,"noUnusedParameters":true,"noFallthroughCasesInSwitch":true},"include":["src/**/*.ts","src/**/*.tsx","src/**/*.vue"]}"#)?;

        std::fs::write(project_dir.join("vite.config.ts"),
            r#"import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
export default defineConfig({ plugins: [vue()], server: { port: 5173, host: true } })"#)?;

        std::fs::write(project_dir.join("vitest.config.ts"),
            r#"import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
export default defineConfig({ plugins: [vue()], test: { globals: true, environment: 'jsdom' } })"#)?;

        std::fs::write(project_dir.join("index.html"),
            klyron_template::TemplateEngine::render_static(r#"<!doctype html><html lang="en"><head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>{{ name }}</title></head><body><div id="app"></div><script type="module" src="/src/main.ts"></script></body></html>"#, vars))?;

        std::fs::write(project_dir.join("src/main.ts"),
            r#"import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './index.css'
const app = createApp(App)
app.use(createPinia()); app.use(router); app.mount('#app')"#)?;

        std::fs::write(project_dir.join("src/App.vue"), r#"<template><router-view /></template>"#)?;

        std::fs::write(project_dir.join("src/pages/Home.vue"),
            klyron_template::TemplateEngine::render_static(r#"<template><div><h1>Welcome to {{ name }}</h1></div></template>"#, vars))?;

        std::fs::write(project_dir.join("src/router/index.ts"),
            r#"import { createRouter, createWebHistory } from 'vue-router'
import Home from '../pages/Home.vue'
const routes = [{ path: '/', name: 'Home', component: Home }]
const router = createRouter({ history: createWebHistory(), routes })
export default router"#)?;

        std::fs::write(project_dir.join("src/stores/counter.ts"),
            r#"import { defineStore } from 'pinia'
export const useCounterStore = defineStore('counter', { state: () => ({ count: 0 }), actions: { increment() { this.count++ } } })"#)?;

        std::fs::write(project_dir.join("src/index.css"), "* { margin: 0; padding: 0; box-sizing: border-box; }\n:root { font-family: Inter, sans-serif; }\nbody { min-height: 100vh; }\n#app { max-width: 1280px; margin: 0 auto; padding: 2rem; }\n")?;
        std::fs::write(project_dir.join("src/vite-env.d.ts"), r#"/// <reference types="vite/client" />
declare module '*.vue' { import type { DefineComponent } from 'vue'; const component: DefineComponent<{}, {}, any>; export default component }"#)?;
        std::fs::write(project_dir.join(".gitignore"), "node_modules\ndist\n.DS_Store\n*.local\n")?;
        std::fs::write(project_dir.join(".prettierrc"), r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import pluginVue from 'eslint-plugin-vue'
export default tseslint.config({ ignores: ['dist'] }, js.configs.recommended, ...tseslint.configs.recommended, ...pluginVue.configs['flat/essential'])"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}\nVue 3 + Pinia + Router\nnpm run dev"#, vars))?;

        Ok(())
    }
}
