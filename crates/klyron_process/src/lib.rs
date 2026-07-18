pub mod pipe;
pub mod spawn;

use std::process::Output;

#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub success: bool,
}

impl From<Output> for ProcessResult {
    #[inline]
    fn from(o: Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&o.stdout).to_string(),
            stderr: String::from_utf8_lossy(&o.stderr).to_string(),
            exit_code: o.status.code(),
            success: o.status.success(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub cpu: f64,
    pub mem: f64,
}

use std::path::Path;
use std::process::Command as StdCommand;

pub struct ProcessManager;

impl ProcessManager {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    pub fn spawn(&self, program: &str, args: &[&str]) -> anyhow::Result<spawn::ChildProcess> {
        spawn::spawn_simple(program, args)
    }

    pub fn spawn_with_pipes(
        &self,
        program: &str,
        args: &[&str],
        stdin: std::process::Stdio,
        stdout: std::process::Stdio,
        stderr: std::process::Stdio,
    ) -> anyhow::Result<spawn::ChildProcess> {
        spawn::spawn(spawn::SpawnOptions {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            stdin,
            stdout,
            stderr,
            ..Default::default()
        })
    }

    pub fn spawn_in_dir(&self, program: &str, args: &[&str], dir: &Path) -> anyhow::Result<spawn::ChildProcess> {
        spawn::spawn_in_dir(program, args, dir)
    }

    pub fn spawn_inherit(&self, program: &str, args: &[&str]) -> anyhow::Result<spawn::ChildProcess> {
        spawn::spawn_inherit(program, args)
    }

    pub fn signal(&self, pid: u32, signal: i32) -> anyhow::Result<()> {
        spawn::signal_process(pid, signal)
    }

    fn build_exec(&self, program: &str, args: &[&str]) -> StdCommand {
        let mut cmd = StdCommand::new(program);
        cmd.args(args).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
        cmd
    }

    pub fn exec(&self, program: &str, args: &[&str]) -> anyhow::Result<ProcessResult> {
        Ok(self.build_exec(program, args).output()?.into())
    }

    pub fn exec_in_dir(&self, program: &str, args: &[&str], dir: &Path) -> anyhow::Result<ProcessResult> {
        let mut cmd = self.build_exec(program, args);
        cmd.current_dir(dir);
        Ok(cmd.output()?.into())
    }

    pub fn exec_with_env(
        &self, program: &str, args: &[&str], envs: &[(String, String)],
    ) -> anyhow::Result<ProcessResult> {
        let mut cmd = self.build_exec(program, args);
        for (k, v) in envs {
            cmd.env(k, v);
        }
        Ok(cmd.output()?.into())
    }

    pub fn exec_with_stdin(
        &self, program: &str, args: &[&str], input: &str,
    ) -> anyhow::Result<ProcessResult> {
        let mut child = StdCommand::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(input.as_bytes())?;
        }
        Ok(child.wait_with_output()?.into())
    }

    pub fn piped(&self, pipeline: &[&str]) -> anyhow::Result<ProcessResult> {
        pipe::piped(pipeline)
    }

    #[inline]
    pub fn which(&self, program: &str) -> Option<String> {
        spawn::which(program)
    }

    #[inline]
    pub fn is_running(&self, program: &str) -> bool {
        spawn::is_running(program)
    }

    pub fn list_processes(&self) -> anyhow::Result<Vec<ProcessInfo>> {
        let result = self.exec("ps", &["-eo", "pid,ppid,comm,%cpu,%mem"])?;
        let mut processes = Vec::new();
        for line in result.stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                if let Ok(pid) = parts[0].parse::<u32>() {
                    processes.push(ProcessInfo {
                        pid,
                        ppid: parts[1].parse().ok(),
                        name: parts[2..parts.len() - 2].join(" "),
                        cpu: parts[parts.len() - 2].parse().unwrap_or(0.0),
                        mem: parts[parts.len() - 1].parse().unwrap_or(0.0),
                    });
                }
            }
        }
        Ok(processes)
    }

    pub async fn spawn_async(&self, program: &str, args: &[&str]) -> anyhow::Result<tokio::process::Child> {
        spawn::spawn_async(program, args).await
    }

    pub async fn exec_async(&self, program: &str, args: &[&str]) -> anyhow::Result<ProcessResult> {
        spawn::exec_async(program, args).await
    }

    pub async fn wait_with_timeout(
        &self,
        program: &str,
        args: &[&str],
        timeout: std::time::Duration,
    ) -> anyhow::Result<Option<ProcessResult>> {
        spawn::wait_with_timeout(program, args, timeout).await
    }
}

impl Default for ProcessManager {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub fn exec(program: &str, args: &[&str]) -> anyhow::Result<ProcessResult> {
    ProcessManager::new().exec(program, args)
}

#[inline]
pub fn exec_with_stdin(program: &str, args: &[&str], input: &str) -> anyhow::Result<ProcessResult> {
    ProcessManager::new().exec_with_stdin(program, args, input)
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

    #[test]
    fn test_exec_with_stdin() {
        let result = exec_with_stdin("cat", &[], "hello pipe").unwrap();
        assert_eq!(result.stdout.trim(), "hello pipe");
    }

    #[test]
    fn test_spawn_pipe() {
        let pm = ProcessManager::new();
        let result = pm.piped(&["echo hello", "wc -c"]).unwrap();
        assert!(result.success);
        let count: usize = result.stdout.trim().parse().unwrap_or(0);
        assert!(count > 0);
    }

    #[test]
    fn test_child_pid() {
        let pm = ProcessManager::new();
        let child = pm.spawn("echo", &["hi"]).unwrap();
        assert!(child.pid() > 0);
    }

    #[tokio::test]
    async fn test_exec_async() {
        let pm = ProcessManager::new();
        let result = pm.exec_async("echo", &["async"]).await.unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "async");
    }

    #[tokio::test]
    async fn test_wait_with_timeout() {
        let pm = ProcessManager::new();
        let result = pm.wait_with_timeout("echo", &["fast"], std::time::Duration::from_secs(5)).await.unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().success);
    }
}
