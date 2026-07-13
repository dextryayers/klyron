use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use base64::Engine;
use deno_core::{extension, op2, Extension, OpState};
use deno_error::JsErrorBox;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

type WsId = u64;

#[derive(Debug, Clone, PartialEq)]
enum WsStateEnum {
  Connecting,
  Open,
  Closing,
  Closed,
}

struct WsConnection {
  write_tx: mpsc::Sender<WsMessage>,
  read_rx: Arc<Mutex<Option<mpsc::Receiver<String>>>>,
  state: Arc<Mutex<WsStateEnum>>,
}

#[derive(Debug, Clone)]
enum WsMessage {
  Text(String),
  Binary(Vec<u8>),
  Ping(Vec<u8>),
  Close(Option<u16>, Option<String>),
}

struct WsState {
  next_id: WsId,
  connections: HashMap<WsId, WsConnection>,
}

impl WsState {
  fn get_conn(&mut self, id: WsId) -> Result<&mut WsConnection, JsErrorBox> {
    self.connections.get_mut(&id).ok_or_else(|| JsErrorBox::generic(format!("WebSocket {id} not found")))
  }
}

extension!(
  klyron_ws,
  ops = [
    op_ws_connect, op_ws_send, op_ws_send_binary,
    op_ws_recv, op_ws_close, op_ws_ready_state,
    op_ws_ping,
  ],
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
fn op_ws_connect(
  state: &mut OpState,
  #[string] url: String,
) -> Result<serde_json::Value, JsErrorBox> {
  let ws_state = state.borrow_mut::<WsState>();
  let id = ws_state.next_id;
  ws_state.next_id += 1;

  let (write_tx, mut write_rx) = mpsc::channel::<WsMessage>(64);
  let (read_tx, read_rx) = mpsc::channel::<String>(64);
  let state_arc = Arc::new(Mutex::new(WsStateEnum::Connecting));

  let url2 = url.clone();
  let state2 = state_arc.clone();

  tokio::spawn(async move {
    match tokio_tungstenite::connect_async(&url2).await {
      Ok((mut ws_stream, _response)) => {
        *state2.lock().unwrap() = WsStateEnum::Open;
        let _ = read_tx.send("__connected__".to_string()).await;

        loop {
          tokio::select! {
            msg = ws_stream.next() => {
              match msg {
                Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                  let _ = read_tx.send(format!("__text__:{text}")).await;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data))) => {
                  let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                  let _ = read_tx.send(format!("__binary__:{b64}")).await;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Close(frame))) => {
                  let msg = match frame {
                    Some(ref f) => {
                      let reason_str: &str = f.reason.as_ref();
                      format!("__close__:{}:{}", f.code, reason_str)
                    }
                    None => "__close__:1005:".to_string(),
                  };
                  let _ = read_tx.send(msg).await;
                  *state2.lock().unwrap() = WsStateEnum::Closed;
                  break;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Ping(data))) => {
                  // Reply with pong automatically (tungstenite handles this)
                  let _ = read_tx.send(format!("__ping__:{}", String::from_utf8_lossy(&data))).await;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Pong(data))) => {
                  let _ = read_tx.send(format!("__pong__:{}", String::from_utf8_lossy(&data))).await;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Frame(_))) => {}
                Some(Err(e)) => {
                  let _ = read_tx.send(format!("__error__:{e}")).await;
                  *state2.lock().unwrap() = WsStateEnum::Closed;
                  break;
                }
                None => {
                  *state2.lock().unwrap() = WsStateEnum::Closed;
                  let _ = read_tx.send("__close__:1006:".to_string()).await;
                  break;
                }
              }
            }
            msg = write_rx.recv() => {
              match msg {
                Some(WsMessage::Text(text)) => {
                  if let Err(e) = ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(text)).await {
                    let _ = read_tx.send(format!("__error__:{e}")).await;
                    break;
                  }
                }
                Some(WsMessage::Binary(data)) => {
                  if let Err(e) = ws_stream.send(tokio_tungstenite::tungstenite::Message::Binary(data)).await {
                    let _ = read_tx.send(format!("__error__:{e}")).await;
                    break;
                  }
                }
                Some(WsMessage::Ping(data)) => {
                  let _ = ws_stream.send(tokio_tungstenite::tungstenite::Message::Ping(data)).await;
                }
                Some(WsMessage::Close(code, reason)) => {
                  use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
                  let reason_str = reason.unwrap_or_default();
                  let frame = tokio_tungstenite::tungstenite::protocol::CloseFrame {
                    code: CloseCode::from(code.unwrap_or(1000)),
                    reason: std::borrow::Cow::Owned(reason_str),
                  };
                  let _ = ws_stream.send(tokio_tungstenite::tungstenite::Message::Close(Some(frame))).await;
                  let _ = ws_stream.close(None).await;
                  *state2.lock().unwrap() = WsStateEnum::Closed;
                  break;
                }
                None => break,
              }
            }
          }
        }
      }
      Err(e) => {
        *state2.lock().unwrap() = WsStateEnum::Closed;
        let _ = read_tx.send(format!("__error__:connect failed: {e}")).await;
      }
    }
  });

  let conn = WsConnection {
    write_tx,
    read_rx: Arc::new(Mutex::new(Some(read_rx))),
    state: state_arc,
  };
  ws_state.connections.insert(id, conn);

  Ok(serde_json::json!({ "id": id }))
}

