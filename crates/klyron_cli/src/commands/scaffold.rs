use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use clap::Args;
use klyron_adapter::{AdapterRegistry, ScaffoldOptions};
use klyron_adapter::adapters::register_all;

#[derive(Args)]
pub struct ScaffoldArgs {
    pub name: String,
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub external: bool,
    #[arg(long)]
    pub stack: Option<String>,
}

pub fn scaffold_via_external_cli(framework: &str, args: &ScaffoldArgs) -> anyhow::Result<()> {
    let project_dir = args.dir.join(&args.name);
    if project_dir.exists() {
        anyhow::bail!("Directory exists: {}", project_dir.display());
    }
    let name = &args.name;
    let dir = &args.dir;
    let _version_flag = args.version.as_deref().map(|v| format!("--version {v}")).unwrap_or_default();

    let (cmd, base_args): (&str, Vec<String>) = match framework {
        "react" => ("npx", vec!["create-vite@latest".into(), name.into(), "--template".into(), "react-ts".into()]),
        "vue" => ("npm", vec!["create".into(), "vue@latest".into(), name.into()]),
        "next" => {
            let mut a = vec!["create-next-app@latest".into(), name.into()];
            if let Some(v) = &args.version { a.push("--version".into()); a.push(v.into()); }
            ("npx", a)
        }
        "astro" => ("npm", vec!["create".into(), "astro@latest".into(), name.into()]),
        "nuxt" => ("npx", vec!["nuxi@latest".into(), "init".into(), name.into()]),
        "sveltekit" => ("npm", vec!["create".into(), "svelte@latest".into(), name.into()]),
        "remix" => ("npx", vec!["create-remix@latest".into(), name.into()]),
        "angular" => ("ng", vec!["new".into(), name.into()]),
        "express" => {
            std::fs::create_dir_all(&project_dir)?;
            std::fs::write(project_dir.join("package.json"), format!(r#"{{"name":"{name}","private":true,"scripts":{{"start":"node index.js"}},"dependencies":{{"express":"^4.21"}}}}"#))?;
            std::fs::write(project_dir.join("index.js"), r#"const express = require('express'); const app = express(); app.get('/', (req, res) => res.send('Hello!')); app.listen(3000);"#)?;
            println!("Express app created: {}", project_dir.display());
            return Ok(());
        }
        "fastify" => ("npm", vec!["create".into(), "fastify@latest".into(), name.into()]),
        "nest" => ("nest", vec!["new".into(), name.into()]),
        "hono" => ("npm", vec!["create".into(), "hono@latest".into(), name.into()]),
        "solid" => ("npm", vec!["create".into(), "solid@latest".into(), name.into()]),
        "qwik" => ("npm", vec!["create".into(), "qwik@latest".into(), name.into()]),
        "preact" => ("npx", vec!["create-vite@latest".into(), name.into(), "--template".into(), "preact-ts".into()]),
        "svelte" => ("npx", vec!["create-vite@latest".into(), name.into(), "--template".into(), "svelte-ts".into()]),
        "lit" => ("npx", vec!["create-vite@latest".into(), name.into(), "--template".into(), "lit-ts".into()]),
        "laravel" => ("composer", vec!["create-project".into(), "laravel/laravel".into(), name.into()]),
        "django" => ("django-admin", vec!["startproject".into(), name.into()]),
        "rails" => ("rails", vec!["new".into(), name.into()]),
        "symfony" => ("composer", vec!["create-project".into(), "symfony/skeleton".into(), name.into()]),
        "codeigniter" => ("composer", vec!["create-project".into(), "codeigniter4/appstarter".into(), name.into()]),
        "wordpress" => {
            println!("WordPress external scaffold: download WordPress from wordpress.org");
            println!("  wp core download --path={}", project_dir.display());
            println!("Or use `klyron create wordpress {}` (without --external)", name);
            return Ok(());
        }
        _ => anyhow::bail!("Unknown framework for external CLI: {framework}"),
    };

    let status = Command::new(cmd)
        .args(&base_args)
        .current_dir(dir)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run {cmd}: {e}"))?;

    if !status.success() {
        anyhow::bail!("{cmd} exited with code {}", status);
    }

    println!("{} app created via external CLI: {}", framework, project_dir.display());
    Ok(())
}

pub fn scaffold_via_adapter(args: &ScaffoldArgs, framework: &str) -> anyhow::Result<()> {
    if args.external {
        return scaffold_via_external_cli(framework, args);
    }

    let mut registry = AdapterRegistry::new();
    register_all(&mut registry);

    let adapter = registry
        .get(framework)
        .ok_or_else(|| anyhow::anyhow!("Unknown framework: {}", framework))?;

    let project_dir = args.dir.join(&args.name);
    if project_dir.exists() {
        anyhow::bail!("Directory exists: {}", project_dir.display());
    }

    let mut template_vars = HashMap::from([("name".to_string(), args.name.clone())]);
    if let Some(stack) = &args.stack {
        template_vars.insert("stack".to_string(), stack.clone());
    }

    let options = ScaffoldOptions {
        dir: args.dir.clone(),
        version: args.version.clone(),
        template_vars,
        external: args.external,
    };

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(adapter.scaffold(&args.name, options))?;

    println!("{} app created: {}", framework, project_dir.display());
    Ok(())
}

fn project_dir(args: &ScaffoldArgs) -> PathBuf {
    args.dir.join(&args.name)
}

pub fn scaffold_next(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "next")
}

pub fn scaffold_react(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "react")
}

pub fn scaffold_vue(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "vue")
}

pub fn scaffold_sveltekit(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "sveltekit")
}

