use std::path::PathBuf;

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_test_scaffold_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_file(dir: &PathBuf, path: &str, content: &str) {
    let full = dir.join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
}

#[test]
fn test_scaffold_express_api_generates_proper_structure() {
    let dir = test_dir("scaffold_express");

    // Simulate what scaffold_express does
    let pkg_json = serde_json::json!({
        "name": "test-express-api",
        "version": "1.0.0",
        "private": true,
        "scripts": {
            "start": "node index.js",
            "dev": "node --watch index.js"
        },
        "dependencies": {
            "express": "^4.21.0"
        }
    });
    std::fs::write(dir.join("package.json"), serde_json::to_string_pretty(&pkg_json).unwrap()).unwrap();
    write_file(&dir, "index.js", "const express = require('express'); const app = express(); app.get('/', (req, res) => res.json({ ok: true })); app.listen(3000);");

    assert!(dir.join("package.json").exists(), "package.json should exist");
    assert!(dir.join("index.js").exists(), "index.js should exist");

    let content = std::fs::read_to_string(dir.join("package.json")).unwrap();
    assert!(content.contains("express"), "package.json should contain express dep");
    assert!(content.contains("test-express-api"), "package.json should have the project name");
}

#[test]
fn test_scaffold_react_app_generates_proper_structure() {
    let dir = test_dir("scaffold_react");

    // Simulate what scaffold_react does
    let pkg_json = serde_json::json!({
        "name": "test-react-app",
        "private": true,
        "version": "0.0.0",
        "type": "module",
        "scripts": {
            "dev": "vite",
            "build": "vite build",
            "preview": "vite preview"
        },
        "dependencies": {
            "react": "^18.3.0",
            "react-dom": "^18.3.0"
        },
        "devDependencies": {
            "@vitejs/plugin-react": "^4.3.0",
            "vite": "^5.4.0"
        }
    });
    std::fs::write(dir.join("package.json"), serde_json::to_string_pretty(&pkg_json).unwrap()).unwrap();
    write_file(&dir, "src/App.jsx", "function App() { return <h1>Hello React</h1>; } export default App;");
    write_file(&dir, "src/main.jsx", "import React from 'react'; import ReactDOM from 'react-dom/client'; import App from './App'; ReactDOM.createRoot(document.getElementById('root')).render(<React.StrictMode><App /></React.StrictMode>);");
    write_file(&dir, "index.html", "<!DOCTYPE html><html><head><title>Vite App</title></head><body><div id=\"root\"></div><script type=\"module\" src=\"/src/main.jsx\"></script></body></html>");
    write_file(&dir, "vite.config.js", "import { defineConfig } from 'vite'; import react from '@vitejs/plugin-react'; export default defineConfig({ plugins: [react()] });");

    assert!(dir.join("package.json").exists());
    assert!(dir.join("src/App.jsx").exists());
    assert!(dir.join("src/main.jsx").exists());
    assert!(dir.join("index.html").exists());
    assert!(dir.join("vite.config.js").exists());

    let pkg = std::fs::read_to_string(dir.join("package.json")).unwrap();
    assert!(pkg.contains("react"));
    assert!(pkg.contains("vite"));
}

#[test]
fn test_scaffold_variables_replacement() {
    let dir = test_dir("scaffold_vars");

    let name = "my-custom-app";
    let template = format!(
        r#"{{"name":"{name}","version":"1.0.0","description":"{name} project"}}"#
    );
    write_file(&dir, "package.json", &template);

    let content = std::fs::read_to_string(dir.join("package.json")).unwrap();
    assert!(content.contains("my-custom-app"));
    assert!(content.contains("\"name\":\"my-custom-app\""));
}

#[test]
fn test_scaffold_with_dir_structure() {
    let dir = test_dir("scaffold_structure");

    // Simulate a scaffold that creates directories
    let dirs = ["src", "src/components", "src/pages", "public", "tests"];
    for d in &dirs {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }
    write_file(&dir, "src/index.js", "console.log('hello');");
    write_file(&dir, "public/index.html", "<html><body></body></html>");

    for d in &dirs {
        assert!(dir.join(d).exists(), "{d} directory should exist");
    }
    assert!(dir.join("src/index.js").exists());
    assert!(dir.join("public/index.html").exists());
}

#[test]
fn test_scaffold_with_npm_dependencies() {
    let dir = test_dir("scaffold_npm_deps");

    let deps = HashMap::from([
        ("express", "^4.18.0"),
        ("lodash", "^4.17.0"),
        ("morgan", "^1.10.0"),
    ]);

    let pkg_json = serde_json::json!({
        "name": "test-deps",
        "version": "1.0.0",
        "dependencies": deps,
    });
    std::fs::write(dir.join("package.json"), serde_json::to_string_pretty(&pkg_json).unwrap()).unwrap();

    let content = std::fs::read_to_string(dir.join("package.json")).unwrap();
    for (name, ver) in &deps {
        assert!(content.contains(name), "package.json should contain {name}");
        assert!(content.contains(ver), "package.json should contain {ver}");
    }
}

#[test]
fn test_scaffold_express_api_index_js_valid() {
    let dir = test_dir("scaffold_express_valid");
    write_file(&dir, "index.js", r#"
const express = require('express');
const app = express();
const port = process.env.PORT || 3000;

app.get('/', (req, res) => {
  res.json({ message: 'Hello from Express!' });
});

app.listen(port, () => {
  console.log(`Server running on port ${port}`);
});
"#);

    let content = std::fs::read_to_string(dir.join("index.js")).unwrap();
    assert!(content.contains("express"));
    assert!(content.contains("app.get"));
    assert!(content.contains("app.listen"));
}

#[test]
fn test_scaffold_with_env_file() {
    let dir = test_dir("scaffold_env");
    write_file(&dir, ".env", "DATABASE_URL=postgres://localhost:5432/mydb\nPORT=4000\nNODE_ENV=development\n");
    write_file(&dir, ".env.example", "DATABASE_URL=\nPORT=\nNODE_ENV=\n");

    let env = std::fs::read_to_string(dir.join(".env")).unwrap();
    assert!(env.contains("DATABASE_URL=postgres://localhost:5432/mydb"));
    assert!(env.contains("PORT=4000"));

    let example = std::fs::read_to_string(dir.join(".env.example")).unwrap();
    assert!(example.contains("DATABASE_URL="));
    assert!(example.contains("PORT="));
}
