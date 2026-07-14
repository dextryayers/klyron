use crate::types::{ProcessResult, Result};
use std::process::{Command, Output, Stdio};

pub fn run_command(cmd: &str, args: &[&str]) -> Result<i32> {
    let status = Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(status.code().unwrap_or(-1))
}

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

pub fn exec(cmd: &str) -> Result<ProcessResult> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()?
    } else {
        Command::new("sh").args(["-c", cmd]).output()?
    };
    Ok(ProcessResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        success: output.status.success(),
    })
}

pub fn spawn(cmd: &str, args: &[&str]) -> Result<u32> {
    let child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(child.id())
}

pub fn kill(pid: u32) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let status = Command::new("kill")
            .arg(pid.to_string())
            .status()?;
        if !status.success() {
            return Err(format!("Failed to kill process {}", pid).into());
        }
    }
    #[cfg(windows)]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()?;
        if !status.success() {
            return Err(format!("Failed to kill process {}", pid).into());
        }
    }
    Ok(())
}

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

pub fn pipe_commands(cmds: &[(&str, &[&str])]) -> Result<String> {
    if cmds.is_empty() {
        return Err("No commands provided".into());
    }
    if cmds.len() == 1 {
        return Ok(capture_output(cmds[0].0, cmds[0].1)?.0);
    }

    let mut prev_stdout = None;
    for (i, (cmd, args)) in cmds.iter().enumerate() {
        let mut command = Command::new(cmd);
        command.args(args);

        if let Some(stdout) = prev_stdout.take() {
            command.stdin(Stdio::piped());
            if let Some(mut stdin) = command.stdin() {
                stdin.write_all(stdout.as_bytes());
            }
        }

        if i == cmds.len() - 1 {
            let output = command.output()?;
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        } else {
            let output = command.output()?;
            prev_stdout = Some(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }
    Err("Pipeline failed".into())
}

pub fn background(cmd: &str, args: &[&str]) -> Result<u32> {
    let child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()?;
    Ok(child.id())
}

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
