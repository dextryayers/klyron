use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineInput {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileEntry>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    #[serde(default)]
    pub result: String,
}

pub struct EngineProcess {
    child: Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl EngineProcess {
    pub fn spawn(program: &str, args: &[&str]) -> anyhow::Result<Self> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn {}: {}", program, e))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdin for {}", program))?;
        let stdout = BufReader::new(child.stdout.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout for {}", program))?);

        Ok(Self { child, stdin, stdout })
    }

    pub fn communicate(&mut self, input: &EngineInput) -> anyhow::Result<EngineOutput> {
        self.communicate_with_timeout(input, Duration::from_secs(30))
    }

    pub fn communicate_with_timeout(&mut self, input: &EngineInput, timeout: Duration) -> anyhow::Result<EngineOutput> {
        let json = serde_json::to_string(input)?;
        self.stdin.write_all(json.as_bytes())?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;

        let mut line = String::new();
        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > timeout {
                let _ = self.child.kill();
                anyhow::bail!("Engine timed out after {:?}", timeout);
            }
            if let Some(status) = self.child.try_wait().ok().flatten() {
                let _ = self.stdin.write_all(b"\n");
                let _ = self.stdin.flush();
                self.stdout.read_line(&mut line).ok();
                let stderr = {
                    let mut buf = String::new();
                    if let Some(mut stderr) = self.child.stderr.take().map(|s| BufReader::new(s)) {
                        stderr.read_to_string(&mut buf).ok();
                    }
                    buf
                };
                if !line.trim().is_empty() {
                    if let Ok(output) = serde_json::from_str::<EngineOutput>(&line.trim().to_string()) {
                        return Ok(EngineOutput { stderr, ..output });
                    }
                }
                anyhow::bail!("Engine exited prematurely with code: {}", status);
            }
            self.stdout.read_line(&mut line).ok();
            if !line.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        line = line.trim().to_string();
        if line.is_empty() {
            anyhow::bail!("Engine returned empty response");
        }

        let output: EngineOutput = serde_json::from_str(&line)
            .map_err(|e| anyhow::anyhow!("Invalid JSON from engine: {} — raw: {}", e, &line[..line.len().min(200)]))?;
        Ok(output)
    }
}

impl Drop for EngineProcess {
    fn drop(&mut self) {
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn find_engine_path(name: &str) -> String {
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| {
        let cwd = std::env::current_dir().ok()
            .and_then(|p| p.parent().map(|pp| pp.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("target"));
        let release = cwd.join("target/release");
        let debug = cwd.join("target/debug");
        if release.join(name).exists() {
            release.to_string_lossy().to_string()
        } else {
            debug.to_string_lossy().to_string()
        }
    });
    Path::new(&out_dir).join(name).to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_input_serialization() {
        let input = EngineInput {
            action: "eval".to_string(),
            code: Some("1+1".to_string()),
            args: None,
            filename: None,
            project: None,
            files: None,
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("\"action\":\"eval\""));
        assert!(json.contains("\"code\":\"1+1\""));
    }

    #[test]
    fn test_engine_input_with_all_fields() {
        let input = EngineInput {
            action: "run".to_string(),
            code: Some("console.log('hi')".to_string()),
            args: Some("--debug".to_string()),
            filename: Some("test.js".to_string()),
            project: Some("my_project".to_string()),
            files: Some(vec![
                FileEntry { name: "lib.js".to_string(), content: "export const x = 1;".to_string() }
            ]),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("\"action\":\"run\""));
        assert!(json.contains("\"project\":\"my_project\""));
        assert!(json.contains("lib.js"));
    }

    #[test]
    fn test_engine_output_deserialization() {
        let json = r#"{"stdout":"hello","stderr":"","exit_code":0,"result":"ok"}"#;
        let output: EngineOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.stdout, "hello");
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.result, "ok");
    }

    #[test]
    fn test_engine_output_with_error() {
        let json = r#"{"stdout":"","stderr":"error msg","exit_code":1,"result":""}"#;
        let output: EngineOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.stderr, "error msg");
        assert_eq!(output.exit_code, 1);
    }

    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry {
            name: "module.js".to_string(),
            content: "export default 42;".to_string(),
        };
        assert_eq!(entry.name, "module.js");
        assert_eq!(entry.content, "export default 42;");
    }

    #[test]
    fn test_engine_input_optional_fields_skipped() {
        let input = EngineInput {
            action: "eval".to_string(),
            code: None,
            args: None,
            filename: None,
            project: None,
            files: None,
        };
        let json = serde_json::to_string(&input).unwrap();
        // optional fields should be skipped via skip_serializing_if
        assert!(!json.contains("\"code\""));
        assert!(!json.contains("\"args\""));
        assert!(!json.contains("\"filename\""));
    }

    #[test]
    fn test_engine_output_default_result() {
        let json = r#"{"stdout":"out","stderr":"err","exit_code":0}"#;
        let output: EngineOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.stdout, "out");
        assert_eq!(output.stderr, "err");
        assert_eq!(output.result, "");
    }

    #[test]
    fn test_engine_output_roundtrip() {
        let output = EngineOutput {
            stdout: "stdout".to_string(),
            stderr: "stderr".to_string(),
            exit_code: 42,
            result: "result".to_string(),
        };
        let json = serde_json::to_string(&output).unwrap();
        let deserialized: EngineOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.stdout, "stdout");
        assert_eq!(deserialized.stderr, "stderr");
        assert_eq!(deserialized.exit_code, 42);
        assert_eq!(deserialized.result, "result");
    }
}
