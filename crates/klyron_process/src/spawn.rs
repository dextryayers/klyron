use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::ProcessResult;

pub struct ChildProcess {
    child: Child,
}

impl ChildProcess {
    #[inline]
    pub fn new(child: Child) -> Self {
        Self { child }
    }

    #[inline]
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn status(&mut self) -> Option<std::process::ExitStatus> {
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

    pub fn try_wait_status(&mut self) -> anyhow::Result<Option<std::process::ExitStatus>> {
        Ok(self.child.try_wait()?)
    }
}

pub struct SpawnOptions {
    pub program: String,
    pub args: Vec<String>,
    pub env: Option<Vec<(String, String)>>,
    pub current_dir: Option<std::path::PathBuf>,
    pub stdin: Stdio,
    pub stdout: Stdio,
    pub stderr: Stdio,
    pub process_group: bool,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self {
            program: String::new(),
            args: Vec::new(),
            env: None,
            current_dir: None,
            stdin: Stdio::piped(),
            stdout: Stdio::piped(),
            stderr: Stdio::piped(),
            process_group: true,
            uid: None,
            gid: None,
        }
    }
}

pub fn spawn(opts: SpawnOptions) -> anyhow::Result<ChildProcess> {
    let mut cmd = Command::new(&opts.program);
    cmd.args(&opts.args)
        .stdin(opts.stdin)
        .stdout(opts.stdout)
        .stderr(opts.stderr);

    if let Some(dir) = opts.current_dir {
        cmd.current_dir(dir);
    }
    if let Some(env) = opts.env {
        for (k, v) in env {
            cmd.env(k, v);
        }
    }

    #[cfg(unix)]
    if opts.process_group {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(unix)]
    if let Some(uid) = opts.uid {
        cmd.uid(uid);
    }
    #[cfg(unix)]
    if let Some(gid) = opts.gid {
        cmd.gid(gid);
    }

    let child = cmd.spawn()?;
    Ok(ChildProcess::new(child))
}

pub fn spawn_simple(program: &str, args: &[&str]) -> anyhow::Result<ChildProcess> {
    spawn(SpawnOptions {
        program: program.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    })
}

pub fn spawn_in_dir(program: &str, args: &[&str], dir: &Path) -> anyhow::Result<ChildProcess> {
    spawn(SpawnOptions {
        program: program.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
        current_dir: Some(dir.to_path_buf()),
        ..Default::default()
    })
}

pub fn spawn_inherit(program: &str, args: &[&str]) -> anyhow::Result<ChildProcess> {
    spawn(SpawnOptions {
        program: program.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
        stdin: Stdio::inherit(),
        stdout: Stdio::inherit(),
        stderr: Stdio::inherit(),
        ..Default::default()
    })
}

pub async fn spawn_async(program: &str, args: &[&str]) -> anyhow::Result<tokio::process::Child> {
    let child = tokio::process::Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    Ok(child)
}

pub async fn exec_async(program: &str, args: &[&str]) -> anyhow::Result<ProcessResult> {
    let output = tokio::process::Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;
    Ok(output.into())
}

pub async fn wait_with_timeout(
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

pub fn signal_process(pid: u32, signal: i32) -> anyhow::Result<()> {
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

pub fn which(program: &str) -> Option<String> {
    Command::new("which").arg(program).output().ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

pub fn is_running(program: &str) -> bool {
    Command::new("pgrep").args(["-x", program]).output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
