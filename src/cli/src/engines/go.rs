use super::{EngineInput, EngineOutput, EngineProcess};
use std::path::Path;

pub struct GoEngine {
    process: EngineProcess,
}

impl GoEngine {
    pub fn new() -> anyhow::Result<Self> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let engine_go = manifest_dir.join("engines/go/engine.go");
        if !engine_go.exists() {
            anyhow::bail!("Go engine not found at: {}", engine_go.display());
        }
        let process = EngineProcess::spawn("go", &["run", engine_go.to_str().unwrap()])?;
        Ok(Self { process })
    }

    pub fn exec(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "exec".into(), code: Some(code.into()),
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
