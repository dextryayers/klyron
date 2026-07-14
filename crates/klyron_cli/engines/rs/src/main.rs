use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;
use std::fs;

const MAX_OUTPUT: usize = 1_048_576;

#[derive(serde::Deserialize)]
struct Input {
    action: String,
    code: Option<String>,
    args: Option<String>,
    filename: Option<String>,
    files: Option<Vec<FileSpec>>,
    project: Option<String>,
}

#[derive(serde::Deserialize)]
struct FileSpec {
    name: String,
    content: String,
}

#[derive(serde::Serialize)]
struct Output {
    stdout: String,
    stderr: String,
    exit_code: i32,
    result: String,
}

fn write_output(o: Output) {
    if let Ok(json) = serde_json::to_string(&o) {
        println!("{}", json);
    }
    let _ = io::stdout().flush();
}

fn truncate(s: &str) -> String {
    if s.len() > MAX_OUTPUT {
        let mut r = s[..MAX_OUTPUT].to_string();
        r.push_str(&format!("\n... (truncated, {} bytes total)", s.len()));
        r
    } else {
        s.to_string()
    }
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("klyron-rs-{}", std::process::id()));
        let _ = fs::create_dir_all(&path);
        TempDir { path }
    }

    fn write(&self, rel: &str, content: &str) -> PathBuf {
        let p = self.path.join(rel);
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&p, content);
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn run(cmd: &mut Command, label: &str) -> Output {
    match cmd.output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            Output {
                stdout: truncate(&stdout),
                stderr: truncate(&stderr),
                exit_code: o.status.code().unwrap_or(-1),
                result: if o.status.success() {
                    "ok".into()
                } else {
                    format!("{} failed", label)
                },
            }
        }
        Err(e) => Output {
            stdout: String::new(),
            stderr: format!("Failed to run {}: {}", label, e),
            exit_code: -1,
            result: String::new(),
        },
    }
}

fn exec_code(
    code: &str,
    filename: Option<&str>,
    files: Option<&[FileSpec]>,
    args: Option<&str>,
) -> Output {
    let tmp = TempDir::new();
    let fname = filename.unwrap_or("main.rs");

    if let Some(files) = files {
        for f in files {
            tmp.write(&f.name, &f.content);
        }
    }

    tmp.write(fname, code);
    let src = tmp.path.join(fname);
    let mut bin = tmp.path.join("prog");
    if cfg!(target_os = "windows") {
        bin.set_extension("exe");
    }

    let mut cargs = vec![
        src.to_string_lossy().to_string(),
        "-o".into(),
        bin.to_string_lossy().to_string(),
    ];
    if args.map_or(false, |a| a.contains("release") || a.contains("optimize")) {
        cargs.push("-O".into());
    }

    let mut compiler = Command::new("rustc");
    compiler.args(&cargs);
    let comp = run(&mut compiler, "compilation");
    if comp.exit_code != 0 {
        return comp;
    }

    let mut runner = Command::new(&bin);
    run(&mut runner, "execution")
}

fn eval_code(code: &str) -> Output {
    let wrapped = format!(
        "fn main() {{\n    println!(\"{{:?}}\", {{ {} }});\n}}",
        code
    );
    exec_code(&wrapped, Some("main.rs"), None, None)
}

fn check_project(project: &str) -> Output {
    let mut cmd = Command::new("cargo");
    cmd.arg("check").current_dir(project);
    run(&mut cmd, "cargo check")
}

fn build_project(project: &str, args: Option<&str>) -> Output {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if args.map_or(false, |a| a.contains("release")) {
        cmd.arg("--release");
    }
    cmd.current_dir(project);
    run(&mut cmd, "cargo build")
}

fn test_project(project: &str, args: Option<&str>) -> Output {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    if let Some(a) = args {
        if !a.is_empty() {
            cmd.args(["--", a]);
        }
    }
    cmd.current_dir(project);
    run(&mut cmd, "cargo test")
}

fn fmt_code(code: &str) -> Output {
    let tmp = TempDir::new();
    let src = tmp.write("input.rs", code);

    let mut formatter = Command::new("rustfmt");
    formatter.arg(src.to_string_lossy().to_string());
    let out = run(&mut formatter, "rustfmt");
    if out.exit_code != 0 {
        return out;
    }

    match fs::read_to_string(&src) {
        Ok(formatted) => Output {
            stdout: formatted,
            stderr: String::new(),
            exit_code: 0,
            result: "ok".into(),
        },
        Err(e) => Output {
            stdout: String::new(),
            stderr: format!("Failed to read formatted file: {}", e),
            exit_code: 1,
            result: String::new(),
        },
    }
}

