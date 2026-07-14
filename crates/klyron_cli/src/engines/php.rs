use super::{EngineInput, EngineOutput, EngineProcess};
use std::path::Path;

pub struct PhpEngine {
    process: EngineProcess,
}

impl PhpEngine {
    pub fn new() -> anyhow::Result<Self> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let engine_php = manifest_dir.join("engines/php/engine.php");
        if !engine_php.exists() {
            anyhow::bail!("PHP engine not found at: {}", engine_php.display());
        }
        let process = EngineProcess::spawn("php", &[engine_php.to_str().unwrap()])?;
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

    pub fn artisan(&mut self, args: &str, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "artisan".into(), code: None,
            args: Some(args.into()), filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn composer(&mut self, args: &str, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "composer".into(), code: None,
            args: Some(args.into()), filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn blade(&mut self, view: &str, data_json: Option<&str>, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "blade".into(), code: Some(view.into()),
            args: data_json.map(|s| s.into()), filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn artisan_serve(&mut self, args: &str, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "artisan:serve".into(), code: None,
            args: Some(args.into()), filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn artisan_make(&mut self, args: &str, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "artisan:make".into(), code: None,
            args: Some(args.into()), filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn artisan_migrate(&mut self, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "artisan:migrate".into(), code: None,
            args: None, filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn tinker(&mut self, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "artisan:tinker".into(), code: None,
            args: None, filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn blade_clear(&mut self, project: Option<&str>) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "blade:clear".into(), code: None,
            args: None, filename: None, project: project.map(|s| s.into()), files: None,
        })
    }

    pub fn ping(&mut self) -> anyhow::Result<EngineOutput> {
        self.process.communicate(&EngineInput {
            action: "ping".into(), code: None,
            args: None, filename: None, project: None, files: None,
        })
    }
}
