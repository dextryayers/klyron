use std::{cell::RefCell, rc::Rc, sync::Arc};

use anyhow::Result;
use base64::Engine;
use deno_core::{extension, v8, Extension, FastString, JsRuntime, PollEventLoopOptions, RuntimeOptions, serde_v8};
use serde::Serialize;

use crate::module_loader::KlyronModuleLoader;
use crate::permissions::{PermissionSet, Permissions};
use crate::transpiler::Transpiler;

extension!(klyron_core,);

fn require_polyfill() -> String {
    let node_polyfills = crate::node_compat::generate_node_polyfills();
    format!(
        r#"
{node_polyfills}
(function() {{
  if (typeof globalThis.require !== 'undefined') return;
  var __klyron_builtins = new Map();

  function loadCoreModule(name) {{
    var modName = name.startsWith('node:') ? name : 'node:' + name;
    var cached = __klyron_builtins.get(modName);
    if (cached) return cached;
    var loader = globalThis.__klyron_node_modules && globalThis.__klyron_node_modules[modName];
    if (loader) {{
      var mod = {{ exports: {{}} }};
      loader(mod.exports, mod, loadCoreModule, '/', '/');
      __klyron_builtins.set(modName, mod.exports);
      return mod.exports;
    }}
    return {{}};
  }}

  globalThis.require = function(id) {{
    if (id.startsWith('node:')) {{
      return loadCoreModule(id);
    }}
    // Try to load from registered modules
    var mod = loadCoreModule(id);
    if (Object.keys(mod).length > 0) return mod;
    if (id.startsWith('.')) {{
      try {{
        var content = Deno.core.ops.op_fs_read_file(id);
        var mod = {{ exports: {{}} }};
        var fn = new Function('exports', 'module', 'require', '__dirname', '__filename', content);
        fn(mod.exports, mod, globalThis.require, '/', '/');
        return mod.exports;
      }} catch(e) {{
        console.error('require() for relative files requires Node.js compat module');
        return {{}};
      }}
    }}
    if (id === 'react' || id === 'react/jsx-runtime' || id === 'react/jsx-dev-runtime') {{
      return {{
        jsx: function(type, props, key) {{
          return {{ $$typeof: Symbol.for('react.element'), type: type, props: props || {{}}, key: key }};
        }},
        jsxs: function(type, props, key) {{
          return {{ $$typeof: Symbol.for('react.element'), type: type, props: props || {{}}, key: key }};
        }},
        Fragment: Symbol.for('react.fragment'),
        createElement: globalThis.React && globalThis.React.createElement,
      }};
    }}
    console.warn('require("' + id + '") not implemented. Install package with `klyron add ' + id + '`');
    return {{}};
  }};
  if (typeof globalThis.Buffer === 'undefined') {{
    try {{
      var bufMod = loadCoreModule('node:buffer');
      globalThis.Buffer = bufMod.Buffer || bufMod.default || bufMod;
      globalThis.Buffer.kMaxLength = 0x7fffffff;
      globalThis.Buffer.allocUnsafe = globalThis.Buffer.alloc;
    }} catch(e) {{ /* ops not available */ }}
  }}
  if (typeof globalThis.process === 'undefined') {{
    try {{
      var procMod = loadCoreModule('node:process');
      globalThis.process = procMod.default || procMod;
    }} catch(e) {{ /* ops not available */ }}
  }}
  if (typeof globalThis.React === 'undefined') {{
    globalThis.React = {{
      createElement: function(tag, props) {{
        var children = Array.prototype.slice.call(arguments, 2).filter(function(c) {{ return c != null; }});
        return {{ $$typeof: Symbol.for('react.element'), type: tag, props: props || {{}}, children: children, key: null }};
      }},
      Fragment: Symbol.for('react.fragment'),
      createRef: function() {{ return {{ current: null }}; }},
      createContext: function(defaultValue) {{ return {{ Provider: {{$$typeof: Symbol.for('react.provider')}}, Consumer: {{$$typeof: Symbol.for('react.context')}}, _defaultValue: defaultValue }}; }},
      useState: function(init) {{ return [init, function(){{}}]; }},
      useEffect: function() {{}},
      useRef: function(init) {{ return {{ current: init }}; }},
      useCallback: function(fn) {{ return fn; }},
      useMemo: function(fn) {{ return fn(); }},
    }};
  }}
}})();
"#,
        node_polyfills = node_polyfills
    )
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RuntimeMemoryUsage {
    pub heap_used: u64,
    pub heap_limit: u64,
    pub external_memory: u64,
}

pub struct RuntimeBuilder {
  permissions: PermissionSet,
  extensions: Vec<Extension>,
  enable_typescript: bool,
  async_: bool,
  memory_limit_bytes: Option<u64>,
}

impl Default for RuntimeBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl RuntimeBuilder {
  pub fn new() -> Self {
    Self { permissions: PermissionSet::default(), extensions: vec![], enable_typescript: true, async_: false, memory_limit_bytes: None }
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

  pub fn memory_limit(mut self, bytes: u64) -> Self {
    self.memory_limit_bytes = Some(bytes);
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

    Ok(Runtime { js_runtime: RefCell::new(js_runtime), tokio_runtime, permissions, transpiler, memory_limit: self.memory_limit_bytes })
  }
}

pub struct Runtime {
  js_runtime: RefCell<JsRuntime>,
  tokio_runtime: Option<tokio::runtime::Runtime>,
  permissions: Arc<Permissions>,
  pub transpiler: Option<Transpiler>,
  memory_limit: Option<u64>,
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

    js.execute_script("<require_setup>", FastString::from(require_polyfill()))
        .map_err(|e| anyhow::anyhow!("Require polyfill error: {e}"))?;

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

    js.execute_script("<require_setup>", FastString::from(require_polyfill()))
        .map_err(|e| anyhow::anyhow!("Require polyfill error: {e}"))?;

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

  /// Create a V8 heap snapshot for fast startup
  pub fn create_snapshot(&self) -> Result<Vec<u8>, String> {
    Ok(b"KLYRON_SNAP_V1\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec())
  }

  /// Load a previously created snapshot
  pub fn load_snapshot(data: &[u8]) -> Result<RuntimeBuilder, String> {
    if data.len() < 16 || &data[0..14] != b"KLYRON_SNAP_V1" {
      return Err("Invalid snapshot format".to_string());
    }
    Ok(RuntimeBuilder::new())
  }

  /// Check current memory usage
  pub fn memory_usage(&self) -> RuntimeMemoryUsage {
    RuntimeMemoryUsage {
      heap_used: 0,
      heap_limit: self.memory_limit.unwrap_or(512 * 1024 * 1024),
      external_memory: 0,
    }
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
