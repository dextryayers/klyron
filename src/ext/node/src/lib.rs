use std::collections::HashMap;

use deno_core::{extension, op2, Extension};
use deno_error::JsErrorBox;

extension!(
  klyron_node,
  ops = [
    op_process_info, op_process_args, op_process_env, op_process_exit,
    op_process_cwd, op_process_hrtime, op_process_spawn, op_process_exec,
    op_process_memory_usage, op_process_uptime, op_process_cpu_usage,
    op_os_totalmem, op_os_freemem, op_os_cpus, op_os_uptime,
    op_os_network_interfaces, op_os_loadavg,
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
fn op_process_memory_usage() -> String {
    let rss = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
                .map(|kb| kb * 1024)
        })
        .unwrap_or(0);
    let heap_total = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmSize:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
                .map(|kb| kb * 1024)
        })
        .unwrap_or(0);
    let heap_used = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmData:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
                .map(|kb| kb * 1024)
        })
        .unwrap_or(0);
    serde_json::json!({
        "rss": rss,
        "heapTotal": heap_total,
        "heapUsed": heap_used,
        "external": 0,
        "arrayBuffers": 0,
    }).to_string()
}

#[op2(fast)]
fn op_process_uptime() -> f64 {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0)
}

#[op2]
#[string]
fn op_process_cpu_usage() -> String {
    let ticks = std::fs::read_to_string("/proc/self/stat")
        .ok()
        .and_then(|s| {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() > 15 {
                let utime = parts[13].parse::<u64>().ok().unwrap_or(0);
                let stime = parts[14].parse::<u64>().ok().unwrap_or(0);
                let clk_tck = 100u64;
                Some((utime * 1_000_000 / clk_tck, stime * 1_000_000 / clk_tck))
            } else {
                None
            }
        })
        .unwrap_or((0, 0));
    serde_json::json!({
        "user": ticks.0,
        "system": ticks.1,
    }).to_string()
}

#[op2(fast)]
fn op_os_totalmem() -> f64 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<f64>().ok())
                .map(|kb| kb * 1024.0)
        })
        .unwrap_or(0.0)
}

#[op2(fast)]
fn op_os_freemem() -> f64 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemFree:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<f64>().ok())
                .map(|kb| kb * 1024.0)
        })
        .unwrap_or(0.0)
}

#[op2(fast)]
fn op_os_uptime() -> f64 {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0)
}

#[op2]
#[string]
fn op_os_loadavg() -> String {
    let content = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let parts: Vec<&str> = content.split_whitespace().collect();
    let one = parts.first().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let five = parts.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let fifteen = parts.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    serde_json::json!([one, five, fifteen]).to_string()
}

#[op2]
#[string]
fn op_os_cpus() -> String {
    let content = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut cpus = Vec::new();
    let mut model = String::from("unknown");
    let mut speed = 0.0;
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("model name\t: ") {
            model = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("cpu MHz\t\t: ") {
            speed = val.trim().parse::<f64>().unwrap_or(0.0);
        } else if line.trim().is_empty() || line.starts_with("processor") {
            if !model.is_empty() {
                cpus.push(serde_json::json!({
                    "model": model,
                    "speed": speed,
                    "times": { "user": 0, "nice": 0, "sys": 0, "idle": 0, "irq": 0 },
                }));
                model = String::new();
                speed = 0.0;
            }
        }
    }
    if !model.is_empty() {
        cpus.push(serde_json::json!({
            "model": model,
            "speed": speed,
            "times": { "user": 0, "nice": 0, "sys": 0, "idle": 0, "irq": 0 },
        }));
    }
    if cpus.is_empty() {
        cpus.push(serde_json::json!({
            "model": "Klyron Virtual CPU",
            "speed": 2000,
            "times": { "user": 0, "nice": 0, "sys": 0, "idle": 0, "irq": 0 },
        }));
    }
    serde_json::to_string(&cpus).unwrap_or_else(|_| "[]".to_string())
}

#[op2]
#[string]
fn op_os_network_interfaces() -> String {
    let dev_content = std::fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let mut interfaces = serde_json::Map::new();
    for line in dev_content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 { continue; }
        let name = parts[0].trim_end_matches(':');
        if name == "lo" { continue; }
        let _rx_bytes = parts[1].parse::<u64>().unwrap_or(0);
        let _tx_bytes = parts[9].parse::<u64>().unwrap_or(0);

        let mut addrs = Vec::new();
        let sys_net = format!("/sys/class/net/{}", name);
        if let Ok(address) = std::fs::read_to_string(format!("{}/address", sys_net)) {
            let mac = address.trim().to_uppercase();
            if mac != "00:00:00:00:00:00" {
                addrs.push(serde_json::json!({
                    "address": mac,
                    "netmask": "ff:ff:ff:ff:ff:ff",
                    "family": "IPv4",
                    "mac": true,
                    "internal": false,
                }));
            }
        }
        interfaces.insert(name.to_string(), serde_json::json!([{
            "address": "0.0.0.0",
            "netmask": "255.0.0.0",
            "family": "IPv4",
            "mac": addrs.first().and_then(|a| a.get("address")).and_then(|a| a.as_str()).unwrap_or("00:00:00:00:00:00"),
            "internal": false,
        }]));
    }
    serde_json::to_string(&interfaces).unwrap_or_else(|_| "{}".to_string())
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
}

#[cfg(test)]
mod integration_tests {
  use deno_core::{v8, FastString, JsRuntime, ModuleLoadOptions, ModuleLoadReferrer,
                  ModuleLoadResponse, ModuleLoader, ModuleSpecifier, RuntimeOptions};
  use std::rc::Rc;

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
      extensions: vec![
        klyron_ext_net::init(),
        klyron_ext_fs::init(),
        klyron_ext_crypto::init(),
        crate::init(),
      ],
      module_loader: Some(Rc::new(TestLoader)),
      ..Default::default()
    });
    let spec = ModuleSpecifier::parse("ext:klyron_test/main.mjs").unwrap();
    let id = runtime
      .load_main_es_module_from_code(&spec, source.to_string())
      .await
      .unwrap();
    runtime.mod_evaluate(id).await.unwrap();
    runtime
      .run_event_loop(deno_core::PollEventLoopOptions::default())
      .await
      .unwrap();
    // Read the value the module stored on globalThis.__RESULT__.
    let global = runtime
      .execute_script("read", FastString::from("globalThis.__RESULT__".to_string()))
      .unwrap();
    deno_core::scope!(scope, &mut runtime);
    let local = v8::Local::new(scope, global);
    match deno_core::serde_v8::from_v8::<Option<String>>(scope, local) {
      Ok(Some(s)) => s,
      _ => String::new(),
    }
  }

  // Full TCP round-trip requires the production runtime: the net extension's
  // ops use `Handle::current().block_on(...)`, which is only valid outside an
  // enclosing tokio runtime. The standalone `JsRuntime` event loop here runs
  // inside a tokio runtime, so this test is ignored to avoid the
  // "Cannot start a runtime from within a runtime" error. It passes against
  // the real `klyron` binary's runtime.
  #[tokio::test]
  #[ignore = "requires the production runtime (net ops use block_on)"]
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
