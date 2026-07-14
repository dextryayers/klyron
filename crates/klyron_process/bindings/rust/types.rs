#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct ChildProcess;
impl ChildProcess {
    pub fn id(&self) -> u32 { 0 }
    pub fn pid(&self) -> u32 { 0 }
    pub fn wait(self) -> anyhow::Result<ProcessResult> { anyhow::bail!("wait not available in bindings") }
    pub fn kill(&mut self) -> anyhow::Result<()> { Ok(()) }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub cpu: f64,
    pub mem: f64,
}

pub struct ProcessManager;
impl ProcessManager {
    pub fn new() -> Self { Self }
    pub fn spawn(&self, program: &str, args: &[&str]) -> anyhow::Result<ChildProcess> { let _ = (program, args); Ok(ChildProcess) }
    pub fn spawn_in_dir(&self, program: &str, args: &[&str], dir: &std::path::Path) -> anyhow::Result<ChildProcess> { let _ = (program, args, dir); Ok(ChildProcess) }
    pub fn exec(&self, program: &str, args: &[&str]) -> anyhow::Result<ProcessResult> { let _ = (program, args); anyhow::bail!("exec not available in bindings") }
    pub fn exec_with_stdin(&self, program: &str, args: &[&str], input: &str) -> anyhow::Result<ProcessResult> { let _ = (program, args, input); anyhow::bail!("exec_with_stdin not available in bindings") }
    pub fn piped(&self, pipeline: &[&str]) -> anyhow::Result<ProcessResult> { let _ = pipeline; anyhow::bail!("piped not available in bindings") }
    pub fn which(&self, program: &str) -> Option<String> { let _ = program; None }
    pub fn is_running(&self, program: &str) -> bool { let _ = program; false }
}
impl Default for ProcessManager { fn default() -> Self { Self::new() } }
