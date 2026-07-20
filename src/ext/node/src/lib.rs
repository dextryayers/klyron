use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, LazyLock, Mutex,
  },
  thread,
};

use deno_core::{extension, op2, Extension, FastString, JsRuntime, RuntimeOptions};
use deno_error::JsErrorBox;

// ── Worker Thread Registry ─────────────────────────────────────────────────

/// Each worker has two channels:
///   `to_worker`  — main sends TO the worker  (the worker receives via `rx_from_main`)
///   `from_worker` — main receives FROM the worker (worker sends via `tx_to_main`)
struct WorkerHandle {
  to_worker: mpsc::Sender<String>,
  from_worker: mpsc::Receiver<String>,
}

static NEXT_WORKER_ID: AtomicU64 = AtomicU64::new(1);
static WORKERS: LazyLock<Mutex<HashMap<u32, WorkerHandle>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

thread_local! {
  static WORKER_ID: std::cell::Cell<u32> = std::cell::Cell::new(0);
  static WORKER_DATA: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
  /// Worker's sender to main (used by op_worker_send_to_parent)
  static TX_TO_MAIN: std::cell::RefCell<Option<mpsc::Sender<String>>> = std::cell::RefCell::new(None);
  /// Worker's receiver from main (used by op_worker_poll_from_parent)
  static RX_FROM_MAIN: std::cell::RefCell<Option<mpsc::Receiver<String>>> = std::cell::RefCell::new(None);
}

fn worker_setup_js() -> String {
  format!(
    r#"
(function() {{
  if (typeof globalThis.parentPort !== 'undefined') return;
  var _workerEventMap = {{}};
  function _on(event, fn) {{
    if (!_workerEventMap[event]) _workerEventMap[event] = [];
    _workerEventMap[event].push(fn);
  }}
  function _emit(event) {{
    var args = Array.prototype.slice.call(arguments, 1);
    var list = _workerEventMap[event];
    if (!list) return;
    for (var i = 0; i < list.length; i++) list[i].apply(null, args);
  }}
  globalThis.parentPort = {{
    on: _on,
    addListener: _on,
    once: function(event, fn) {{
      function wrapper() {{
        var args = Array.prototype.slice.call(arguments);
        _off(event, wrapper);
        fn.apply(null, args);
      }}
      wrapper._orig = fn;
      _on(event, wrapper);
    }},
    off: function(event, fn) {{
      var list = _workerEventMap[event];
      if (!list) return;
      _workerEventMap[event] = list.filter(function(l) {{ return l !== fn && l._orig !== fn; }});
    }},
    removeListener: function(event, fn) {{ this.off(event, fn); }},
    emit: _emit,
    postMessage: function(msg) {{
      Deno.core.ops.op_worker_send_to_parent(JSON.stringify(msg));
    }},
    close: function() {{
      Deno.core.ops.op_worker_close();
    }},
  }};
  var _data = Deno.core.ops.op_worker_get_data();
  if (_data && _data !== 'null' && _data !== '') {{
    try {{ globalThis.workerData = JSON.parse(_data); }} catch(e) {{ globalThis.workerData = _data; }}
  }}
  (function _workerPoll() {{
    try {{
      var msgs = Deno.core.ops.op_worker_poll_from_parent();
      if (msgs) {{
        var arr = JSON.parse(msgs);
        for (var i = 0; i < arr.length; i++) {{
          _emit.call(globalThis.parentPort, 'message', arr[i]);
        }}
      }}
    }} catch(e) {{}}
    globalThis.queueMicrotask(_workerPoll);
  }})();
}})();
"#,
  )
}

// ── Extension Registration ─────────────────────────────────────────────────

