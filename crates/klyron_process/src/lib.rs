use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::Duration;

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

pub struct ChildProcess {
    child: Child,
}

impl ChildProcess {
    #[inline]
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn status(&mut self) -> Option<ExitStatus> {
        self.child.try_wait().ok().flatten()
    }

    #[inline]
    pub fn write_stdin(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if let Some(ref mut stdin) = self.child.stdin {
            Ok(stdin.write_all(data)?)
        } else {
            anyhow::bail!("stdin not piped")
        }
    }

    pub fn wait(self) -> anyhow::Result<ProcessResult> {
        let output = self.child.wait_with_output()?;
        Ok(output.into())
    }

    pub fn kill(&mut self) -> anyhow::Result<()> {
        self.child.kill()?;
        self.child.wait()?;
        Ok(())
    }

    #[inline]
    pub fn try_wait(&mut self) -> anyhow::Result<Option<i32>> {
        match self.child.try_wait()? {
            Some(status) => Ok(Some(status.code().unwrap_or(-1))),
            None => Ok(None),
        }
    }
}

pub struct ProcessManager;

impl ProcessManager {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    pub fn spawn(&self, program: &str, args: &[&str]) -> anyhow::Result<ChildProcess> {
        let child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(ChildProcess { child })
    }

    pub fn spawn_with_pipes(
        &self,
        program: &str,
        args: &[&str],
        stdin: Stdio,
        stdout: Stdio,
        stderr: Stdio,
    ) -> anyhow::Result<ChildProcess> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let child = Command::new(program)
                .args(args)
                .stdin(stdin)
                .stdout(stdout)
                .stderr(stderr)
                .process_group(0)
                .spawn()?;
            Ok(ChildProcess { child })
        }
        #[cfg(not(unix))]
        {
            let child = Command::new(program)
                .args(args)
                .stdin(stdin)
                .stdout(stdout)
                .stderr(stderr)
                .spawn()?;
            Ok(ChildProcess { child })
        }
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

    pub fn spawn_inherit(&self, program: &str, args: &[&str]) -> anyhow::Result<ChildProcess> {
        let child = Command::new(program).args(args).spawn()?;
        Ok(ChildProcess { child })
    }

    pub fn signal(&self, pid: u32, signal: i32) -> anyhow::Result<()> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            let sig = Signal::try_from(signal)
                .map_err(|_| anyhow::anyhow!("Invalid signal: {signal}"))?;
            kill(Pid::from_raw(pid as i32), sig)?;
            Ok(())
        }
        #[cfg(not(unix))]
        {
            let _ = (pid, signal);
            anyhow::bail!("signal() not supported on this platform")
        }
    }

    fn build_exec(&self, program: &str, args: &[&str]) -> Command {
        let mut cmd = Command::new(program);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
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
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes())?;
        }
        Ok(child.wait_with_output()?.into())
    }

    pub fn piped(&self, pipeline: &[&str]) -> anyhow::Result<ProcessResult> {
        if pipeline.is_empty() {
            anyhow::bail!("Empty pipeline");
        }
        if pipeline.len() == 1 {
            return self.exec(pipeline[0], &[]);
        }

        let mut cmds: Vec<Command> = pipeline.iter().map(|p| {
            let parts: Vec<&str> = p.split_whitespace().collect();
            let mut cmd = Command::new(parts[0]);
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }
            cmd
        }).collect();

        for i in 0..cmds.len() {
            cmds[i].stdout(Stdio::piped());
            if i > 0 {
                cmds[i].stdin(Stdio::piped());
            }
        }
        cmds.last_mut().unwrap().stderr(Stdio::piped());

        let mut children: Vec<Child> = cmds.into_iter()
            .map(|mut c| c.spawn())
            .collect::<Result<Vec<_>, _>>()?;

        for i in 0..children.len().saturating_sub(1) {
            let stdout = children[i].stdout.take()
                .ok_or_else(|| anyhow::anyhow!("no stdout on pipe element {i}"))?;
            let stdin = children[i + 1].stdin.take()
                .ok_or_else(|| anyhow::anyhow!("no stdin on pipe element {}", i + 1))?;
            let mut reader = stdout;
            let mut writer = stdin;
            std::thread::spawn(move || {
                let mut buf = [0u8; 8192];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => { writer.write_all(&buf[..n]).ok(); }
                    }
                }
            });
        }

        let output = children.pop()
            .ok_or_else(|| anyhow::anyhow!("no children in pipeline"))?
            .wait_with_output()?;
        Ok(output.into())
    }

    #[inline]
    pub fn which(&self, program: &str) -> Option<String> {
        self.exec("which", &[program]).ok()
            .filter(|r| r.success)
            .map(|r| r.stdout.trim().to_string())
    }

    #[inline]
    pub fn is_running(&self, program: &str) -> bool {
        self.exec("pgrep", &["-x", program])
            .map(|r| r.success)
            .unwrap_or(false)
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
        let child = tokio::process::Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(child)
    }

    pub async fn exec_async(&self, program: &str, args: &[&str]) -> anyhow::Result<ProcessResult> {
        let output = tokio::process::Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;
        Ok(output.into())
    }

    pub async fn wait_with_timeout(
        &self,
        program: &str,
        args: &[&str],
        timeout: Duration,
    ) -> anyhow::Result<Option<ProcessResult>> {
        let mut child = tokio::process::Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                let _ = child.start_kill();
                let _ = child.wait().await;
                return Ok(None);
            }
            match child.try_wait() {
                Ok(Some(_status)) => {
                    let output = child.wait_with_output().await?;
                    return Ok(Some(output.into()));
                }
                Ok(None) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(e) => anyhow::bail!("Process error: {e}"),
            }
        }
    }
}

impl Default for ProcessManager {
    #[inline]
    fn default() -> Self {
        Self::new()
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
        let result = pm.wait_with_timeout("echo", &["fast"], Duration::from_secs(5)).await.unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().success);
    }
}
