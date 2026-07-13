use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use deno_core::{extension, op2, Extension, OpState};
use deno_error::JsErrorBox;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

type WsId = u64;

struct WsConnection {
  write_tx: mpsc::Sender<String>,
  read_rx: Arc<Mutex<Option<mpsc::Receiver<String>>>>,
}

struct WsState {
  next_id: WsId,
  connections: HashMap<WsId, WsConnection>,
}

extension!(
  klyron_ws,
  ops = [op_ws_connect, op_ws_send, op_ws_recv, op_ws_close],
  esm_entry_point = "ext:klyron_ws/ws.js",
  esm = [dir "js", "ws.js"],
  state = |state| {
    state.put::<WsState>(WsState { next_id: 1, connections: HashMap::new() });
  },
);

pub fn init() -> Extension {
  klyron_ws::init()
}

#[op2]
#[serde]
fn op_ws_connect(state: &mut OpState, #[string] url: String) -> Result<serde_json::Value, JsErrorBox> {
  let ws_state = state.borrow_mut::<WsState>();
  let id = ws_state.next_id;
  ws_state.next_id += 1;

  let (write_tx, mut write_rx) = mpsc::channel::<String>(64);
  let (read_tx, read_rx) = mpsc::channel::<String>(64);

  let url2 = url.clone();
  tokio::spawn(async move {
    match tokio_tungstenite::connect_async(&url2).await {
      Ok((mut ws_stream, _)) => {
        // Mark as connected by sending a ready signal
        let _ = read_tx.send("__connected__".to_string()).await;

        loop {
          tokio::select! {
            msg = ws_stream.next() => {
              match msg {
                Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                  let _ = read_tx.send(format!("__text__:{text}")).await;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data))) => {
                  let _ = read_tx.send(format!("__binary__:{}", String::from_utf8_lossy(&data))).await;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => {
                  let _ = read_tx.send("__close__".to_string()).await;
                  break;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Ping(_))) => {
                  // tungstenite handles pong automatically
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Pong(_))) => {}
                Some(Ok(tokio_tungstenite::tungstenite::Message::Frame(_))) => {
                  // Raw frame, skip
                }
                Some(Err(e)) => {
                  let _ = read_tx.send(format!("__error__:{e}")).await;
                  break;
                }
                None => {
                  let _ = read_tx.send("__close__".to_string()).await;
                  break;
                }
              }
            }
            msg = write_rx.recv() => {
              match msg {
                Some(text) => {
                  if text == "__close__" {
                    let _ = ws_stream.close(None).await;
                    break;
                  }
                  if let Err(e) = ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(text)).await {
                    let _ = read_tx.send(format!("__error__:{e}")).await;
                    break;
                  }
                }
                None => break,
              }
            }
          }
        }
      }
      Err(e) => {
        let _ = read_tx.send(format!("__error__:connect failed: {e}")).await;
      }
    }
  });

  let conn = WsConnection {
    write_tx,
    read_rx: Arc::new(Mutex::new(Some(read_rx))),
  };
  ws_state.connections.insert(id, conn);

  Ok(serde_json::json!({ "id": id }))
}

#[op2]
#[string]
fn op_ws_recv(state: &mut OpState, #[serde] id: WsId) -> Result<String, JsErrorBox> {
  let ws_state = state.borrow_mut::<WsState>();
  let conn = ws_state.connections.get(&id).ok_or_else(|| JsErrorBox::generic("WebSocket not found"))?;
  let mut rx_guard = conn.read_rx.lock().map_err(|_| JsErrorBox::generic("lock error"))?;
  if let Some(rx) = rx_guard.as_mut() {
    match rx.try_recv() {
      Ok(msg) => Ok(msg),
      Err(_) => Ok("__no_message__".to_string()),
    }
  } else {
    Ok("__closed__".to_string())
  }
}

#[op2]
fn op_ws_send(state: &mut OpState, #[serde] id: WsId, #[string] message: String) -> Result<(), JsErrorBox> {
  let ws_state = state.borrow_mut::<WsState>();
  let conn = ws_state.connections.get(&id).ok_or_else(|| JsErrorBox::generic("WebSocket not found"))?;
  conn
    .write_tx
    .blocking_send(message)
    .map_err(|e| JsErrorBox::generic(format!("WebSocket send error: {e}")))
}

#[op2]
fn op_ws_close(state: &mut OpState, #[serde] id: WsId) -> Result<(), JsErrorBox> {
  let ws_state = state.borrow_mut::<WsState>();
  let conn = ws_state.connections.remove(&id).ok_or_else(|| JsErrorBox::generic("WebSocket not found"))?;
  let _ = conn.write_tx.blocking_send("__close__".to_string());
  Ok(())
}
