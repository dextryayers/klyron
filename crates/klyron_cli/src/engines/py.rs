use super::{EngineInput, EngineOutput, EngineProcess};
use std::path::Path;

pub struct PyEngine {
    process: EngineProcess,
}

#[allow(dead_code)]
impl PyEngine {
    pub fn new() -> anyhow::Result<Self> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let engine_py = manifest_dir.join("engines/py/engine.py");
        if !engine_py.exists() {
            anyhow::bail!("Python engine not found at: {}", engine_py.display());
        }
        let process = EngineProcess::spawn("python3", &[engine_py.to_str().unwrap()])
            .or_else(|_| EngineProcess::spawn("python", &[engine_py.to_str().unwrap()]))?;
        Ok(Self { process })
    }

    pub fn exec(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "exec".into(), code: Some(code.into()),
            args: None, filename: None, project: None, files: None,
        })
    }

    pub fn eval_expr(&mut self, expr: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "eval".into(), code: Some(expr.into()),
            args: None, filename: None, project: None, files: None,
        })
    }

    pub fn run_file(&mut self, path: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "file".into(), code: Some(path.into()),
            args: None, filename: None, project: None, files: None,
        })
    }

    pub fn check(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "check".into(), code: Some(code.into()),
            args: None, filename: None, project: None, files: None,
        })
    }

    pub fn ping(&mut self) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "ping".into(), code: None,
            args: None, filename: None, project: None, files: None,
        })
    }
}
