use crate::{EngineRuntime, JsEngineKind};

#[derive(Debug, Clone)]
pub struct CodeChunk {
    pub data: String,
    pub is_final: bool,
}

#[derive(Debug, Clone)]
pub struct ChunkResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub is_final: bool,
}

pub struct StreamProcessor {
    engine: Option<EngineRuntime>,
    buffer: String,
    kind: JsEngineKind,
}

impl StreamProcessor {
    pub fn new(kind: JsEngineKind) -> Self {
        Self { engine: None, buffer: String::new(), kind }
    }

    pub fn feed(&mut self, chunk: CodeChunk) -> Result<ChunkResult, String> {
        self.buffer.push_str(&chunk.data);

        if !chunk.is_final {
            return Ok(ChunkResult {
                success: true,
                output: None,
                error: None,
                is_final: false,
            });
        }

        if self.engine.is_none() {
            self.engine = Some(EngineRuntime::new(self.kind).map_err(|e| format!("Engine init: {}", e))?);
        }

        let engine = self.engine.as_ref().unwrap();
        match engine.eval(&self.buffer) {
            Ok(output) => {
                self.buffer.clear();
                Ok(ChunkResult {
                    success: true,
                    output: Some(output),
                    error: None,
                    is_final: true,
                })
            }
            Err(e) => Ok(ChunkResult {
                success: false,
                output: None,
                error: Some(e.to_string()),
                is_final: true,
            }),
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.engine = None;
    }
}
