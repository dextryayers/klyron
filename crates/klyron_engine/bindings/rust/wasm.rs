//! WASM bindings for klyron_engine

pub fn wasm_init() -> anyhow::Result<()> {
    Ok(())
}

pub fn wasm_process(input: &str) -> String {
    format!("{input} processed by klyron_engine")
}