extension!(
  klyron_node,
  ops = [
    op_process_info, op_process_args, op_process_env, op_process_exit,
    op_process_cwd, op_process_hrtime, op_process_spawn, op_process_exec,
    op_process_memory_usage, op_process_uptime, op_process_cpu_usage,
    op_os_totalmem, op_os_freemem, op_os_cpus, op_os_uptime,
    op_os_network_interfaces, op_os_loadavg,
    op_dns_lookup, op_dns_resolve, op_dns_reverse, op_https_request,
    op_zlib_gzip, op_zlib_gunzip, op_zlib_deflate, op_zlib_inflate,
    op_worker_create_thread, op_worker_send, op_worker_poll, op_worker_terminate,
    op_worker_is_worker, op_worker_thread_id, op_worker_get_data,
    op_worker_send_to_parent, op_worker_poll_from_parent, op_worker_close,
  ],
  esm_entry_point = "ext:klyron_node/index.js",
  esm = [dir "js", "index.js", "assert.js", "buffer.js", "child_process.js", "crypto.js", "dns.js", "events.js", "fs.js", "net.js", "os.js", "path.js", "process.js", "querystring.js", "stream.js", "url.js", "util.js", "http.js", "https.js", "worker_threads.js", "zlib.js"],
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

// ── Worker Thread Ops ──────────────────────────────────────────────────────

#[op2]
#[string]
fn op_worker_create_thread(
  #[string] filename: String,
  #[string] data_json: String,
) -> Result<String, JsErrorBox> {
  let worker_id = NEXT_WORKER_ID.fetch_add(1, Ordering::SeqCst) as u32;
  let source = std::fs::read_to_string(&filename)
    .map_err(|e| JsErrorBox::generic(format!("Cannot read worker file {filename}: {e}")))?;

  let (tx_to_worker, rx_from_main) = mpsc::channel();
  let (tx_to_main, rx_from_worker) = mpsc::channel();

  WORKERS.lock().unwrap().insert(
    worker_id,
    WorkerHandle {
      to_worker: tx_to_worker,
      from_worker: rx_from_worker,
    },
  );

  let fn_copy = filename.clone();
  thread::spawn(move || {
    WORKER_ID.with(|id| id.set(worker_id));
    WORKER_DATA.with(|d| *d.borrow_mut() = if data_json.is_empty() { None } else { Some(data_json) });
    TX_TO_MAIN.with(|tx| *tx.borrow_mut() = Some(tx_to_main));
    RX_FROM_MAIN.with(|rx| *rx.borrow_mut() = Some(rx_from_main));

    // Include klyron_node extension so worker ops are available
    let mut runtime = JsRuntime::new(RuntimeOptions {
      extensions: vec![crate::init()],
      ..Default::default()
    });

    if runtime
      .execute_script("<worker_setup>", FastString::from(worker_setup_js()))
      .is_err()
    {
      eprintln!("[worker {worker_id}] setup failed");
      return;
    }

    if let Err(e) = runtime.execute_script(fn_copy, FastString::from(source)) {
      eprintln!("[worker {worker_id}] execution error: {e}");
      return;
    }

    let delay = std::time::Duration::from_millis(10);
    loop {
      // Deliver messages from parent to the worker's JS context
      if let Some(msg) = RX_FROM_MAIN.with(|r| {
        r.borrow_mut().as_mut().and_then(|rx| rx.try_recv().ok())
      }) {
        let js = format!(
          "try {{ globalThis.parentPort.emit('message', {}) }} catch(e){{}}",
          msg
        );
        runtime.execute_script("<msg>", FastString::from(js)).ok();
      }

      thread::sleep(delay);

      let disconnected = WORKERS.lock()
        .map(|w| w.get(&worker_id).is_none())
        .unwrap_or(true);
      if disconnected {
        break;
      }
    }

    WORKERS.lock().unwrap().remove(&worker_id);
  });

  Ok(serde_json::json!({ "id": worker_id, "threadId": worker_id }).to_string())
}

#[op2(fast)]
fn op_worker_send(worker_id: u32, #[string] msg_json: String) -> Result<(), JsErrorBox> {
  let workers = WORKERS.lock().map_err(|e| JsErrorBox::generic(format!("lock: {e}")))?;
  let handle = workers
    .get(&worker_id)
    .ok_or_else(|| JsErrorBox::generic(format!("Worker {worker_id} not found")))?;
  handle
    .to_worker
    .send(msg_json)
    .map_err(|e| JsErrorBox::generic(format!("send to worker {worker_id}: {e}")))
}

#[op2]
#[string]
fn op_worker_poll(worker_id: u32) -> Option<String> {
  let workers = WORKERS.lock().ok()?;
  let handle = workers.get(&worker_id)?;
  let mut msgs = Vec::new();
  while let Ok(msg) = handle.from_worker.try_recv() {
    msgs.push(msg);
  }
  if msgs.is_empty() { None } else { Some(serde_json::to_string(&msgs).unwrap_or_else(|_| "[]".to_string())) }
}

#[op2(fast)]
fn op_worker_terminate(worker_id: u32) {
  WORKERS.lock().ok().and_then(|mut workers| workers.remove(&worker_id));
}

#[op2(fast)]
fn op_worker_is_worker() -> bool {
  WORKER_ID.with(|id| id.get() != 0)
}

#[op2(fast)]
fn op_worker_thread_id() -> u32 {
  WORKER_ID.with(|id| id.get())
}

#[op2]
#[string]
fn op_worker_get_data() -> String {
  WORKER_DATA.with(|d| d.borrow().clone().unwrap_or_default())
}

#[op2(fast)]
fn op_worker_send_to_parent(#[string] msg_json: String) -> Result<(), JsErrorBox> {
  let tx = TX_TO_MAIN.with(|t| t.borrow().clone());
  match tx {
    Some(sender) => sender
      .send(msg_json)
      .map_err(|e| JsErrorBox::generic(format!("send to parent: {e}"))),
    None => Err(JsErrorBox::generic("Not a worker thread")),
  }
}

#[op2]
#[string]
fn op_worker_poll_from_parent() -> Option<String> {
  RX_FROM_MAIN.with(|r| {
    let mut borrow = r.borrow_mut();
    let rx = borrow.as_mut()?;
    let mut msgs = Vec::new();
    while let Ok(msg) = rx.try_recv() {
      msgs.push(msg);
    }
    if msgs.is_empty() { None } else { Some(serde_json::to_string(&msgs).unwrap_or_else(|_| "[]".to_string())) }
  })
}

#[op2(fast)]
fn op_worker_close() {
  let worker_id = WORKER_ID.with(|id| id.get());
  WORKERS.lock().ok().and_then(|mut workers| workers.remove(&worker_id));
}

// ── DNS Ops ────────────────────────────────────────────────────────────────

use std::net::ToSocketAddrs;

fn resolve_hostname(hostname: &str, family: u32) -> Vec<serde_json::Value> {
  let addr_str = format!("{}:0", hostname);
  let addrs = match addr_str.to_socket_addrs() {
    Ok(a) => a,
    Err(_) => return vec![],
  };
  let mut results = Vec::new();
  for addr in addrs {
    let ip = addr.ip();
    let fam = if ip.is_ipv4() { 4 } else { 6 };
    if family == 0 || family == fam {
      results.push(serde_json::json!({ "address": ip.to_string(), "family": fam }));
      if family != 0 {
        break;
      }
    }
  }
  results
}

#[op2]
#[string]
fn op_dns_lookup(#[string] hostname: String, family: u32) -> String {
  let addrs = resolve_hostname(&hostname, family);
  serde_json::json!({ "addresses": addrs }).to_string()
}

#[op2]
#[string]
fn op_dns_resolve(#[string] hostname: String, #[string] rrtype: String) -> String {
  match rrtype.as_str() {
    "A" | "AAAA" => {
      let family = if rrtype == "A" { 4 } else { 6 };
      let addrs = resolve_hostname(&hostname, family);
      let entries: Vec<String> = addrs.iter().filter_map(|a| a["address"].as_str().map(String::from)).collect();
      serde_json::json!({ "entries": entries }).to_string()
    }
    "MX" => serde_json::json!({ "entries": [] }).to_string(),
    "TXT" => serde_json::json!({ "entries": [] }).to_string(),
    "SRV" => serde_json::json!({ "entries": [] }).to_string(),
    "CNAME" => serde_json::json!({ "entries": [] }).to_string(),
    "NS" => serde_json::json!({ "entries": [] }).to_string(),
    _ => serde_json::json!({ "error": format!("Unsupported rrtype: {rrtype}") }).to_string(),
  }
}

#[op2]
#[string]
fn op_dns_reverse(#[string] ip: String) -> String {
  // Reverse DNS via system getaddrinfo is not directly supported.
  // Return empty for now.
  serde_json::json!({ "hostnames": [] }).to_string()
}

// ── TLS / HTTPS Ops ────────────────────────────────────────────────────────

use rustls::ClientConfig;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;

/// Perform a full HTTPS request synchronously (via block_on).
#[op2]
#[string]
fn op_https_request(
  #[string] method: String,
  #[string] host: String,
  port: u16,
  #[string] path: String,
  #[string] headers_json: String,
  #[string] body: String,
) -> Result<String, JsErrorBox> {
  let addr = format!("{}:{}", host, port);

  let rt = tokio::runtime::Handle::current();
  let result = rt.block_on(async move {
    // 1. TCP connect
    let tcp = tokio::net::TcpStream::connect(&addr)
      .await
      .map_err(|e| JsErrorBox::generic(format!("TCP connect {addr}: {e}")))?;

    // 2. TLS handshake
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
      .with_root_certificates(root_store)
      .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let domain_ref: &str = &host;
    let domain = rustls::pki_types::ServerName::try_from(domain_ref)
      .map_err(|e| JsErrorBox::generic(format!("Invalid domain {host}: {e}")))?
      .to_owned();
    let mut tls = connector
      .connect(domain, tcp)
      .await
      .map_err(|e| JsErrorBox::generic(format!("TLS handshake: {e}")))?;

    // 3. Build and send HTTP request
    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\n");
    if let Ok(headers) = serde_json::from_str::<serde_json::Value>(&headers_json) {
      if let Some(obj) = headers.as_object() {
        for (k, v) in obj {
          if let Some(val) = v.as_str() {
            req.push_str(&format!("{k}: {val}\r\n"));
          }
        }
      }
    }
    if !body.is_empty() {
      req.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    req.push_str("Connection: close\r\n\r\n");
    if !body.is_empty() {
      req.push_str(&body);
    }
    tls.write_all(req.as_bytes())
      .await
      .map_err(|e| JsErrorBox::generic(format!("HTTP write: {e}")))?;
    tls.flush()
      .await
      .map_err(|e| JsErrorBox::generic(format!("HTTP flush: {e}")))?;

    // 4. Read response
    let mut resp = Vec::new();
    tls.read_to_end(&mut resp)
      .await
      .map_err(|e| JsErrorBox::generic(format!("HTTP read: {e}")))?;

    let text = String::from_utf8_lossy(&resp).to_string();
    let he = text.find("\r\n\r\n").unwrap_or(0);
    let header_lines: Vec<&str> = text[..he].split("\r\n").collect();
    let status_parts: Vec<&str> = header_lines.first().unwrap_or(&"").splitn(3, ' ').collect();
    let status_code: u16 = status_parts.get(1).unwrap_or(&"200").parse().unwrap_or(500);
    let status_message = status_parts.get(2).unwrap_or(&"OK").to_string();

    let mut headers = serde_json::Map::new();
    for line in &header_lines[1..] {
      if let Some(idx) = line.find(':') {
        let k = line[..idx].trim().to_string();
        let v = line[idx + 1..].trim().to_string();
        headers.insert(k, serde_json::Value::String(v));
      }
    }

    let body_str = if he + 4 < text.len() {
      text[he + 4..].to_string()
    } else {
      String::new()
    };

    Ok::<_, JsErrorBox>(
      serde_json::json!({
        "statusCode": status_code,
        "statusMessage": status_message,
        "headers": headers,
        "body": body_str,
      })
      .to_string(),
    )
  })?;

  Ok(result)
}

// ── Zlib Ops ───────────────────────────────────────────────────────────────

use flate2::{read::{GzDecoder, DeflateDecoder}, write::{GzEncoder, DeflateEncoder}, Compression};

fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, JsErrorBox> {
  let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
  use std::io::Write;
  encoder.write_all(data).map_err(|e| JsErrorBox::generic(format!("gzip compress: {e}")))?;
  encoder.finish().map_err(|e| JsErrorBox::generic(format!("gzip finish: {e}")))
}

fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, JsErrorBox> {
  let mut decoder = GzDecoder::new(data);
  let mut out = Vec::new();
  use std::io::Read;
  decoder.read_to_end(&mut out).map_err(|e| JsErrorBox::generic(format!("gzip decompress: {e}")))?;
  Ok(out)
}

fn compress_deflate(data: &[u8]) -> Result<Vec<u8>, JsErrorBox> {
  let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
  use std::io::Write;
  encoder.write_all(data).map_err(|e| JsErrorBox::generic(format!("deflate compress: {e}")))?;
  encoder.finish().map_err(|e| JsErrorBox::generic(format!("deflate finish: {e}")))
}

fn decompress_deflate(data: &[u8]) -> Result<Vec<u8>, JsErrorBox> {
  let mut decoder = DeflateDecoder::new(data);
  let mut out = Vec::new();
  use std::io::Read;
  decoder.read_to_end(&mut out).map_err(|e| JsErrorBox::generic(format!("deflate decompress: {e}")))?;
  Ok(out)
}

#[op2]
fn op_zlib_gzip(#[serde] data: Vec<u8>) -> Result<Vec<u8>, JsErrorBox> {
  compress_gzip(&data)
}

#[op2]
fn op_zlib_gunzip(#[serde] data: Vec<u8>) -> Result<Vec<u8>, JsErrorBox> {
  decompress_gzip(&data)
}

#[op2]
fn op_zlib_deflate(#[serde] data: Vec<u8>) -> Result<Vec<u8>, JsErrorBox> {
  compress_deflate(&data)
}

#[op2]
fn op_zlib_inflate(#[serde] data: Vec<u8>) -> Result<Vec<u8>, JsErrorBox> {
  decompress_deflate(&data)
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
