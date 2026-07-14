pub mod process;
pub mod traits;
pub mod engine;

pub use process::{EngineProcess, EngineInput, EngineOutput, FileEntry, find_engine_path};
pub use traits::EngineTrait;
pub use engine::{JsEngineKind, EngineRuntime, JsEngine, JsValue, JsError, BenchResult, benchmark_all_engines, detect_best_engine};
