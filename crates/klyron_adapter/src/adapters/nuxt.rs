use std::collections::HashMap;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct NuxtAdapter;

#[async_trait]
impl FrameworkAdapter for NuxtAdapter {
    fn name(&self) -> &'static str { "nuxt" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("nuxt.config.ts").exists() || dir.join("nuxt.config.js").exists() || dir.join("nuxt.config.mjs").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["3.13"] }
    fn default_version(&self) -> &'static str { "3.13" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Fullstack }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["nuxt", "dev"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["nuxt", "build"]).current_dir(dir).status().await?;
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
        std::fs::create_dir_all(project_dir.join("pages"))?;
        std::fs::create_dir_all(project_dir.join("components"))?;
        std::fs::create_dir_all(project_dir.join("layouts"))?;
        std::fs::create_dir_all(project_dir.join("composables"))?;
        std::fs::create_dir_all(project_dir.join("server"))?;
        std::fs::create_dir_all(project_dir.join("public"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "nuxt dev",
    "build": "nuxt build",
    "preview": "nuxt preview",
    "test": "vitest run",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "nuxt": "^3.13.0",
    "vue": "^3.5.0",
    "vue-router": "^4.4.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "vitest": "^2.1.0",
    "eslint": "^9.0.0",
    "@nuxt/eslint": "^0.7.0",
    "prettier": "^3.4.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("nuxt.config.ts"),
            r#"export default defineNuxtConfig({
  devtools: { enabled: true },
  modules: [],
  css: ['~/assets/css/main.css'],
  postcss: {
    plugins: { tailwindcss: {}, autoprefixer: {} },
  },
})"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "extends": "./.nuxt/tsconfig.json",
  "compilerOptions": { "strict": true }
}"#)?;

        std::fs::write(project_dir.join("app.vue"),
            klyron_template::TemplateEngine::render(r#"<template>
  <div>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </div>
</template>
"#, vars))?;

        std::fs::write(project_dir.join("pages/index.vue"),
            klyron_template::TemplateEngine::render(r#"<template>
  <div>
    <h1>Welcome to {{ name }}</h1>
  </div>
</template>
"#, vars))?;

        std::fs::write(project_dir.join("pages/about.vue"),
            r#"<template>
  <div>
    <h1>About</h1>
  </div>
</template>
"#)?;

        std::fs::write(project_dir.join("layouts/default.vue"),
            r#"<template>
  <div>
    <header>
      <nav>
        <NuxtLink to="/">Home</NuxtLink>
        <NuxtLink to="/about">About</NuxtLink>
      </nav>
    </header>
    <main>
      <slot />
    </main>
  </div>
</template>
"#)?;

        std::fs::write(project_dir.join("assets/css/main.css"),
            r#"@tailwind base;
@tailwind components;
@tailwind utilities;
"#)?;

        std::fs::write(project_dir.join("tailwind.config.js"),
            r#"export default {
  content: ['./components/**/*.vue', './layouts/**/*.vue', './pages/**/*.vue', './app.vue'],
  theme: { extend: {} },
  plugins: [],
}"#)?;

        std::fs::write(project_dir.join("postcss.config.js"),
            r#"export default { plugins: { tailwindcss: {}, autoprefixer: {} } }"#)?;

        std::fs::write(project_dir.join("composables/useCounter.ts"),
            r#"export const useCounter = () => {
  const count = useState('counter', () => 0)
  const increment = () => count.value++
  return { count, increment }
}
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.nuxt\n.output\n.DS_Store\n")?;
        std::fs::write(project_dir.join(".prettierrc"),
            r#"{"semi": false, "singleQuote": true, "tabWidth": 2, "trailingComma": "es5", "printWidth": 100}"#)?;
        std::fs::write(project_dir.join("eslint.config.js"),
            r#"import withNuxt from './.nuxt/eslint.config.mjs'
export default withNuxt()
"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render(r#"# {{ name }}

Nuxt 3 project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
