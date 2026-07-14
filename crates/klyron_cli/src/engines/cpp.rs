use super::{EngineInput, EngineOutput, EngineProcess, find_engine_path};

pub struct CppEngine {
    process: EngineProcess,
}

impl CppEngine {
    pub fn new() -> anyhow::Result<Self> {
        let path = find_engine_path("klyron-engine-cpp");
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

    pub fn run_file(&mut self, path: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "file".into(), code: None,
            args: None, filename: Some(path.into()), project: None, files: None,
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
