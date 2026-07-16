use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_scripts() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("build".into(), "echo build".into());
        m.insert("test".into(), "echo test".into());
        m.insert("prebuild".into(), "echo prebuild".into());
        m.insert("postbuild".into(), "echo postbuild".into());
        m.insert("start".into(), "echo start".into());
        m
    }

    #[test]
    fn test_script_config_creation() {
        let scripts = make_scripts();
        let hooks = get_lifecycle_scripts(&scripts);
        let config = ScriptConfig { scripts: scripts.clone(), lifecycle_hooks: hooks };
        assert!(config.scripts.contains_key("build"));
        assert!(config.lifecycle_hooks.prebuild.is_some());
    }

    #[test]
    fn test_get_lifecycle_scripts() {
        let scripts = make_scripts();
        let hooks = get_lifecycle_scripts(&scripts);
        assert_eq!(hooks.prebuild.as_deref(), Some("echo prebuild"));
        assert_eq!(hooks.build.as_deref(), Some("echo build"));
        assert_eq!(hooks.postbuild.as_deref(), Some("echo postbuild"));
        assert!(hooks.install.is_none());
    }

    #[test]
    fn test_get_lifecycle_order() {
        let order = get_lifecycle_order("build");
        assert_eq!(order[0], "prebuild");
        assert_eq!(order[1], "build");
        assert_eq!(order[2], "postbuild");
    }

    #[test]
    fn test_lifecycle_events_constant() {
        assert!(LIFECYCLE_EVENTS.contains(&"preinstall"));
        assert!(LIFECYCLE_EVENTS.contains(&"postinstall"));
        assert!(LIFECYCLE_EVENTS.contains(&"build"));
        assert!(LIFECYCLE_EVENTS.contains(&"test"));
        assert!(LIFECYCLE_EVENTS.contains(&"start"));
        assert_eq!(LIFECYCLE_EVENTS.len(), 18);
    }

    #[test]
    fn test_script_runner_creation() {
        let runner = ScriptRunner::new(std::path::Path::new("/tmp"));
        assert_eq!(runner.shell, "sh");
        assert_eq!(runner.shell_args, vec!["-c"]);
    }

    #[test]
    fn test_script_runner_with_env() {
        let env: HashMap<String, String> = [("NODE_ENV".into(), "test".into())].into();
        let runner = ScriptRunner::new(std::path::Path::new("/tmp")).with_env(env);
        assert_eq!(runner.env.get("NODE_ENV").unwrap(), "test");
    }

    #[test]
    fn test_script_runner_with_shell() {
        let runner = ScriptRunner::new(std::path::Path::new("/tmp"))
            .with_shell("bash", vec!["-c"]);
        assert_eq!(runner.shell, "bash");
    }

    #[test]
    fn test_find_script_references_none() {
        let names: HashSet<&str> = ["build", "test"].into();
        let refs = find_script_references("echo hello", &names);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_find_script_references_run() {
        let names: HashSet<&str> = ["build", "test"].into();
        let refs = find_script_references("npm run build", &names);
        assert!(refs.contains(&"build"));
    }

    #[test]
    fn test_find_script_references_yarn() {
        let names: HashSet<&str> = ["test"].into();
        let refs = find_script_references("yarn test", &names);
        assert!(refs.contains(&"test"));
    }

    #[test]
    fn test_detect_no_circular() {
        let mut scripts = HashMap::new();
        scripts.insert("build".into(), "echo build".into());
        scripts.insert("test".into(), "npm run build".into());
        let runner = ScriptRunner::new(std::path::Path::new("/tmp"));
        let cycles = runner.detect_circular_scripts(&scripts);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_detect_circular() {
        let mut scripts = HashMap::new();
        scripts.insert("a".into(), "npm run b".into());
        scripts.insert("b".into(), "npm run a".into());
        let runner = ScriptRunner::new(std::path::Path::new("/tmp"));
        let cycles = runner.detect_circular_scripts(&scripts);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_sort_by_dependency() {
        let packages = vec![
            ("a".to_string(), [("build".into(), "echo a".into())].into()),
            ("b".to_string(), [("build".into(), "npm run build".into())].into()),
        ];
        let runner = ScriptRunner::new(std::path::Path::new("/tmp"));
        let _ = runner.sort_by_dependency(&packages);
    }

    #[test]
    fn test_run_result_format_output() {
        let result = RunResult {
            exit_code: 0, stdout: "out".into(), stderr: "".into(),
            duration: std::time::Duration::from_millis(100), success: true,
        };
        let formatted = result.format_output(false);
        assert!(formatted.contains("exit: 0"));
    }

    #[test]
    fn test_run_result_check_success() {
        let ok = RunResult {
            exit_code: 0, stdout: "".into(), stderr: "".into(),
            duration: std::time::Duration::default(), success: true,
        };
        assert!(ok.check_success().is_ok());

        let fail = RunResult {
            exit_code: 1, stdout: "".into(), stderr: "error".into(),
            duration: std::time::Duration::default(), success: false,
        };
        assert!(fail.check_success().is_err());
    }

    #[test]
    fn test_script_error_display() {
        let err = ScriptError::NotFound("test".into());
        assert_eq!(err.to_string(), "Script not found: test");

        let err = ScriptError::CircularDependency(vec!["a".into(), "b".into()]);
        assert_eq!(err.to_string(), "Circular dependency: a -> b");
    }

    #[test]
    fn test_lifecycle_hooks_empty() {
        let scripts = HashMap::new();
        let hooks = get_lifecycle_scripts(&scripts);
        assert!(hooks.prebuild.is_none());
        assert!(hooks.build.is_none());
        assert!(hooks.install.is_none());
    }

    #[test]
    fn test_run_result_format_output_verbose() {
        let result = RunResult {
            exit_code: 0, stdout: "hello".into(), stderr: "".into(),
            duration: std::time::Duration::from_millis(50), success: true,
        };
        let formatted = result.format_output(true);
        assert!(formatted.contains("stdout"));
        assert!(formatted.contains("hello"));
    }
}

#[derive(Debug, Clone)]
pub struct ScriptConfig {
    pub scripts: HashMap<String, String>,
    pub lifecycle_hooks: LifecycleHooks,
}

#[derive(Debug, Clone)]
pub struct LifecycleHooks {
    pub preinstall: Option<String>,
    pub install: Option<String>,
    pub postinstall: Option<String>,
    pub prebuild: Option<String>,
    pub build: Option<String>,
    pub postbuild: Option<String>,
    pub predev: Option<String>,
    pub dev: Option<String>,
    pub postdev: Option<String>,
    pub pretest: Option<String>,
    pub test: Option<String>,
    pub posttest: Option<String>,
    pub prepublish: Option<String>,
    pub publish: Option<String>,
    pub postpublish: Option<String>,
    pub prestart: Option<String>,
    pub start: Option<String>,
    pub poststart: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: std::time::Duration,
    pub success: bool,
}

#[derive(Debug)]
pub enum ScriptError {
    NotFound(String),
    CircularDependency(Vec<String>),
    ExecutionError { script: String, exit_code: i32, stderr: String },
    Io(std::io::Error),
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "Script not found: {name}"),
            Self::CircularDependency(chain) => {
                write!(f, "Circular dependency: {}", chain.join(" -> "))
            }
            Self::ExecutionError { script, exit_code, stderr } => {
                write!(f, "Script '{script}' failed (exit {exit_code}): {stderr}")
            }
            Self::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for ScriptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ScriptError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Debug, Clone)]
