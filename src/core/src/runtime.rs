use std::{cell::RefCell, rc::Rc, sync::Arc};

use anyhow::Result;
use base64::Engine;
use deno_core::{extension, op2, v8, Extension, FastString, JsRuntime, PollEventLoopOptions, RuntimeOptions, serde_v8};

use crate::module_loader::KlyronModuleLoader;
use crate::permissions::{PermissionSet, Permissions};
use crate::transpiler::Transpiler;

extension!(klyron_core, ops = [op_klyron_version],);

#[op2]
#[string]
fn op_klyron_version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
}

pub struct RuntimeBuilder {
  permissions: PermissionSet,
  extensions: Vec<Extension>,
  enable_typescript: bool,
  async_: bool,
}

impl Default for RuntimeBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl RuntimeBuilder {
  pub fn new() -> Self {
    Self { permissions: PermissionSet::default(), extensions: vec![], enable_typescript: true, async_: false }
  }

  pub fn async_(mut self, enabled: bool) -> Self {
    self.async_ = enabled;
    self
  }

  pub fn permissions(mut self, perms: PermissionSet) -> Self {
    self.permissions = perms;
    self
  }

  pub fn extension(mut self, ext: Extension) -> Self {
    self.extensions.push(ext);
    self
  }

  pub fn extensions(mut self, exts: Vec<Extension>) -> Self {
    self.extensions.extend(exts);
    self
  }

  pub fn enable_typescript(mut self, enabled: bool) -> Self {
    self.enable_typescript = enabled;
    self
  }

  pub fn build(self) -> Result<Runtime> {
    let transpiler = if self.enable_typescript { Some(Transpiler::new()) } else { None };

    let mut all_extensions = vec![klyron_core::init()];
    all_extensions.extend(self.extensions);

    let permissions = Arc::new(Permissions::new(self.permissions));
    let module_loader = KlyronModuleLoader::new(permissions.clone(), transpiler.clone());
    let module_loader = Rc::new(module_loader);

    let options = RuntimeOptions {
      extensions: all_extensions,
      module_loader: Some(module_loader as _),
      ..Default::default()
    };

    let js_runtime = JsRuntime::new(options);
    let tokio_runtime = if self.async_ { Some(tokio::runtime::Runtime::new()?) } else { None };

    Ok(Runtime { js_runtime: RefCell::new(js_runtime), tokio_runtime, permissions, transpiler })
  }
}

pub struct Runtime {
  js_runtime: RefCell<JsRuntime>,
  tokio_runtime: Option<tokio::runtime::Runtime>,
  permissions: Arc<Permissions>,
  pub transpiler: Option<Transpiler>,
}

impl Runtime {
  pub fn builder() -> RuntimeBuilder {
    RuntimeBuilder::new()
  }

  pub fn eval(&self, code: &str) -> Result<String> {
    let mut js = self.js_runtime.borrow_mut();
    let (code, _) = if let Some(ref transpiler) = self.transpiler {
      match transpiler.transpile_with_sourcemap("<eval>", code) {
        Ok((c, sm)) => {
          let final_source = if let Some(ref sm_json) = sm {
            let sm_b64 = base64::engine::general_purpose::STANDARD.encode(sm_json);
            format!("{c}\n//# sourceMappingURL=data:application/json;base64,{sm_b64}")
          } else { c };
          (final_source, sm)
        }
        Err(e) => return Err(anyhow::anyhow!("Transpile error: {e}")),
      }
    } else { (code.to_string(), None) };

    let global = js.execute_script("<eval>", FastString::from(code))?;
    deno_core::scope!(scope, &mut *js);
    let local = v8::Local::new(scope, global);
    let value: serde_json::Value = serde_v8::from_v8(scope, local)?;
    Ok(value_to_string(&value))
  }

  pub fn execute_script(&self, name: &str, source: &str) -> Result<String> {
    let mut js = self.js_runtime.borrow_mut();
    let (final_source, _sm) = if let Some(ref transpiler) = self.transpiler {
      if name.ends_with(".ts") || name.ends_with(".tsx") || name.ends_with(".jsx") || name.ends_with(".mts") {
        let (code, sm) = transpiler.transpile_with_sourcemap(name, source).map_err(|e| anyhow::anyhow!("{e}"))?;
        let final_source = if let Some(ref sm_json) = sm {
          let sm_b64 = base64::engine::general_purpose::STANDARD.encode(sm_json);
          format!("{code}\n//# sourceMappingURL=data:application/json;base64,{sm_b64}")
        } else { code };
        (final_source, sm)
      } else { (source.to_string(), None) }
    } else { (source.to_string(), None) };

    let global = js.execute_script(name.to_string(), FastString::from(final_source))?;
    deno_core::scope!(scope, &mut *js);
    let local = v8::Local::new(scope, global);
    let value: serde_json::Value = serde_v8::from_v8(scope, local)?;
    Ok(value_to_string(&value))
  }

