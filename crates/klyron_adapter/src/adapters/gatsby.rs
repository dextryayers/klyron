use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;
use crate::{FrameworkAdapter, BuildOptions, ScaffoldOptions, FrameworkKind};

pub struct GatsbyAdapter;

#[async_trait]
impl FrameworkAdapter for GatsbyAdapter {
    fn name(&self) -> &'static str { "gatsby" }

    fn detect(&self, dir: &Path) -> bool {
        dir.join("gatsby-config.ts").exists() || dir.join("gatsby-config.js").exists()
    }

    fn supported_versions(&self) -> Vec<&'static str> { vec!["5.13"] }
    fn default_version(&self) -> &'static str { "5.13" }
    fn kind(&self) -> FrameworkKind { FrameworkKind::Frontend }

    async fn dev(&self, dir: &Path, port: Option<u16>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("npx");
        cmd.args(["gatsby", "develop"]).current_dir(dir);
        if let Some(p) = port { cmd.env("PORT", p.to_string()); }
        cmd.status().await?; Ok(())
    }

    async fn build(&self, dir: &Path, _opts: BuildOptions) -> Result<()> {
        tokio::process::Command::new("npx").args(["gatsby", "build"]).current_dir(dir).status().await?;
        Ok(())
    }

    async fn test(&self, dir: &Path, _filter: Option<&str>) -> Result<()> {
        tokio::process::Command::new("npx").args(["jest"]).current_dir(dir).status().await?;
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
        Some(("npx".into(), vec!["create-gatsby".into(), name.into()]))
    }

    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> Result<()> {
        let project_dir = options.dir.join(name);
        std::fs::create_dir_all(&project_dir)?;
        std::fs::create_dir_all(project_dir.join("src/pages"))?;
        std::fs::create_dir_all(project_dir.join("src/components"))?;
        std::fs::create_dir_all(project_dir.join("src/images"))?;

        let vars = &options.template_vars;

        std::fs::write(project_dir.join("package.json"),
            klyron_template::TemplateEngine::render_static(r#"{
  "name": "{{ name }}",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "dev": "gatsby develop",
    "build": "gatsby build",
    "serve": "gatsby serve",
    "test": "jest",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "gatsby": "^5.13.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "gatsby-plugin-image": "^3.13.0",
    "gatsby-plugin-sharp": "^5.13.0",
    "gatsby-source-filesystem": "^5.13.0",
    "gatsby-transformer-sharp": "^5.13.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "gatsby-plugin-typegen": "^5.0.0",
    "eslint": "^9.0.0",
    "eslint-plugin-react": "^7.0.0",
    "prettier": "^3.4.0",
    "jest": "^29.7.0"
  }
}"#, vars))?;

        std::fs::write(project_dir.join("gatsby-config.ts"),
            r#"import type { GatsbyConfig } from 'gatsby'

const config: GatsbyConfig = {
  siteMetadata: { siteUrl: 'https://example.com' },
  plugins: ['gatsby-plugin-image', 'gatsby-plugin-sharp', 'gatsby-transformer-sharp'],
}

export default config
"#)?;

        std::fs::write(project_dir.join("gatsby-node.ts"),
            r#"import type { GatsbyNode } from 'gatsby'

export const onCreatePage: GatsbyNode['onCreatePage'] = async ({ page, actions }) => {
  const { createPage } = actions
  if (page.path === '/') {
    createPage({ ...page })
  }
}
"#)?;

        std::fs::write(project_dir.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "jsx": "react-jsx",
    "noEmit": true
  },
  "include": ["./src/**/*", "./gatsby-node.ts", "./gatsby-config.ts"]
}"#)?;

        std::fs::write(project_dir.join("src/pages/index.tsx"),
            klyron_template::TemplateEngine::render_static(r#"import * as React from 'react'
import Layout from '../components/layout'
import Seo from '../components/seo'

const IndexPage = () => (
  <Layout>
    <h1>Welcome to {{ name }}</h1>
  </Layout>
)

export const Head = () => <Seo title="{{ name }}" />

export default IndexPage
"#, vars))?;

        std::fs::write(project_dir.join("src/pages/about.tsx"),
            r#"import * as React from 'react'
import Layout from '../components/layout'
import Seo from '../components/seo'

const AboutPage = () => (
  <Layout>
    <h1>About</h1>
  </Layout>
)

export const Head = () => <Seo title="About" />

export default AboutPage
"#)?;

        std::fs::write(project_dir.join("src/components/layout.tsx"),
            r#"import * as React from 'react'

const Layout = ({ children }: { children: React.ReactNode }) => (
  <main>{children}</main>
)

export default Layout
"#)?;

        std::fs::write(project_dir.join("src/components/seo.tsx"),
            r#"import * as React from 'react'

const Seo = ({ title }: { title: string }) => (
  <title>{title}</title>
)

export default Seo
"#)?;

        std::fs::write(project_dir.join("src/pages/404.tsx"),
            r#"import * as React from 'react'
import Layout from '../components/layout'

const NotFoundPage = () => (
  <Layout>
    <h1>404: Not Found</h1>
  </Layout>
)

export default NotFoundPage
"#)?;

        std::fs::write(project_dir.join(".gitignore"), "node_modules\n.cache\npublic\n.DS_Store\n")?;
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
  { ignores: ['.cache', 'public'] },
)"#)?;
        std::fs::write(project_dir.join("README.md"),
            klyron_template::TemplateEngine::render_static(r#"# {{ name }}

Gatsby project

## Getting Started

npm run dev
"#, vars))?;

        Ok(())
    }
}
