use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::parser::Job as ParsedJob;

static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Running,
    Stopped,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
pub struct JobInfo {
    pub id: u64,
    pub command: String,
    pub pid: Option<u32>,
    pub state: JobState,
    pub background: bool,
}

pub struct JobManager {
    jobs: Arc<Mutex<HashMap<u64, JobInfo>>>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create(&self, command: &str, background: bool) -> u64 {
        let id = NEXT_JOB_ID.fetch_add(1, Ordering::SeqCst);
        let mut jobs = self.jobs.lock().unwrap();
        jobs.insert(id, JobInfo {
            id,
            command: command.to_string(),
            pid: None,
            state: JobState::Running,
            background,
        });
        id
    }

    pub fn set_pid(&self, id: u64, pid: u32) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(&id) {
            job.pid = Some(pid);
        }
    }

    pub fn set_state(&self, id: u64, state: JobState) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(&id) {
            job.state = state;
        }
    }

    pub fn get(&self, id: u64) -> Option<JobInfo> {
        self.jobs.lock().unwrap().get(&id).cloned()
    }

    pub fn list(&self) -> Vec<JobInfo> {
        let mut jobs: Vec<JobInfo> = self.jobs.lock().unwrap().values().cloned().collect();
        jobs.sort_by_key(|j| j.id);
        jobs
    }

    pub fn list_active(&self) -> Vec<JobInfo> {
        self.list().into_iter().filter(|j| j.state == JobState::Running).collect()
    }

    pub fn remove(&self, id: u64) {
        self.jobs.lock().unwrap().remove(&id);
    }

    pub fn cleanup_done(&self) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|_, j| j.state != JobState::Done && j.state != JobState::Failed);
    }

    pub fn count(&self) -> usize {
        self.jobs.lock().unwrap().len()
    }

    pub fn format_job_status(&self, id: u64) -> Option<String> {
        self.get(id).map(|info| {
            let state_str = match info.state {
                JobState::Running => "Running",
                JobState::Stopped => "Stopped",
                JobState::Done => "Done",
                JobState::Failed => "Failed",
            };
            format!("[{}] {}  {}", info.id, state_str, info.command)
        })
    }

    pub fn jobs_summary(&self) -> Vec<String> {
        self.list().iter().map(|info| {
            let state_str = match info.state {
                JobState::Running => "Running",
                JobState::Stopped => "Stopped",
                JobState::Done => "Done",
                JobState::Failed => "Failed",
            };
            let bg = if info.background { " &" } else { "" };
            format!("[{}] {}{}  {}", info.id, state_str, bg, info.command)
        }).collect()
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run_job(job: &ParsedJob) -> anyhow::Result<std::process::Child> {
    let cmd = &job.pipeline.commands[0];
    let mut command = std::process::Command::new(&cmd.program);
    command.args(&cmd.args);

    if let Some(ref stdin_path) = cmd.stdin_redirect {
        let file = std::fs::File::open(stdin_path)?;
        command.stdin(std::process::Stdio::from(file));
    }

    if let Some(ref stdout_path) = cmd.stdout_redirect {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(cmd.append_stdout)
            .truncate(!cmd.append_stdout)
            .open(stdout_path)?;
        command.stdout(std::process::Stdio::from(file));
    }

    if let Some(ref stderr_path) = cmd.stderr_redirect {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(stderr_path)?;
        command.stderr(std::process::Stdio::from(file));
    }

    let child = command.spawn()?;
    Ok(child)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let mgr = JobManager::new();
        let id = mgr.create("echo hello", false);
        assert_eq!(id, 1);
        let job = mgr.get(id).unwrap();
        assert_eq!(job.command, "echo hello");
        assert_eq!(job.state, JobState::Running);
        assert!(!job.background);
    }

    #[test]
    fn test_job_state_transitions() {
        let mgr = JobManager::new();
        let id = mgr.create("sleep 1", false);
        mgr.set_state(id, JobState::Done);
        assert_eq!(mgr.get(id).unwrap().state, JobState::Done);
    }

    #[test]
    fn test_job_list() {
        let mgr = JobManager::new();
        mgr.create("job1", false);
        mgr.create("job2", true);
        assert_eq!(mgr.count(), 2);
        assert_eq!(mgr.list_active().len(), 2);
    }

    #[test]
    fn test_job_remove() {
        let mgr = JobManager::new();
        let id = mgr.create("test", false);
        mgr.remove(id);
        assert!(mgr.get(id).is_none());
    }

    #[test]
    fn test_job_cleanup() {
        let mgr = JobManager::new();
        let id1 = mgr.create("done_job", false);
        mgr.set_state(id1, JobState::Done);
        let id2 = mgr.create("running_job", false);
        mgr.cleanup_done();
        assert!(mgr.get(id1).is_none());
        assert!(mgr.get(id2).is_some());
    }

    #[test]
    fn test_job_summary() {
        let mgr = JobManager::new();
        mgr.create("echo test", false);
        mgr.create("sleep 10 &", true);
        let summary = mgr.jobs_summary();
        assert_eq!(summary.len(), 2);
    }
}
