use super::{EngineInput, EngineOutput, EngineProcess};
use std::path::Path;

pub struct RbEngine {
    process: EngineProcess,
}

#[allow(dead_code)]
impl RbEngine {
    pub fn new() -> anyhow::Result<Self> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let engine_rb = manifest_dir.join("engines/rb/engine.rb");
        if !engine_rb.exists() {
            anyhow::bail!("Ruby engine not found at: {}", engine_rb.display());
        }
        let process = EngineProcess::spawn("ruby", &[engine_rb.to_str().unwrap()])?;
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
