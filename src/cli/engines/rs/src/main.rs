use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};
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
    let json = serde_json::to_string(&o).unwrap();
    println!("{}", json);
    io::stdout().flush().ok();
}

fn exec_code(code: &str) {
    let tmp_dir = format!("/tmp/klyron-rs-{}", std::process::id());
    let _ = fs::create_dir_all(&tmp_dir);
    let src_path = Path::new(&tmp_dir).join("main.rs");
    let bin_path = Path::new(&tmp_dir).join("prog");

    fs::write(&src_path, code).unwrap();

    let compile_output = Command::new("rustc")
        .args([&src_path.to_string_lossy(), "-o", &bin_path.to_string_lossy(), "-O"])
        .output()
        .unwrap();

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        write_output(Output {
            stdout: "".into(),
            stderr: stderr.to_string(),
            exit_code: 1,
            result: "Compilation failed".into(),
        });
        let _ = fs::remove_dir_all(&tmp_dir);
        return;
    }

    let run_output = Command::new(&bin_path)
        .output()
        .unwrap();

    let _ = fs::remove_dir_all(&tmp_dir);

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
