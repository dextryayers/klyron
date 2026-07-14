use std::path::Path;
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub success: bool,
}

pub struct ProcessManager;

impl ProcessManager {
    pub fn new() -> Self { Self }

    pub fn spawn(&self, program: &str, args: &[&str]) -> anyhow::Result<ChildProcess> {
        let child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(ChildProcess { child })
    }

    pub fn spawn_in_dir(&self, program: &str, args: &[&str], dir: &Path) -> anyhow::Result<ChildProcess> {
        let child = Command::new(program)
            .args(args)
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(ChildProcess { child })
    }

    pub fn exec(&self, program: &str, args: &[&str]) -> anyhow::Result<ProcessResult> {
        let output = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        Ok(ProcessResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            success: output.status.success(),
        })
    }

    pub fn exec_in_dir(&self, program: &str, args: &[&str], dir: &Path) -> anyhow::Result<ProcessResult> {
        let output = Command::new(program)
            .args(args)
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        Ok(ProcessResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            success: output.status.success(),
        })
    }

    pub fn exec_with_env(&self, program: &str, args: &[&str], envs: &[(String, String)]) -> anyhow::Result<ProcessResult> {
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let output = cmd.output()?;
        Ok(ProcessResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            success: output.status.success(),
        })
    }

    pub fn which(&self, program: &str) -> Option<String> {
        let result = self.exec("which", &[program]).ok()?;
        if result.success {
            Some(result.stdout.trim().to_string())
        } else {
            None
        }
    }

    pub fn is_running(&self, program: &str) -> bool {
        self.exec("pgrep", &["-x", program])
            .map(|r| r.success)
            .unwrap_or(false)
    }
}

pub struct ChildProcess {
    child: Child,
}

impl ChildProcess {
    pub fn wait(self) -> anyhow::Result<ProcessResult> {
        let output = self.child.wait_with_output()?;
        Ok(ProcessResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            success: output.status.success(),
        })
    }

    pub fn kill(&mut self) -> anyhow::Result<()> {
        self.child.kill()?;
        self.child.wait()?;
        Ok(())
    }

    pub fn try_wait(&mut self) -> anyhow::Result<Option<i32>> {
        match self.child.try_wait()? {
            Some(status) => Ok(Some(status.code().unwrap_or(-1))),
            None => Ok(None),
        }
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }
}

pub fn exec(program: &str, args: &[&str]) -> anyhow::Result<ProcessResult> {
    ProcessManager::new().exec(program, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_exec_echo() {
        let result = exec("echo", &["hello"]).unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello");
    }

    #[test]
    fn test_exec_false() {
        let result = exec("false", &[]).unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(1));
    }
}