pub struct ScriptRunner {
    cwd: std::path::PathBuf,
    env: HashMap<String, String>,
    shell: String,
    shell_args: Vec<String>,
}

impl ScriptRunner {
    pub fn new(cwd: &Path) -> Self {
        Self {
            cwd: cwd.to_path_buf(),
            env: HashMap::new(),
            shell: "sh".to_string(),
            shell_args: vec!["-c".to_string()],
        }
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    pub fn with_shell(mut self, shell: &str, args: Vec<&str>) -> Self {
        self.shell = shell.to_string();
        self.shell_args = args.into_iter().map(String::from).collect();
        self
    }

    pub fn run(&self, script: &str) -> RunResult {
        let start = Instant::now();
        let output = Command::new(&self.shell)
            .args(&self.shell_args)
            .arg(script)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .output();

        match output {
            Ok(out) => {
                let duration = start.elapsed();
                RunResult {
                    exit_code: out.status.code().unwrap_or(-1),
                    stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                    duration,
                    success: out.status.success(),
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                RunResult {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("{e}"),
                    duration,
                    success: false,
                }
            }
        }
    }

    pub fn run_script(
        &self,
        name: &str,
        scripts: &HashMap<String, String>,
    ) -> Result<RunResult, ScriptError> {
        let script = scripts
            .get(name)
            .ok_or_else(|| ScriptError::NotFound(name.to_string()))?;
        let result = self.run(script);
        if result.success {
            Ok(result)
        } else {
            Err(ScriptError::ExecutionError {
                script: name.to_string(),
                exit_code: result.exit_code,
                stderr: result.stderr.clone(),
            })
        }
    }

    pub fn run_lifecycle(
        &self,
        hook: &str,
        scripts: &HashMap<String, String>,
    ) -> Result<Option<RunResult>, ScriptError> {
        match scripts.get(hook) {
            Some(script) => {
                let result = self.run(script);
                if result.success {
                    Ok(Some(result))
                } else {
                    Err(ScriptError::ExecutionError {
                        script: hook.to_string(),
                        exit_code: result.exit_code,
                        stderr: result.stderr.clone(),
                    })
                }
            }
            None => Ok(None),
        }
    }

    pub fn run_script_with_lifecycle(
        &self,
        name: &str,
        scripts: &HashMap<String, String>,
    ) -> Result<RunResult, ScriptError> {
        self.run_pre_hook(name, scripts)?;

        let main_result = match self.run_script(name, scripts) {
            Ok(r) => r,
            Err(e) => {
                let failed = RunResult {
                    exit_code: match &e {
                        ScriptError::ExecutionError { exit_code, .. } => *exit_code,
                        _ => 1,
                    },
                    stdout: String::new(),
                    stderr: format!("{e}"),
                    duration: std::time::Duration::default(),
                    success: false,
                };
                let _ = self.run_post_hook(name, scripts, &failed)?;
                return Err(e);
            }
        };

        self.run_post_hook(name, scripts, &main_result)?;
        Ok(main_result)
    }

    fn run_pre_hook(
        &self,
        name: &str,
        scripts: &HashMap<String, String>,
    ) -> Result<Option<RunResult>, ScriptError> {
        let pre_name = format!("pre{name}");
        self.run_lifecycle(&pre_name, scripts)
    }

    fn run_post_hook(
        &self,
        name: &str,
        scripts: &HashMap<String, String>,
        _main_result: &RunResult,
    ) -> Result<Option<RunResult>, ScriptError> {
        let post_name = format!("post{name}");
        match scripts.get(&post_name) {
            Some(script) => {
                let result = self.run(script);
                if result.success {
                    Ok(Some(result))
                } else {
                    Err(ScriptError::ExecutionError {
                        script: post_name,
                        exit_code: result.exit_code,
                        stderr: result.stderr.clone(),
                    })
                }
            }
            None => Ok(None),
        }
    }

    pub fn detect_circular_scripts(
        &self,
        scripts: &HashMap<String, String>,
    ) -> Vec<Vec<String>> {
        let script_names: HashSet<&str> = scripts.keys().map(|s| s.as_str()).collect();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        for (name, value) in scripts {
            let refs = find_script_references(value, &script_names);
            adj.entry(name.as_str()).or_default().extend(refs);
        }

        let mut cycles = Vec::new();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut path: Vec<&str> = Vec::new();
        let mut path_set: HashSet<&str> = HashSet::new();

        for node in scripts.keys().map(|s| s.as_str()) {
            if !visited.contains(node) {
                dfs_cycles(node, &adj, &mut visited, &mut path, &mut path_set, &mut cycles);
            }
        }

        cycles
    }

    pub fn sort_by_dependency(
        &self,
        packages: &[(String, HashMap<String, String>)],
    ) -> Result<Vec<String>, ScriptError> {
        let n = packages.len();
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
        let mut in_degree: Vec<usize> = vec![0; n];

        for (i, (_, scripts_i)) in packages.iter().enumerate() {
            let names_j: Vec<HashSet<&str>> = packages
                .iter()
                .map(|(_, s)| s.keys().map(|k| k.as_str()).collect())
                .collect();

            for (j, (_, _scripts_j)) in packages.iter().enumerate() {
                if i == j {
                    continue;
                }
                for value in scripts_i.values() {
                    let refs = find_script_references(value, &names_j[j]);
                    if !refs.is_empty() {
                        adj[j].push(i);
                        in_degree[i] += 1;
                        break;
                    }
                }
            }
        }

        let mut queue: VecDeque<usize> = in_degree
            .iter()
            .enumerate()
            .filter(|(_, deg)| **deg == 0)
            .map(|(i, _)| i)
            .collect();

        let mut sorted = Vec::new();
        while let Some(idx) = queue.pop_front() {
            sorted.push(packages[idx].0.clone());
            for &next in &adj[idx] {
                in_degree[next] -= 1;
                if in_degree[next] == 0 {
                    queue.push_back(next);
                }
            }
        }

        if sorted.len() != n {
            let unsorted: Vec<String> = packages
                .iter()
                .filter(|(name, _)| !sorted.contains(name))
                .map(|(name, _)| name.clone())
                .collect();
            return Err(ScriptError::CircularDependency(unsorted));
        }

        Ok(sorted)
    }

    pub fn npx(&self, package: &str, args: &[&str]) -> RunResult {
        let script = format!("npx {} {}", package, args.join(" "));
        self.run(&script)
    }

    pub fn run_in_parallel(
        &self,
        scripts: &[(String, &HashMap<String, String>)],
    ) -> Vec<(String, RunResult)> {
        let script_strings: Vec<(String, String)> = scripts
            .iter()
            .filter_map(|(name, map)| {
                map.get(name).map(|script| (name.clone(), script.clone()))
            })
            .collect();

        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        let cwd = Arc::new(self.cwd.clone());
        let env = Arc::new(self.env.clone());
        let shell = Arc::new(self.shell.clone());
        let shell_args = Arc::new(self.shell_args.clone());

        for (name, script) in script_strings {
            let results = Arc::clone(&results);
            let cwd = Arc::clone(&cwd);
            let env = Arc::clone(&env);
            let shell = Arc::clone(&shell);
            let shell_args = Arc::clone(&shell_args);

            handles.push(std::thread::spawn(move || {
                let start = Instant::now();
                let output = Command::new(&*shell)
                    .args(&*shell_args)
                    .arg(&script)
                    .current_dir(&*cwd)
                    .envs(&*env)
                    .output();

                let (exit_code, stdout, stderr, success, duration) = match output {
                    Ok(out) => (
                        out.status.code().unwrap_or(-1),
                        String::from_utf8_lossy(&out.stdout).to_string(),
                        String::from_utf8_lossy(&out.stderr).to_string(),
                        out.status.success(),
                        start.elapsed(),
                    ),
                    Err(e) => (-1, String::new(), format!("{e}"), false, start.elapsed()),
                };

                let result = RunResult {
                    exit_code,
                    stdout,
                    stderr,
                    duration,
                    success,
                };

                results.lock().unwrap().push((name, result));
            }));
        }

        for handle in handles {
            let _ = handle.join();
        }

        Arc::try_unwrap(results).unwrap().into_inner().unwrap()
    }
}

impl RunResult {
    pub fn format_output(&self, verbose: bool) -> String {
        let status = if self.success { "\u{2713}" } else { "\u{2717}" };
        let duration_ms = self.duration.as_millis();

        if verbose {
            format!(
                "{status} (exit: {}, {}ms)\n--- stdout ---\n{}--- stderr ---\n{}",
                self.exit_code, duration_ms, self.stdout, self.stderr
            )
        } else {
            format!("{status} (exit: {}, {}ms)", self.exit_code, duration_ms)
        }
    }

    pub fn check_success(&self) -> Result<(), ScriptError> {
        if self.success {
            Ok(())
        } else {
            Err(ScriptError::ExecutionError {
                script: "<unknown>".to_string(),
                exit_code: self.exit_code,
                stderr: self.stderr.clone(),
            })
        }
    }
}

fn find_script_references<'a>(
    value: &'a str,
    script_names: &HashSet<&'a str>,
) -> Vec<&'a str> {
    let mut refs = Vec::new();
    let words: Vec<&str> = value.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        if matches!(*word, "run" | "yarn" | "pnpm" | "bun") {
            if i + 1 < words.len() {
                let next = words[i + 1];
                if next != "run" && script_names.contains(next) && !refs.contains(&next) {
                    refs.push(next);
                }
            }
        }
        if let Some(bin_name) = word.strip_prefix("node_modules/.bin/") {
            if script_names.contains(bin_name) && !refs.contains(&bin_name) {
                refs.push(bin_name);
            }
        }
    }

    for (i, word) in words.iter().enumerate() {
        if i == 0 && script_names.contains(word) && !refs.contains(word) {
            let common = [
                "node", "npm", "npx", "yarn", "pnpm", "bun", "echo", "cat", "ls", "cd",
                "mkdir", "rm", "cp", "mv", "touch", "grep", "find", "sed", "awk",
            ];
            if !common.contains(word) {
                refs.push(word);
            }
        }
    }

    refs
}

