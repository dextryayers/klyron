use std::sync::{Arc, Mutex};

use deno_core::{extension, op2, Extension, OpState};
use deno_error::JsErrorBox;

type ServerHandle = Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>;

extension!(
  klyron_http,
  ops = [op_http_serve, op_http_stop],
  esm_entry_point = "ext:klyron_http/http.js",
  esm = [dir "js", "http.js"],
  state = |state| { state.put::<ServerHandle>(Arc::new(Mutex::new(None))); },
);

pub fn init() -> Extension {
  klyron_http::init()
}

#[op2]
#[string]
fn op_http_serve(state: &mut OpState, #[string] addr: String) -> Result<String, JsErrorBox> {
  let bind_addr = addr.clone();
  let rt = tokio::runtime::Handle::current();
  let handle = rt.spawn(async move {
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
      Ok(l) => l,
      Err(e) => { eprintln!("HTTP serve bind error: {e}"); return; }
    };
    loop {
      match listener.accept().await {
        Ok((stream, _)) => {
          tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            if let Ok(n) = stream.try_read(&mut buf) {
              if n > 0 {
                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nKlyron HTTP\r\n";
                let _ = stream.try_write(response.as_bytes());
              }
            }
          });
        }
        Err(_) => break,
      }
    }
  });

  let server_handle = state.borrow_mut::<ServerHandle>();
  *server_handle.lock().unwrap() = Some(handle);
  Ok(format!("Serving on http://{addr}"))
}

#[op2(fast)]
fn op_http_stop(state: &mut OpState) -> Result<(), JsErrorBox> {
  let server_handle = state.borrow_mut::<ServerHandle>();
  if let Some(handle) = server_handle.lock().unwrap().take() {
    handle.abort();
  }
  Ok(())
}