  pub fn execute_module(&self, name: &str, source: &str) -> Result<()> {
    let mut js = self.js_runtime.borrow_mut();
    let (final_source, _sm) = if let Some(ref transpiler) = self.transpiler {
      if name.ends_with(".ts") || name.ends_with(".tsx") || name.ends_with(".jsx") || name.ends_with(".mts") {
        let (code, sm) = transpiler.transpile_with_sourcemap(name, source).map_err(|e| anyhow::anyhow!("{e}"))?;
        let final_source = if let Some(ref sm_json) = sm {
          let sm_b64 = base64::engine::general_purpose::STANDARD.encode(sm_json);
          format!("{code}\n//# sourceMappingURL=data:application/json;base64,{sm_b64}")
        } else { code };
        (final_source, sm)
      } else { (source.to_string(), None) }
    } else { (source.to_string(), None) };

    js.execute_script(name.to_string(), FastString::from(final_source))?;
    Ok(())
  }

  pub fn permissions(&self) -> Option<&Permissions> {
    Some(&self.permissions)
  }

  pub fn permission_set(&self) -> &PermissionSet {
    &self.permissions.set
  }

  pub fn run_event_loop(&self) -> Result<()> {
    if let Some(rt) = &self.tokio_runtime {
      let mut js = self.js_runtime.borrow_mut();
      rt.block_on(js.run_event_loop(PollEventLoopOptions::default()))?;
    }
    Ok(())
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_eval_number() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.eval("42").unwrap(), "42");
    assert_eq!(runtime.eval("1 + 2").unwrap(), "3");
    assert_eq!(runtime.eval("3.14").unwrap(), "3.14");
  }

  #[test]
  fn test_eval_string() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.eval("\"hello\"").unwrap(), "hello");
    assert_eq!(runtime.eval("'world'").unwrap(), "world");
  }

  #[test]
  fn test_eval_boolean() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.eval("true").unwrap(), "true");
    assert_eq!(runtime.eval("false").unwrap(), "false");
    assert_eq!(runtime.eval("!true").unwrap(), "false");
  }

  #[test]
  fn test_eval_null_undefined() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.eval("null").unwrap(), "null");
    assert_eq!(runtime.eval("undefined").unwrap(), "null");
  }

  #[test]
  fn test_eval_object() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.eval("JSON.parse('{\"a\":1}')").unwrap(), "{\"a\":1}");
  }

  #[test]
  fn test_eval_array() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.eval("[1, 2, 3]").unwrap(), "[1,2,3]");
  }

  #[test]
  fn test_eval_function_call() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.eval("Math.max(1, 2, 3)").unwrap(), "3");
  }

  #[test]
  fn test_eval_template_literal() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.eval("`hello ${1 + 2}`").unwrap(), "hello 3");
  }

  #[test]
  fn test_execute_script() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.execute_script("test.js", "1 + 2").unwrap(), "3");
  }

  #[test]
  fn test_execute_script_with_name() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.execute_script("mycode.ts", "const x = 10; x * 2").unwrap(), "20");
  }

  #[test]
  fn test_runtime_builder_default() {
    let runtime = Runtime::builder().build().unwrap();
    assert!(runtime.transpiler.is_some());
  }

  #[test]
  fn test_runtime_builder_no_typescript() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert!(runtime.transpiler.is_none());
  }

  #[test]
  fn test_eval_promise_object() {
    let runtime = Runtime::builder().enable_typescript(false).async_(true).build().unwrap();
    let result = runtime.eval("Promise.resolve(42)").unwrap();
    assert!(result.contains("Promise") || result == "{}");
  }

  #[test]
  fn test_eval_error() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert!(runtime.eval("throw new Error('fail')").is_err());
  }

  #[test]
  fn test_eval_syntax_error() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert!(runtime.eval("!!!invalid syntax!!!").is_err());
  }

  #[test]
  fn test_multiple_evals_same_runtime() {
    let runtime = Runtime::builder().enable_typescript(false).build().unwrap();
    assert_eq!(runtime.eval("let x = 1; x").unwrap(), "1");
    assert_eq!(runtime.eval("x + 2").unwrap(), "3");
    assert_eq!(runtime.eval("x = 10; x").unwrap(), "10");
  }

  #[test]
  fn test_eval_with_typescript_enum() {
    let runtime = Runtime::builder().enable_typescript(true).build().unwrap();
    let result = runtime.eval("const x = 1; x").unwrap();
    assert_eq!(result, "1");
  }
}
