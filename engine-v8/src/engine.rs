use anyhow::Result;
use deno_core::{v8, Extension, FastString, JsRuntime, RuntimeOptions, serde_v8};

pub struct V8Engine {
  js_runtime: JsRuntime,
}

impl V8Engine {
  pub fn new(extensions: Vec<Extension>) -> Result<Self> {
    let options = RuntimeOptions { extensions, ..Default::default() };
    let js_runtime = JsRuntime::new(options);
    Ok(Self { js_runtime })
  }

  pub fn js_runtime(&mut self) -> &mut JsRuntime {
    &mut self.js_runtime
  }

  pub fn execute_script(&mut self, name: &str, source: &str) -> Result<String> {
    let global = self.js_runtime.execute_script(name.to_string(), FastString::from(source.to_string()))?;
    deno_core::scope!(scope, &mut self.js_runtime);
    let local = v8::Local::new(scope, global);
    let value: serde_json::Value = serde_v8::from_v8(scope, local)?;
    Ok(value_to_string(&value))
  }

  pub fn eval(&mut self, code: &str) -> Result<String> {
    self.execute_script("<eval>", code)
  }
}

fn value_to_string(value: &serde_json::Value) -> String {
  match value {
    serde_json::Value::String(s) => s.clone(),
    serde_json::Value::Null => "null".to_string(),
    serde_json::Value::Bool(b) => b.to_string(),
    serde_json::Value::Number(n) => n.to_string(),
    _ => value.to_string(),
  }
}
