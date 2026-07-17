use std::sync::{Arc, Mutex};

use deno_core::{extension, op2, Extension, OpState};
use deno_error::JsErrorBox;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

type ConnStore = Arc<Mutex<Vec<Option<Arc<Mutex<TcpStream>>>>>>;
type ListenerStore = Arc<Mutex<Vec<Option<TcpListener>>>>;

extension!(
  klyron_net,
  ops = [op_net_connect, op_net_send, op_net_recv, op_net_close, op_net_listen, op_net_accept, op_net_listen_close, op_net_send_bin, op_net_recv_bin, op_net_sockname, op_net_peername],
  esm_entry_point = "ext:klyron_net/net.js",
  esm = [dir "js", "net.js"],
  state = |state| {
    state.put::<ConnStore>(Arc::new(Mutex::new(Vec::new())));
    state.put::<ListenerStore>(Arc::new(Mutex::new(Vec::new())));
  },
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

fn op_net_connect_impl(state: &mut OpState, addr: String) -> Result<ConnectResult, JsErrorBox> {
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

#[op2]
#[serde]
fn op_net_connect(state: &mut OpState, #[string] addr: String) -> Result<ConnectResult, JsErrorBox> {
  op_net_connect_impl(state, addr)
}

fn op_net_send_impl(state: &mut OpState, rid: i32, data: String) -> Result<(), JsErrorBox> {
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

#[op2(fast)]
fn op_net_send(state: &mut OpState, rid: i32, #[string] data: String) -> Result<(), JsErrorBox> {
  op_net_send_impl(state, rid, data)
}

fn op_net_send_bin_impl(state: &mut OpState, rid: i32, data: Vec<u8>) -> Result<(), JsErrorBox> {
  let store = state.borrow::<ConnStore>();
  let guard = store.lock().unwrap();
  if let Some(Some(stream)) = guard.get(rid as usize) {
    let rt = tokio::runtime::Handle::current();
    let mut stream = stream.lock().unwrap();
    rt.block_on(async { stream.write_all(&data).await }).map_err(|e| JsErrorBox::generic(format!("send_bin: {e}")))
  } else {
    Err(JsErrorBox::generic(format!("connection {rid} not found")))
  }
}

#[op2]
fn op_net_send_bin(state: &mut OpState, rid: i32, #[serde] data: Vec<u8>) -> Result<(), JsErrorBox> {
  op_net_send_bin_impl(state, rid, data)
}

fn op_net_recv_impl(state: &mut OpState, rid: i32) -> Result<String, JsErrorBox> {
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

#[op2]
#[string]
fn op_net_recv(state: &mut OpState, rid: i32) -> Result<String, JsErrorBox> {
  op_net_recv_impl(state, rid)
}

fn op_net_recv_bin_impl(state: &mut OpState, rid: i32) -> Result<Vec<u8>, JsErrorBox> {
  let store = state.borrow::<ConnStore>();
  let guard = store.lock().unwrap();
  if let Some(Some(stream)) = guard.get(rid as usize) {
    let rt = tokio::runtime::Handle::current();
    let mut stream = stream.lock().unwrap();
    let mut buf = vec![0u8; 65536];
    let n = rt.block_on(async { stream.read(&mut buf).await }).map_err(|e| JsErrorBox::generic(format!("recv_bin: {e}")))?;
    buf.truncate(n);
    Ok(buf)
  } else {
    Err(JsErrorBox::generic(format!("connection {rid} not found")))
  }
}

#[op2]
#[serde]
fn op_net_recv_bin(state: &mut OpState, rid: i32) -> Result<Vec<u8>, JsErrorBox> {
  op_net_recv_bin_impl(state, rid)
}

fn op_net_close_impl(state: &mut OpState, rid: i32) -> Result<(), JsErrorBox> {
  let store = state.borrow_mut::<ConnStore>();
  let mut guard = store.lock().unwrap();
  if let Some(slot) = guard.get_mut(rid as usize) {
    *slot = None;
    Ok(())
  } else {
    Err(JsErrorBox::generic(format!("connection {rid} not found")))
  }
}

#[op2(fast)]
fn op_net_close(state: &mut OpState, rid: i32) -> Result<(), JsErrorBox> {
  op_net_close_impl(state, rid)
}

fn op_net_listen_impl(state: &mut OpState, addr: String) -> Result<i32, JsErrorBox> {
  let rt = tokio::runtime::Handle::current();
  let listener = rt.block_on(async { TcpListener::bind(&addr).await }).map_err(|e| JsErrorBox::generic(format!("listen {addr}: {e}")))?;
  let store = state.borrow_mut::<ListenerStore>();
  let mut guard = store.lock().unwrap();
  let rid = guard.len() as i32;
  guard.push(Some(listener));
  Ok(rid)
}

#[op2(fast)]
fn op_net_listen(state: &mut OpState, #[string] addr: String) -> Result<i32, JsErrorBox> {
  op_net_listen_impl(state, addr)
}

fn op_net_accept_impl(state: &mut OpState, listen_rid: i32) -> Result<ConnectResult, JsErrorBox> {
  let stream = {
    let lstore = state.borrow::<ListenerStore>();
    let lguard = lstore.lock().unwrap();
    let listener = match lguard.get(listen_rid as usize) {
      Some(Some(l)) => l,
      _ => return Err(JsErrorBox::generic(format!("listener {listen_rid} not found"))),
    };
    let rt = tokio::runtime::Handle::current();
    let (conn, peer_addr) = rt.block_on(async { listener.accept().await }).map_err(|e| JsErrorBox::generic(format!("accept: {e}")))?;
    let local_addr = conn.local_addr().map(|a| a.to_string()).unwrap_or_default();
    let stream = Arc::new(Mutex::new(conn));
    (stream, local_addr, peer_addr.to_string())
  };

  let store = state.borrow_mut::<ConnStore>();
  let mut guard = store.lock().unwrap();
  let rid = guard.len() as i32;
  guard.push(Some(stream.0));
  Ok(ConnectResult { rid, local_addr: stream.1, peer_addr: stream.2 })
}

#[op2]
#[serde]
fn op_net_accept(state: &mut OpState, listen_rid: i32) -> Result<ConnectResult, JsErrorBox> {
  op_net_accept_impl(state, listen_rid)
}

fn op_net_listen_close_impl(state: &mut OpState, rid: i32) -> Result<(), JsErrorBox> {
  let store = state.borrow_mut::<ListenerStore>();
  let mut guard = store.lock().unwrap();
  if let Some(slot) = guard.get_mut(rid as usize) {
    *slot = None;
    Ok(())
  } else {
    Err(JsErrorBox::generic(format!("listener {rid} not found")))
  }
}

#[op2(fast)]
fn op_net_listen_close(state: &mut OpState, rid: i32) -> Result<(), JsErrorBox> {
  op_net_listen_close_impl(state, rid)
}

#[derive(serde::Serialize)]
struct AddrResult {
  address: String,
  family: String,
  port: i32,
}

fn op_net_sockname_impl(state: &mut OpState, rid: i32) -> Result<AddrResult, JsErrorBox> {
  let store = state.borrow::<ConnStore>();
  let guard = store.lock().unwrap();
  if let Some(Some(stream)) = guard.get(rid as usize) {
    let stream = stream.lock().unwrap();
    let addr = stream.local_addr().map_err(|e| JsErrorBox::generic(format!("sockname: {e}")))?;
    Ok(AddrResult {
      address: addr.ip().to_string(),
      family: if addr.is_ipv4() { "IPv4".to_string() } else { "IPv6".to_string() },
      port: addr.port() as i32,
    })
  } else {
    Err(JsErrorBox::generic(format!("connection {rid} not found")))
  }
}

#[op2]
#[serde]
fn op_net_sockname(state: &mut OpState, rid: i32) -> Result<AddrResult, JsErrorBox> {
  op_net_sockname_impl(state, rid)
}

fn op_net_peername_impl(state: &mut OpState, rid: i32) -> Result<AddrResult, JsErrorBox> {
  let store = state.borrow::<ConnStore>();
  let guard = store.lock().unwrap();
  if let Some(Some(stream)) = guard.get(rid as usize) {
    let stream = stream.lock().unwrap();
    let addr = stream.peer_addr().map_err(|e| JsErrorBox::generic(format!("peername: {e}")))?;
    Ok(AddrResult {
      address: addr.ip().to_string(),
      family: if addr.is_ipv4() { "IPv4".to_string() } else { "IPv6".to_string() },
      port: addr.port() as i32,
    })
  } else {
    Err(JsErrorBox::generic(format!("connection {rid} not found")))
  }
}

#[op2]
#[serde]
fn op_net_peername(state: &mut OpState, rid: i32) -> Result<AddrResult, JsErrorBox> {
  op_net_peername_impl(state, rid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_returns_extension() {
        let ext = init();
        assert_eq!(ext.name, "klyron_net");
    }

    fn run_in_runtime(f: impl FnOnce()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        f();
    }

    fn make_state() -> OpState {
        let mut state = OpState::new(None);
        state.put::<ConnStore>(Arc::new(Mutex::new(Vec::new())));
        state.put::<ListenerStore>(Arc::new(Mutex::new(Vec::new())));
        state
    }

    #[test]
    fn test_net_connect_refused() {
        run_in_runtime(|| {
            let mut state = make_state();
            let result = op_net_connect_impl(&mut state, "127.0.0.1:1".to_string());
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_net_listen_and_accept() {
        run_in_runtime(|| {
            let mut state = make_state();
            let lrid = op_net_listen_impl(&mut state, "127.0.0.1:0".to_string()).unwrap();
            assert!(lrid >= 0);

            // Get the actual address
            let rt = tokio::runtime::Handle::current();
            let addr = {
                let lstore = state.borrow::<ListenerStore>();
                let lguard = lstore.lock().unwrap();
                let listener = lguard.get(lrid as usize).unwrap().as_ref().unwrap();
                listener.local_addr().unwrap()
            };

            // Connect to the listener
            let conn_result = op_net_connect_impl(&mut state, addr.to_string()).unwrap();
            assert_eq!(conn_result.peer_addr, addr.to_string());

            // Accept the connection
            let accept_result = op_net_accept_impl(&mut state, lrid).unwrap();
            assert_eq!(accept_result.rid, conn_result.rid + 1);

            // Clean up
            op_net_listen_close_impl(&mut state, lrid).unwrap();
        });
    }

    #[test]
    fn test_net_close_invalid_rid() {
        run_in_runtime(|| {
            let mut state = make_state();
            let result = op_net_close_impl(&mut state, 999);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_net_send_invalid_rid() {
        run_in_runtime(|| {
            let mut state = make_state();
            let result = op_net_send_impl(&mut state, 999, "data".to_string());
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_net_recv_invalid_rid() {
        run_in_runtime(|| {
            let mut state = make_state();
            let result = op_net_recv_impl(&mut state, 999);
            assert!(result.is_err());
        });
    }
}