fn clippy_code(code: &str) -> Output {
    let tmp = TempDir::new();
    tmp.write(
        "Cargo.toml",
        r#"[package]
name = "temp_check"
version = "0.1.0"
edition = "2021"
"#,
    );
    tmp.write("src/main.rs", code);

    let mut clippy = Command::new("cargo");
    clippy.args(["clippy", "--", "-D", "warnings"]);
    clippy.current_dir(&tmp.path);
    run(&mut clippy, "clippy")
}

fn scaffold(kind: &str) -> Output {
    let (files, label): (&[(&str, &str)], &str) = match kind {
        "actix-web" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "actix-app"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
"#,
                ),
                (
                    "src/main.rs",
                    r#"use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
"#,
                ),
            ],
            "actix-web",
        ),
        "axum" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "axum-app"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
"#,
                ),
                (
                    "src/main.rs",
                    r#"use axum::{Router, routing::get, response::Html};

async fn hello() -> Html<&'static str> {
    Html("<h1>Hello, world!</h1>")
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
"#,
                ),
            ],
            "axum",
        ),
        "rocket" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "rocket-app"
version = "0.1.0"
edition = "2021"

[dependencies]
rocket = "0.5"
"#,
                ),
                (
                    "src/main.rs",
                    r#"#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
"#,
                ),
            ],
            "rocket",
        ),
        "cli" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "cli-app"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
"#,
                ),
                (
                    "src/main.rs",
                    r#"use clap::Parser;

#[derive(Parser)]
#[command(name = "cli-app")]
#[command(about = "A CLI application")]
struct Cli {
    #[arg(short, long)]
    name: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    let name = cli.name.unwrap_or_else(|| "world".to_string());
    println!("Hello, {name}!");
}
"#,
                ),
            ],
            "cli",
        ),
        "lib" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "my-lib"
version = "0.1.0"
edition = "2021"

[lib]
name = "my_lib"
path = "src/lib.rs"
"#,
                ),
                (
                    "src/lib.rs",
                    r#"pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("world"), "Hello, world!");
    }
}
"#,
                ),
            ],
            "lib",
        ),
        "lambda" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "lambda-app"
version = "0.1.0"
edition = "2021"

[dependencies]
lambda_runtime = "0.13"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
"#,
                ),
                (
                    "src/main.rs",
                    r#"use lambda_runtime::{handler_fn, Context, Error};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    body: String,
}

#[derive(Serialize)]
struct Response {
    status_code: u16,
    body: String,
}

async fn handler(event: Request, _: Context) -> Result<Response, Error> {
    Ok(Response {
        status_code: 200,
        body: format!("Hello from Lambda! Received: {}", event.body),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}
"#,
                ),
            ],
            "lambda",
        ),
        "tauri" => (
            &[
                (
                    "src-tauri/Cargo.toml",
                    r#"[package]
name = "tauri-app"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
"#,
                ),
                (
                    "src-tauri/src/main.rs",
                    r#"#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Tauri.", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
"#,
                ),
                (
                    "src-tauri/tauri.conf.json",
                    r#"{
  "$schema": "https://raw.githubusercontent.com/nicedoc/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "tauri-app",
  "version": "0.1.0",
  "identifier": "com.tauri-app",
  "build": {
    "frontendDist": "../src",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "tauri-app",
        "width": 800,
        "height": 600
      }
    ],
    "security": {
      "csp": null
    }
  }
}
"#,
                ),
                (
                    "src-tauri/icons/README.txt",
                    r#"Place your app icons in this directory.
Tauri requires icons in various sizes. Generate them using:
  cargo tauri icon path/to/icon.png
"#,
                ),
            ],
            "tauri",
        ),
        "leptos" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "leptos-app"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.7", features = ["csr"] }
leptos_meta = { version = "0.7" }
leptos_router = { version = "0.7" }
tokio = { version = "1", features = ["full"] }
"#,
                ),
                (
                    "src/main.rs",
                    r#"use leptos::*;
use leptos_meta::*;
use leptos_router::*;

fn main() {
    mount_to_body(|| view! { <App/> })
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Html attr:lang="en" attr:dir="ltr"/>
        <Title text="Leptos App"/>
        <Router>
            <Routes>
                <Route path="/" view=Home/>
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    view! {
        <h1>"Hello, Leptos!"</h1>
        <button on:click=move |_| set_count.update(|n| *n += 1)> 
            "Click me: " {count}
        </button>
    }
}
"#,
                ),
            ],
            "leptos",
        ),
        "yew" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "yew-app"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = { version = "0.21", features = ["csr"] }
