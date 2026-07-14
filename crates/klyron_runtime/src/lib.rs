//! klyron_runtime — Re-exports and utilities from klyron-core

pub use klyron_core::*;

use anyhow::Result;

pub fn create_runtime(extensions: Vec<deno_core::Extension>, enable_typescript: bool, async_: bool) -> Result<Runtime> {
    Runtime::builder()
        .enable_typescript(enable_typescript)
        .async_(async_)
        .extensions(extensions)
        .build()
}

pub fn execute_script(runtime: &mut Runtime, name: &str, source: &str) -> Result<String> {
    runtime.execute_script(name, source)
}