fn dfs_cycles<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
    path_set: &mut HashSet<&'a str>,
    cycles: &mut Vec<Vec<String>>,
) {
    if path_set.contains(node) {
        if let Some(pos) = path.iter().position(|n| *n == node) {
            let cycle: Vec<String> = path[pos..].iter().map(|s| (*s).to_string()).collect();
            cycles.push(cycle);
        }
        return;
    }

    if visited.contains(node) {
        return;
    }

    visited.insert(node);
    path.push(node);
    path_set.insert(node);

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            dfs_cycles(neighbor, adj, visited, path, path_set, cycles);
        }
    }

    path.pop();
    path_set.remove(node);
}

pub const LIFECYCLE_EVENTS: &[&str] = &[
    "preinstall", "install", "postinstall",
    "prebuild", "build", "postbuild",
    "predev", "dev", "postdev",
    "pretest", "test", "posttest",
    "prepublish", "publish", "postpublish",
    "prestart", "start", "poststart",
];

pub fn get_lifecycle_scripts(scripts: &HashMap<String, String>) -> LifecycleHooks {
    LifecycleHooks {
        preinstall: scripts.get("preinstall").cloned(),
        install: scripts.get("install").cloned(),
        postinstall: scripts.get("postinstall").cloned(),
        prebuild: scripts.get("prebuild").cloned(),
        build: scripts.get("build").cloned(),
        postbuild: scripts.get("postbuild").cloned(),
        predev: scripts.get("predev").cloned(),
        dev: scripts.get("dev").cloned(),
        postdev: scripts.get("postdev").cloned(),
        pretest: scripts.get("pretest").cloned(),
        test: scripts.get("test").cloned(),
        posttest: scripts.get("posttest").cloned(),
        prepublish: scripts.get("prepublish").cloned(),
        publish: scripts.get("publish").cloned(),
        postpublish: scripts.get("postpublish").cloned(),
        prestart: scripts.get("prestart").cloned(),
        start: scripts.get("start").cloned(),
        poststart: scripts.get("poststart").cloned(),
    }
}

pub fn get_lifecycle_order(name: &str) -> [String; 3] {
    [format!("pre{name}"), name.to_string(), format!("post{name}")]
}