wasm-bindgen = "0.2"
"#,
                ),
                (
                    "src/main.rs",
                    r#"use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        Callback::from(move |_| counter.set(*counter + 1))
    };

    html! {
        <div>
            <h1>{ "Hello, Yew!" }</h1>
            <p>{ "Counter: " } { *counter }</p>
            <button {onclick}>{ "Click me" }</button>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
"#,
                ),
                (
                    "index.html",
                    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <title>Yew App</title>
    <script type="module">
        import init from "./pkg/yew_app.js";
        init();
    </script>
</head>
<body>
    <div id="app"></div>
</body>
</html>
"#,
                ),
            ],
            "yew",
        ),
        "dioxus" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "dioxus-app"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { version = "0.6", features = ["web", "desktop"] }
"#,
                ),
                (
                    "src/main.rs",
                    r#"use dioxus::prelude::*;

fn app() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div {
            h1 { "Hello, Dioxus!" }
            p { "Counter: {count}" }
            button { onclick: move |_| count += 1, "Click me" }
        }
    }
}

fn main() {
    dioxus::launch(app);
}
"#,
                ),
            ],
            "dioxus",
        ),
        "warp" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "warp-app"
version = "0.1.0"
edition = "2021"

[dependencies]
warp = "0.3"
tokio = { version = "1", features = ["full"] }
"#,
                ),
                (
                    "src/main.rs",
                    r#"use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::path::end()
        .map(|| "Hello, world!");

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
"#,
                ),
            ],
            "warp",
        ),
        "tide" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "tide-app"
version = "0.1.0"
edition = "2021"

[dependencies]
tide = "0.16"
serde = { version = "1", features = ["derive"] }
"#,
                ),
                (
                    "src/main.rs",
                    r#"#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/").get(|_| async {
        Ok(tide::Response::builder(200)
            .body("Hello, world!")
            .build())
    });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
"#,
                ),
            ],
            "tide",
        ),
        "poem" => (
            &[
                (
                    "Cargo.toml",
                    r#"[package]
name = "poem-app"
version = "0.1.0"
edition = "2021"

[dependencies]
poem = "3"
tokio = { version = "1", features = ["full"] }
"#,
                ),
                (
                    "src/main.rs",
                    r#"use poem::{get, handler, listener::TcpListener, Route, Server};

#[handler]
fn hello() -> String {
    "Hello, world!".to_string()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().at("/", get(hello));
    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}
"#,
                ),
            ],
            "poem",
        ),
        _ => {
            return Output {
                stdout: String::new(),
                stderr: format!(
                    "Unknown scaffold type: {kind}. Supported: actix-web, axum, rocket, cli, lib, lambda, tauri, leptos, yew, dioxus, warp, tide, poem"
                ),
                exit_code: 1,
                result: String::new(),
            }
        }
    };

    let tmp = TempDir::new();
    let mut entries = Vec::new();
    for (name, content) in files {
        tmp.write(name, content);
        entries.push(serde_json::json!({
            "name": name,
            "content": content,
        }));
    }

    Output {
        stdout: String::new(),
        stderr: String::new(),
        exit_code: 0,
        result: serde_json::to_string(&serde_json::json!({
            "type": label,
            "files": entries,
        }))
        .unwrap_or_default(),
    }
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l.trim().to_string(),
            Err(_) => break,
        };
        if line.is_empty() {
            continue;
        }

        let input: Input = match serde_json::from_str(&line) {
            Ok(i) => i,
            Err(e) => {
                write_output(Output {
                    stdout: String::new(),
                    stderr: format!("Invalid JSON: {e}"),
                    exit_code: 1,
                    result: String::new(),
                });
                continue;
            }
        };

        let code = input.code.as_deref();
        let filename = input.filename.as_deref();
        let args = input.args.as_deref();
        let files = input.files.as_ref();
        let project = input.project.as_deref();

        let output = match input.action.as_str() {
            "exec" | "run" => exec_code(
                code.unwrap_or_default(),
                filename,
                files.map(|v| v.as_slice()),
                args,
            ),
            "eval" => eval_code(code.unwrap_or_default()),
            "check" => check_project(project.unwrap_or_default()),
            "build" => build_project(project.unwrap_or_default(), args),
            "test" => test_project(project.unwrap_or_default(), args),
            "fmt" => fmt_code(code.unwrap_or_default()),
            "clippy" => clippy_code(code.unwrap_or_default()),
            "scaffold" => scaffold(args.unwrap_or_default()),
            "ping" => Output {
                stdout: "pong".into(),
                stderr: String::new(),
                exit_code: 0,
                result: "ok".into(),
            },
            _ => Output {
                stdout: String::new(),
                stderr: format!("Unknown action: {}", input.action),
                exit_code: 1,
                result: String::new(),
            },
        };

        write_output(output);
    }
}
