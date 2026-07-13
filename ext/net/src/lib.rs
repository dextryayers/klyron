use std::sync::{Arc, Mutex};

use deno_core::{extension, op2, Extension, OpState};
use deno_error::JsErrorBox;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

type ConnStore = Arc<Mutex<Vec<Option<Arc<Mutex<TcpStream>>>>>>;

extension!(
  klyron_net,
  ops = [op_net_connect, op_net_send, op_net_recv, op_net_close],
  esm_entry_point = "ext:klyron_net/net.js",
  esm = [dir "js", "net.js"],
  state = |state| { state.put::<ConnStore>(Arc::new(Mutex::new(Vec::new()))); },
);

pub fn init() -> Extension {
  klyron_net::init()
}

#[derive(serde::Serialize)]
struct ConnectResult {
  rid: i32,
  local_addr: String,
  peer_addr: String,
}

#[op2]
#[serde]
fn op_net_connect(state: &mut OpState, #[string] addr: String) -> Result<ConnectResult, JsErrorBox> {
  let rt = tokio::runtime::Handle::current();
  let conn = rt.block_on(async { TcpStream::connect(&addr).await }).map_err(|e| JsErrorBox::generic(format!("connect {addr}: {e}")))?;
  let local_addr = conn.local_addr().map(|a| a.to_string()).unwrap_or_default();
  let peer_addr = conn.peer_addr().map(|a| a.to_string()).unwrap_or_default();
  let stream = Arc::new(Mutex::new(conn));

  let store = state.borrow_mut::<ConnStore>();
  let mut guard = store.lock().unwrap();
  let rid = guard.len() as i32;
  guard.push(Some(stream));
  Ok(ConnectResult { rid, local_addr, peer_addr })
}

#[op2(fast)]
fn op_net_send(state: &mut OpState, rid: i32, #[string] data: String) -> Result<(), JsErrorBox> {
  let store = state.borrow::<ConnStore>();
  let guard = store.lock().unwrap();
  if let Some(Some(stream)) = guard.get(rid as usize) {
    let rt = tokio::runtime::Handle::current();
    let mut stream = stream.lock().unwrap();
    rt.block_on(async { stream.write_all(data.as_bytes()).await }).map_err(|e| JsErrorBox::generic(format!("send: {e}")))
  } else {
    Err(JsErrorBox::generic(format!("connection {rid} not found")))
  }
}

#[op2]
#[string]
fn op_net_recv(state: &mut OpState, rid: i32) -> Result<String, JsErrorBox> {
  let store = state.borrow::<ConnStore>();
  let guard = store.lock().unwrap();
  if let Some(Some(stream)) = guard.get(rid as usize) {
    let rt = tokio::runtime::Handle::current();
    let mut stream = stream.lock().unwrap();
    let mut buf = vec![0u8; 65536];
    let n = rt.block_on(async { stream.read(&mut buf).await }).map_err(|e| JsErrorBox::generic(format!("recv: {e}")))?;
    if n == 0 { return Ok(String::new()); }
    buf.truncate(n);
    Ok(String::from_utf8_lossy(&buf).to_string())
  } else {
    Err(JsErrorBox::generic(format!("connection {rid} not found")))
  }
}

#[op2(fast)]
fn op_net_close(state: &mut OpState, rid: i32) -> Result<(), JsErrorBox> {
  let store = state.borrow_mut::<ConnStore>();
  let mut guard = store.lock().unwrap();
  if let Some(slot) = guard.get_mut(rid as usize) {
    *slot = None;
    Ok(())
  } else {
    Err(JsErrorBox::generic(format!("connection {rid} not found")))
  }
}
