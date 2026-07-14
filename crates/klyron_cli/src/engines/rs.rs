use super::{EngineInput, EngineOutput, EngineProcess, find_engine_path};

pub struct RsEngine {
    process: EngineProcess,
}

impl RsEngine {
    pub fn new() -> anyhow::Result<Self> {
        let path = find_engine_path("klyron-engine-rust");
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

    pub fn check(&mut self, project: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "check".into(), code: None,
            args: None, filename: None, project: Some(project.into()), files: None,
        })
    }

    pub fn build(&mut self, args: Option<&str>, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "build".into(), code: None,
            args: args.map(|s| s.into()), filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn test(&mut self, args: Option<&str>, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "test".into(), code: None,
            args: args.map(|s| s.into()), filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn fmt(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "fmt".into(), code: Some(code.into()),
            args: None, filename: None, project: None, files: None,
        })
    }

    pub fn clippy(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "clippy".into(), code: Some(code.into()),
            args: None, filename: None, project: None, files: None,
        })
    }

    pub fn scaffold(&mut self, kind: &str) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "scaffold".into(), code: None,
            args: Some(kind.into()), filename: None, project: None, files: None,
        })
    }

    pub fn ping(&mut self) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "ping".into(), code: None,
            args: None, filename: None, project: None, files: None,
        })
    }
}
