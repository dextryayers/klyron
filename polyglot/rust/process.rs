//! Process execution helpers.
//!
//! Functions for running commands, capturing output,
//! and locating executables on the system PATH.

use crate::types::Result;
use std::process::{Command, Output, Stdio};

/// Run a command with arguments, inheriting stdio streams.
///
/// Returns the exit code (or `-1` if terminated by signal).
///
/// ```
/// use klyron_rust::process::run_command;
/// let code = run_command("echo", &["hello"]).unwrap();
/// ```
pub fn run_command(cmd: &str, args: &[&str]) -> Result<i32> {
    let status = Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(status.code().unwrap_or(-1))
}

/// Run a command and capture its stdout and stderr as strings.
///
/// Returns a tuple of `(stdout, stderr, exit_code)`.
///
/// ```
/// use klyron_rust::process::capture_output;
/// let (out, err, code) = capture_output("echo", &["hello"]).unwrap();
/// ```
pub fn capture_output(cmd: &str, args: &[&str]) -> Result<(String, String, i32)> {
    let output: Output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    Ok((stdout, stderr, code))
}

/// Locate an executable on the system `PATH`.
///
/// Returns `Some(full_path)` if found, or `None` otherwise.
///
/// ```
/// use klyron_rust::process::which;
/// assert!(which("sh").is_some());
/// ```
pub fn which(program: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let full_path = dir.join(program);
        if full_path.is_file() {
            return Some(full_path.to_string_lossy().to_string());
        }
        #[cfg(windows)]
        {
            let with_exe = dir.join(format!("{}.exe", program));
            if with_exe.is_file() {
                return Some(with_exe.to_string_lossy().to_string());
            }
        }
    }
    None
}
