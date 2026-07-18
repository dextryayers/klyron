use crate::error::BoaError;
use crate::runtime::BoaRuntime;

pub fn create_snapshot(_runtime: &BoaRuntime) -> Result<Vec<u8>, BoaError> {
    let code = "typeof globalThis !== 'undefined'";
    Ok(code.as_bytes().to_vec())
}

pub fn load_snapshot(data: &[u8]) -> Result<BoaRuntime, BoaError> {
    let code = std::str::from_utf8(data)
        .map_err(|_| BoaError::CompileError("invalid snapshot data".into()))?;
    let mut runtime = BoaRuntime::new();
    runtime.eval("// snapshot restored")?;
    runtime.eval(code)?;
    Ok(runtime)
}

pub fn restore_snapshot(data: &[u8]) -> Result<BoaRuntime, BoaError> {
    load_snapshot(data)
}
