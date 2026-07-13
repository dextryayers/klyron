use std::io::{self, BufRead, Write};
use std::process::Command;
use std::path::Path;
use std::fs;

#[derive(serde::Deserialize)]
struct Input {
    action: String,
    code: Option<String>,
    args: Option<String>,
    filename: Option<String>,
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

struct TempDir { path: String }
impl TempDir {
    fn new() -> Self {
        let path = format!("/tmp/klyron-rs-{}", std::process::id());
        let _ = fs::create_dir_all(&path);
        TempDir { path }
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn exec_code(code: &str) {
    let tmp = TempDir::new();
    let src_path = Path::new(&tmp.path).join("main.rs");
    let bin_path = Path::new(&tmp.path).join("prog");

    if fs::write(&src_path, code).is_err() {
        write_output(Output { stdout: "".into(), stderr: "Failed to write source".into(), exit_code: 1, result: "".into() });
        return;
    }

    let compile_output = match Command::new("rustc")
        .args([&src_path.to_string_lossy(), "-o", &bin_path.to_string_lossy(), "-O"])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            write_output(Output { stdout: "".into(), stderr: format!("Failed to run rustc: {e}"), exit_code: 1, result: "".into() });
            return;
        }
    };

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        write_output(Output {
            stdout: "".into(),
            stderr: stderr.to_string(),
            exit_code: 1,
            result: "Compilation failed".into(),
        });
        return;
    }

    let run_output = match Command::new(&bin_path).output() {
        Ok(o) => o,
        Err(e) => {
            write_output(Output { stdout: "".into(), stderr: format!("Failed to run: {e}"), exit_code: 1, result: "".into() });
            return;
        }
    };

    write_output(Output {
        stdout: String::from_utf8_lossy(&run_output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&run_output.stderr).to_string(),
        exit_code: run_output.status.code().unwrap_or(-1),
        result: String::from_utf8_lossy(&run_output.stdout).to_string(),
    });
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l.trim().to_string(),
            Err(_) => break,
        };
        if line.is_empty() { continue; }

        let input: Input = match serde_json::from_str(&line) {
            Ok(i) => i,
            Err(e) => {
                write_output(Output { stdout: "".into(), stderr: format!("Invalid JSON: {e}"), exit_code: 1, result: "".into() });
                continue;
            }
        };

        match input.action.as_str() {
            "exec" | "run" => exec_code(&input.code.unwrap_or_default()),
            "ping" | "" => write_output(Output { stdout: "pong".into(), stderr: "".into(), exit_code: 0, result: "ok".into() }),
            _ => write_output(Output { stdout: "".into(), stderr: format!("Unknown action: {}", input.action), exit_code: 1, result: "".into() }),
        }
    }
}
