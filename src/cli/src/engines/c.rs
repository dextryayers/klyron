use super::{EngineInput, EngineOutput, EngineProcess, find_engine_path};

pub struct CEngine {
    process: EngineProcess,
}

#[allow(dead_code)]
impl CEngine {
    pub fn new() -> anyhow::Result<Self> {
        let path = find_engine_path("klyron-engine-c");
        let process = EngineProcess::spawn(&path, &[])?;
        Ok(Self { process })
    }

    pub fn exec(&mut self, code: &str, args: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "exec".into(), code: Some(code.into()),
            args: args.map(|s| s.into()), filename: None, project: None, files: None,
        })
    }

    pub fn eval_expr(&mut self, expr: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "eval".into(), code: Some(expr.into()),
            args: None, filename: None, project: None, files: None,
        })
    }

    pub fn compile(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "compile".into(), code: Some(code.into()),
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
