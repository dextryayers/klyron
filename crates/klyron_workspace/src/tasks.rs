use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::{Context, Result};
use chrono::Utc;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{CacheEntry, CacheConfig, TaskResult, Workspace, WorkspaceConfig, WorkspaceMember};

impl Workspace {
    pub fn run_task_parallel(
        dir: &Path,
        command: &str,
        args: &[&str],
    ) -> Result<Vec<TaskResult>> {
        let members = Self::list_members(dir)?;
        if members.is_empty() {
            anyhow::bail!("no workspace members found");
        }

        let results: Vec<TaskResult> = members
            .par_iter()
            .map(|member| {
                let start = Instant::now();
                let output = Command::new(command)
                    .args(args)
                    .current_dir(&member.path)
                    .output();
                let duration = start.elapsed().as_secs_f64();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        TaskResult {
                            member: member.name.clone(),
                            success: out.status.success(),
                            duration,
                            output: format!("{stdout}\n{stderr}"),
                            cached: false,
                        }
                    }
                    Err(e) => TaskResult {
                        member: member.name.clone(),
                        success: false,
                        duration,
                        output: format!("{e:#}"),
                        cached: false,
                    },
                }
            })
            .collect();

        Ok(results)
    }

    pub fn run_affected(
        dir: &Path,
        base_ref: &str,
        command: &str,
        args: &[&str],
    ) -> Result<Vec<TaskResult>> {
        let affected = Self::get_affected_members(dir, base_ref)?;
        if affected.is_empty() {
            return Ok(Vec::new());
        }

        let results: Vec<TaskResult> = affected
            .par_iter()
            .map(|member| {
                let start = Instant::now();
                let output = Command::new(command)
                    .args(args)
                    .current_dir(&member.path)
                    .output();
                let duration = start.elapsed().as_secs_f64();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        TaskResult {
                            member: member.name.clone(),
                            success: out.status.success(),
                            duration,
                            output: format!("{stdout}\n{stderr}"),
                            cached: false,
                        }
                    }
                    Err(e) => TaskResult {
                        member: member.name.clone(),
                        success: false,
                        duration,
                        output: format!("{e:#}"),
                        cached: false,
                    },
                }
            })
            .collect();

        Ok(results)
    }

    pub fn run_with_cache(
        &self,
        dir: &Path,
        member: &WorkspaceMember,
        task_name: &str,
        command: &str,
        args: &[&str],
    ) -> Result<TaskResult> {
        let config = Self::detect(dir).unwrap_or(WorkspaceConfig {
            members: vec![],
            name: "ws".into(),
            version: "0.1.0".into(),
            shared_deps: std::collections::HashMap::new(),
            pipeline: std::collections::HashMap::new(),
            cache_config: CacheConfig::default(),
        });

        let cache_key = format!("{}:{}:{}", member.name, task_name, member.hash);

        if config.cache_config.enabled {
            if let Ok(cache) = self.cache.lock() {
                if let Some(entry) = cache.get(&cache_key) {
                    if entry.hash == member.hash {
                        return Ok(TaskResult {
                            member: member.name.clone(),
                            success: true,
                            duration: 0.0,
                            output: "(cached)".into(),
                            cached: true,
                        });
                    }
                }
            }
        }

        let start = Instant::now();
        let output = Command::new(command)
            .args(args)
            .current_dir(&member.path)
            .output();
        let duration = start.elapsed().as_secs_f64();

        let result = match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                TaskResult {
                    member: member.name.clone(),
                    success: out.status.success(),
                    duration,
                    output: format!("{stdout}\n{stderr}"),
                    cached: false,
                }
            }
            Err(e) => TaskResult {
                member: member.name.clone(),
                success: false,
                duration,
                output: format!("{e:#}"),
                cached: false,
            },
        };

        if config.cache_config.enabled && result.success {
            let entry = CacheEntry {
                hash: member.hash.clone(),
                timestamp: Utc::now(),
                outputs: std::collections::HashMap::new(),
            };
            if let Ok(mut cache) = self.cache.lock() {
                cache.insert(cache_key.clone(), entry);
            }

            if let Some(ref remote_url) = config.cache_config.remote_url {
                if let Ok(json) =
                    serde_json::to_string(&self.cache.lock().unwrap().get(&cache_key))
                {
                    let _ = Self::put_remote_cache(remote_url, &cache_key, &json);
                }
            }
        }

        Ok(result)
    }

    fn put_remote_cache(url: &str, key: &str, data: &str) -> Result<()> {
        let endpoint = format!("{url}/cache/{key}");
        let result = std::process::Command::new("curl")
            .args([
                "-X",
                "PUT",
                "-H",
                "Content-Type: application/json",
                "-d",
                data,
                &endpoint,
            ])
            .output();
        match result {
            Ok(out) if out.status.success() => Ok(()),
            Ok(out) => anyhow::bail!(
                "curl failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
            Err(e) => anyhow::bail!("failed to run curl: {e}"),
        }
    }

    pub fn run_script(dir: &Path, script: &str) -> Result<()> {
        let members = Self::list_members(dir)?;
        if members.is_empty() {
            anyhow::bail!("no workspace members found");
        }
        for member in &members {
            eprintln!("[{}] running '{}'...", member.name, script);
            let status = Command::new("sh")
                .args(["-c", script])
                .current_dir(&member.path)
                .status()
                .with_context(|| format!("failed to run script in {}", member.name))?;
            if !status.success() {
                anyhow::bail!(
                    "script '{}' failed in {} with exit code {:?}",
                    script,
                    member.name,
                    status.code()
                );
            }
        }
        Ok(())
    }

    pub fn exec_in_members(dir: &Path, cmd: &str, args: &[&str]) -> Result<()> {
        let members = Self::list_members(dir)?;
        if members.is_empty() {
            anyhow::bail!("no workspace members found");
        }
        for member in &members {
            eprintln!(
                "[{}] executing '{} {}'...",
                member.name,
                cmd,
                args.join(" ")
            );
            let status = Command::new(cmd)
                .args(args)
                .current_dir(&member.path)
                .status()
                .with_context(|| format!("failed to execute in {}", member.name))?;
            if !status.success() {
                anyhow::bail!(
                    "command '{} {}' failed in {} with exit code {:?}",
                    cmd,
                    args.join(" "),
                    member.name,
                    status.code()
                );
            }
        }
        Ok(())
    }
}