#[op2]
#[string]
fn op_ws_recv(
  state: &mut OpState,
  #[serde] id: WsId,
) -> Result<String, JsErrorBox> {
  let ws_state = state.borrow_mut::<WsState>();
  let conn = ws_state.get_conn(id)?;
  let mut rx_guard = conn.read_rx.lock().map_err(|_| JsErrorBox::generic("lock error"))?;
  if let Some(rx) = rx_guard.as_mut() {
    match rx.try_recv() {
      Ok(msg) => Ok(msg),
      Err(tokio::sync::mpsc::error::TryRecvError::Empty) => Ok("__no_message__".to_string()),
      Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
        *rx_guard = None;
        Ok("__closed__".to_string())
      }
    }
  } else {
    Ok("__closed__".to_string())
  }
}

#[op2]
fn op_ws_send(
  state: &mut OpState,
  #[serde] id: WsId,
  #[string] message: String,
) -> Result<(), JsErrorBox> {
  let ws_state = state.borrow_mut::<WsState>();
  let conn = ws_state.get_conn(id)?;
  conn
    .write_tx
    .blocking_send(WsMessage::Text(message))
    .map_err(|e| JsErrorBox::generic(format!("WebSocket send error: {e}")))
}

#[op2]
fn op_ws_send_binary(
  state: &mut OpState,
  #[serde] id: WsId,
  #[string] data_b64: String,
) -> Result<(), JsErrorBox> {
  let data = base64::engine::general_purpose::STANDARD
    .decode(&data_b64)
    .map_err(|e| JsErrorBox::generic(format!("Base64 decode error: {e}")))?;
  let ws_state = state.borrow_mut::<WsState>();
  let conn = ws_state.get_conn(id)?;
  conn
    .write_tx
    .blocking_send(WsMessage::Binary(data))
    .map_err(|e| JsErrorBox::generic(format!("WebSocket binary send error: {e}")))
}

#[op2]
fn op_ws_close(
  state: &mut OpState,
  #[serde] id: WsId,
  #[string] close_json: Option<String>,
) -> Result<(), JsErrorBox> {
  let (code, reason) = if let Some(json) = close_json {
    let v: serde_json::Value = serde_json::from_str(&json).unwrap_or_default();
    let code = v.get("code").and_then(|c| c.as_u64()).map(|c| c as u16);
    let reason = v.get("reason").and_then(|r| r.as_str()).map(|s| s.to_string());
    (code, reason)
  } else {
    (Some(1000), None)
  };

  let ws_state = state.borrow_mut::<WsState>();
  let conn = ws_state.get_conn(id)?;
  *conn.state.lock().unwrap() = WsStateEnum::Closing;
  conn
    .write_tx
    .blocking_send(WsMessage::Close(code, reason))
    .map_err(|e| JsErrorBox::generic(format!("WebSocket close error: {e}")))
}

#[op2]
#[serde]
fn op_ws_ready_state(
  state: &mut OpState,
  #[serde] id: WsId,
) -> Result<serde_json::Value, JsErrorBox> {
  let ws_state = state.borrow_mut::<WsState>();
  let conn = ws_state.get_conn(id)?;
  let state_val = conn.state.lock().unwrap().clone();
  let num = match state_val {
    WsStateEnum::Connecting => 0,
    WsStateEnum::Open => 1,
    WsStateEnum::Closing => 2,
    WsStateEnum::Closed => 3,
  };
  Ok(serde_json::json!(num))
}

#[op2]
fn op_ws_ping(
  state: &mut OpState,
  #[serde] id: WsId,
  #[string] data: Option<String>,
) -> Result<(), JsErrorBox> {
  let ws_state = state.borrow_mut::<WsState>();
  let conn = ws_state.get_conn(id)?;
  let payload = data.unwrap_or_default().into_bytes();
  conn
    .write_tx
    .blocking_send(WsMessage::Ping(payload))
    .map_err(|e| JsErrorBox::generic(format!("WebSocket ping error: {e}")))
}
