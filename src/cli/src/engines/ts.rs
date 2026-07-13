use super::{EngineInput, EngineOutput, EngineProcess};
use std::path::Path;

#[allow(dead_code)]
pub struct TsEngine {
    process: EngineProcess,
}

#[allow(dead_code)]
impl TsEngine {
    pub fn new() -> anyhow::Result<Self> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let engine_ts = manifest_dir.join("engines/ts/engine.ts");
        if !engine_ts.exists() {
            anyhow::bail!("TS engine not found at: {}", engine_ts.display());
        }
        let process = EngineProcess::spawn("node", &[engine_ts.to_str().unwrap()])?;
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
            action: "file".into(), code: None,
            args: None, filename: Some(path.into()), project: None, files: None,
        })
    }

    pub fn typecheck(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
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
