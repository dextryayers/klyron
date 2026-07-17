use std::collections::HashMap;

use deno_core::{extension, op2, Extension};
use deno_error::JsErrorBox;

extension!(
  klyron_node,
  ops = [
    op_process_info, op_process_args, op_process_env, op_process_exit,
    op_process_cwd, op_process_hrtime, op_process_spawn, op_process_exec,
  ],
  esm_entry_point = "ext:klyron_node/index.js",
  esm = [dir "js", "index.js", "assert.js", "buffer.js", "child_process.js", "crypto.js", "events.js", "fs.js", "net.js", "os.js", "path.js", "process.js", "querystring.js", "stream.js", "url.js", "util.js", "http.js", "https.js"],
);

pub fn init() -> Extension {
  klyron_node::init()
}

#[op2]
#[string]
fn op_process_info() -> String {
  serde_json::json!({
    "pid": std::process::id(),
    "ppid": 0,
    "platform": std::env::consts::OS,
    "arch": std::env::consts::ARCH,
    "v8_version": "11.0",
  })
  .to_string()
}

#[op2]
#[string]
fn op_process_args() -> String {
  let args: Vec<String> = std::env::args().collect();
  serde_json::to_string(&args).unwrap_or_else(|_| "[]".to_string())
}

#[op2]
#[string]
fn op_process_env() -> String {
  let env: HashMap<String, String> = std::env::vars().collect();
  serde_json::to_string(&env).unwrap_or_else(|_| "{}".to_string())
}

#[op2(fast)]
fn op_process_exit(code: i32) {
  std::process::exit(code);
}

#[op2]
#[string]
fn op_process_cwd() -> String {
  std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
}

#[op2]
#[string]
fn op_process_hrtime() -> String {
  let dur = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default();
  serde_json::json!([dur.as_secs(), dur.subsec_nanos()]).to_string()
}

#[op2]
#[string]
fn op_process_spawn(#[string] cmd: String, #[string] args_json: String) -> Result<String, JsErrorBox> {
  let args: Vec<String> = serde_json::from_str(&args_json).map_err(|e| JsErrorBox::generic(format!("spawn args: {e}")))?;
  match std::process::Command::new(&cmd).args(&args).output() {
    Ok(output) => Ok(serde_json::json!({
      "pid": 0,
      "stdout": String::from_utf8_lossy(&output.stdout),
      "stderr": String::from_utf8_lossy(&output.stderr),
      "code": output.status.code(),
    }).to_string()),
    Err(e) => Err(JsErrorBox::generic(format!("spawn {cmd}: {e}"))),
  }
}

#[op2]
#[string]
fn op_process_exec(#[string] cmd: String) -> Result<String, JsErrorBox> {
  let shell = if cfg!(windows) { "cmd" } else { "sh" };
  let flag = if cfg!(windows) { "/c" } else { "-c" };
  match std::process::Command::new(shell).arg(flag).arg(&cmd).output() {
    Ok(output) => Ok(serde_json::json!({
      "stdout": String::from_utf8_lossy(&output.stdout),
      "stderr": String::from_utf8_lossy(&output.stderr),
      "code": output.status.code(),
    }).to_string()),
    Err(e) => Err(JsErrorBox::generic(format!("exec {cmd}: {e}"))),
  }

#[cfg(test)]
mod integration_tests {
  use deno_core::{v8, FastString, JsRuntime, ModuleLoadOptions, ModuleLoadReferrer,
                  ModuleLoadResponse, ModuleLoader, ModuleSpecifier, RuntimeOptions};
  use std::sync::Arc;

  // Minimal module loader: extension `ext:` ES modules are provided by the
  // extensions themselves, so we never need to fetch source here.
  struct TestLoader;
  impl ModuleLoader for TestLoader {
    fn resolve(
      &self,
      specifier: &str,
      _referrer: &str,
      _kind: deno_core::ResolutionKind,
    ) -> deno_core::ModuleResolveResponse {
      Ok(ModuleSpecifier::parse(specifier).unwrap())
    }
    fn load(
      &self,
      _specifier: &ModuleSpecifier,
      _maybe_referrer: Option<&ModuleLoadReferrer>,
      _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
      ModuleLoadResponse::Sync(Err(deno_error::JsErrorBox::generic("unexpected load")))
    }
  }

  async fn run_js(source: &str) -> String {
    let mut runtime = JsRuntime::new(RuntimeOptions {
      extensions: vec![klyron_ext_net::init(), crate::init()],
      module_loader: Some(Arc::new(TestLoader)),
      ..Default::default()
    });
    let spec = ModuleSpecifier::parse("ext:klyron_test/main.mjs").unwrap();
    let id = runtime
      .load_main_es_module_from_code(&spec, source)
      .await
      .unwrap();
    runtime.mod_evaluate(id).await.unwrap();
    runtime
      .run_event_loop(deno_core::PollEventLoopOptions::default())
      .await
      .unwrap();
    // Read the value the module stored on globalThis.__RESULT__.
    let global = runtime
      .execute_script("read", deno_core::FastString::from("globalThis.__RESULT__".to_string()))
      .unwrap();
    deno_core::scope!(scope, &mut runtime);
    let local = v8::Local::new(scope, global);
    match deno_core::serde_v8::from_v8::<Option<String>>(scope, local) {
      Ok(Some(s)) => s,
      _ => String::new(),
    }
  }

  #[tokio::test]
  async fn test_http_server_roundtrip() {
    let source = r#"
      import http from 'ext:klyron_node/http.js';
      import net from 'ext:klyron_node/net.js';

      const server = http.createServer((req, res) => {
        res.setHeader('Content-Type', 'text/plain');
        res.end('HELLO FROM KLYRON');
      });
      await new Promise((r) => server.listen(18099, '127.0.0.1', r));

      const body = await new Promise((resolve) => {
        const sock = net.connect(18099, '127.0.0.1');
        let data = Buffer.alloc(0);
        sock.on('connect', () => sock.write('GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n'));
        sock.on('data', (d) => { data = Buffer.concat([data, d]); });
        sock.on('end', () => resolve(data.toString()));
      });

      server.close();
      globalThis.__RESULT__ = body;
    "#;
    let out = run_js(source).await;
    assert!(out.contains("HELLO FROM KLYRON"), "expected response body, got: {out:?}");
  }

  #[tokio::test]
  async fn test_http_exports_present() {
    let out = run_js(r#"
      import http from 'ext:klyron_node/http.js';
      import https from 'ext:klyron_node/https.js';
      globalThis.__RESULT__ = (typeof http.createServer === 'function'
        && typeof http.Server === 'function'
        && typeof https.createServer === 'function'
        && typeof http.STATUS_CODES === 'object') ? 'OK' : 'FAIL';
    "#).await;
    assert_eq!(out, "OK");
  }
}

}