pub fn scaffold_astro(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "astro")
}

pub fn scaffold_nuxt(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "nuxt")
}

pub fn scaffold_remix(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "remix")
}

pub fn scaffold_angular(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "angular")
}

pub fn scaffold_solid(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "solid")
}

pub fn scaffold_qwik(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "qwik")
}

pub fn scaffold_preact(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "preact")
}

pub fn scaffold_lit(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "lit")
}

pub fn scaffold_express(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "express")
}

pub fn scaffold_fastify(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "fastify")
}

pub fn scaffold_nest(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "nestjs")
}

pub fn scaffold_hono(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "hono")
}

pub fn scaffold_koa(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "koa")
}

pub fn scaffold_hapi(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "hapi")
}

pub fn scaffold_adonis(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "adonis")
}

pub fn scaffold_svelte(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "svelte")
}

pub fn scaffold_tauri(args: &ScaffoldArgs) -> anyhow::Result<()> {
    if args.external {
        return scaffold_via_external_cli("tauri", args);
    }
    let pd = project_dir(args);
    if pd.exists() {
        anyhow::bail!("Directory exists: {}", pd.display());
    }
    crate::mkdirs(
        &pd,
        &["src", "src-tauri", "src-tauri/src", "public"],
    )?;
    crate::write_files(
        &pd,
        vec![
            (
                "package.json",
                &r#"{
  "name": "NAME", "version": "0.1.0", "private": true, "type": "module",
  "scripts": { "dev": "vite", "build": "vite build && tauri build", "preview": "vite preview", "tauri": "tauri" },
  "dependencies": { "@tauri-apps/api": "^2.0.0" },
  "devDependencies": { "@tauri-apps/cli": "^2.0.0", "vite": "^6.0.0", "typescript": "^5.0.0" }
}"#
                .replace("NAME", &args.name),
            ),
            (
                "index.html",
                r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8" /><title>Tauri App</title></head><body><div id="app"></div><script type="module" src="/src/main.ts"></script></body></html>"#,
            ),
            (
                "src/main.ts",
                r##"import { invoke } from "@tauri-apps/api/core";
const app = document.querySelector<HTMLDivElement>("#app");
if (app) {
  app.innerHTML = `<h1>Tauri + Klyron</h1><button id="greet-btn">Greet</button>`;
  document.querySelector("#greet-btn")?.addEventListener("click", async () => {
    alert(await invoke("greet", { name: "Klyron" }));
  });
}"##,
            ),
            (
                "src/styles.css",
                r#":root { font-family: Inter, sans-serif; } body { margin: 0; padding: 2rem; }"#,
            ),
            (
                "src-tauri/Cargo.toml",
                r#"[package] name = "tauri-app" version = "0.1.0" edition = "2021"
[dependencies] tauri = { version = "2", features = [] } serde = { version = "1", features = ["derive"] } serde_json = "1"
[build-dependencies] tauri-build = { version = "2", features = [] }"#,
            ),
            (
                "src-tauri/src/lib.rs",
                r#"#[tauri::command] fn greet(name: &str) -> String { format!("Hello, {}!", name) }
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() { tauri::Builder::default().invoke_handler(tauri::generate_handler![greet]).run(tauri::generate_context!()).expect("error"); }"#,
            ),
            (
                "src-tauri/src/main.rs",
                r#"#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() { tauri_app::run(); }"#,
            ),
            (
                "src-tauri/tauri.conf.json",
                r#"{"productName":"tauri-app","version":"0.1.0","identifier":"com.tauri-app","build":{"beforeDevCommand":"npm run dev","devUrl":"http://localhost:5173","beforeBuildCommand":"npm run build","frontendDist":"../dist"},"app":{"windows":[{"title":"Tauri App","width":800,"height":600}]}}"#,
            ),
            (
                "src-tauri/build.rs",
                r#"fn main() { tauri_build::build(); }"#,
            ),
            (
                "vite.config.ts",
                r#"import { defineConfig } from 'vite'

export default defineConfig({
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
})
"#,
            ),
            (
                "vitest.config.ts",
                r#"import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    environment: 'jsdom',
    globals: true,
  },
})
"#,
            ),
        ],
    )?;
    println!("Tauri app created: {}", pd.display());
    Ok(())
}

pub fn scaffold_symfony(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "symfony")
}

pub fn scaffold_codeigniter(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "codeigniter")
}

pub fn scaffold_wordpress(args: &ScaffoldArgs) -> anyhow::Result<()> {
    scaffold_via_adapter(args, "wordpress")
}

pub fn scaffold_leptos(args: &ScaffoldArgs) -> anyhow::Result<()> {
    if args.external {
        return scaffold_via_external_cli("leptos", args);
    }
    let pd = project_dir(args);
    if pd.exists() {
        anyhow::bail!("Directory exists: {}", pd.display());
    }
    crate::mkdirs(&pd, &["src", "public"])?;
    crate::write_files(
        &pd,
        vec![
            (
                "Cargo.toml",
                r#"[package] name = "leptos-app" version = "0.1.0" edition = "2021"
[dependencies] leptos = { version = "0.7", features = ["csr"] } console_log = "1" wasm-bindgen = "0.2"
[profile.release] codegen-units = 1 lto = true"#,
            ),
            (
                "index.html",
                r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>Leptos App</title></head><body></body></html>"#,
            ),
            (
                "src/main.rs",
                r#"use leptos::*;
fn main() { mount_to_body(|| view! { <h1>"Hello, Leptos!"</h1> }) }"#,
            ),
        ],
    )?;
    println!("Leptos app created: {}", pd.display());
    Ok(())
}
