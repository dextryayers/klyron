use std::time::Instant;

use rayon::prelude::*;

use crate::{EngineRuntime, JsEngineKind};

#[derive(Debug, Clone)]
pub struct ScriptJob {
    pub id: String,
    pub code: String,
    pub filename: Option<String>,
    pub engine_kind: Option<JsEngineKind>,
}

#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: f64,
}

pub fn execute_parallel(scripts: &[ScriptJob]) -> Vec<ScriptResult> {
    scripts.par_iter().map(|job| {
        let start = Instant::now();
        let kind = job.engine_kind.unwrap_or_else(crate::detect_best_engine);

        match EngineRuntime::new(kind) {
            Ok(engine) => {
                let result = if let Some(ref filename) = job.filename {
                    engine.execute_script(filename, &job.code)
                } else {
                    engine.eval(&job.code)
                };

                let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

                match result {
                    Ok(output) => ScriptResult {
                        id: job.id.clone(),
                        success: true,
                        output: Some(output),
                        error: None,
                        duration_ms,
                    },
                    Err(e) => ScriptResult {
                        id: job.id.clone(),
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        duration_ms,
                    },
                }
            }
            Err(e) => ScriptResult {
                id: job.id.clone(),
                success: false,
                output: None,
                error: Some(format!("Failed to create engine: {}", e)),
                duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            },
        }
    }).collect()
}
