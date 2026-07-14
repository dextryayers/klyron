use crate::engines::{EngineInput, EngineOutput, EngineProcess};

#[allow(dead_code)]
pub struct JsEngine {
    process: EngineProcess,
}

impl JsEngine {
    pub fn new() -> anyhow::Result<Self> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let engine_js = std::path::Path::new(manifest_dir).join("engines/js/engine.js");
        if !engine_js.exists() {
            anyhow::bail!("JS engine not found at: {}", engine_js.display());
        }
        let process = EngineProcess::spawn("node", &[engine_js.to_str().unwrap()])?;
        Ok(Self { process })
    }

    pub fn exec(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
        let input = EngineInput {
            action: "exec".into(),
            code: Some(code.into()),
            args: None,
            filename: None,
            project: None,
            files: None,
        };
        self.process.communicate(&input)
    }

    pub fn exec_file(&mut self, path: &str) -> anyhow::Result<EngineOutput> {
        let input = EngineInput {
            action: "file".into(),
            code: None,
            args: Some(path.into()),
            filename: None,
            project: None,
            files: None,
        };
        self.process.communicate(&input)
    }

    pub fn lint(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
        let input = EngineInput {
            action: "lint".into(),
            code: Some(code.into()),
            args: None,
            filename: None,
            project: None,
            files: None,
        };
        self.process.communicate(&input)
    }

    pub fn format(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
        let input = EngineInput {
            action: "format".into(),
            code: Some(code.into()),
            args: None,
            filename: None,
            project: None,
            files: None,
        };
        self.process.communicate(&input)
    }

    pub fn check(&mut self, project: &str) -> anyhow::Result<EngineOutput> {
        let input = EngineInput {
            action: "check".into(),
            code: None,
            args: None,
            filename: None,
            project: Some(project.into()),
            files: None,
        };
        self.process.communicate(&input)
    }

    pub fn build(&mut self, project: &str) -> anyhow::Result<EngineOutput> {
        let input = EngineInput {
            action: "build".into(),
            code: None,
            args: None,
            filename: None,
            project: Some(project.into()),
            files: None,
        };
        self.process.communicate(&input)
    }

    pub fn test(&mut self, project: &str) -> anyhow::Result<EngineOutput> {
        let input = EngineInput {
            action: "test".into(),
            code: None,
            args: None,
            filename: None,
            project: Some(project.into()),
            files: None,
        };
        self.process.communicate(&input)
    }
}
