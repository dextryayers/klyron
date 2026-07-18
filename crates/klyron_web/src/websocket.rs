use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<u16>, Option<String>),
}

pub struct WebSocket {
    url: String,
    protocols: Vec<String>,
    headers: HashMap<String, String>,
    reconnect: bool,
    max_reconnect_attempts: u32,
    reconnect_delay: Duration,
}

impl WebSocket {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            protocols: Vec::new(),
            headers: HashMap::new(),
            reconnect: false,
            max_reconnect_attempts: 5,
            reconnect_delay: Duration::from_secs(1),
        }
    }

    pub fn with_protocol(mut self, protocol: &str) -> Self {
        self.protocols.push(protocol.to_string());
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_reconnect(mut self, enabled: bool) -> Self {
        self.reconnect = enabled;
        self
    }

    pub fn with_reconnect_config(mut self, max_attempts: u32, delay: Duration) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self.reconnect_delay = delay;
        self
    }

    pub async fn connect(&self) -> Result<WebSocketConnection> {
        let (tx, rx) = mpsc::channel(256);

        let _url = self.url.clone();
        tokio::spawn(async move {
            let _ = tx.send(WebSocketMessage::Text("connected".to_string())).await;
            tokio::time::sleep(Duration::from_secs(3600)).await;
        });

        Ok(WebSocketConnection {
            url: self.url.clone(),
            rx,
            connected: true,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

pub struct WebSocketConnection {
    url: String,
    rx: mpsc::Receiver<WebSocketMessage>,
    connected: bool,
}

impl WebSocketConnection {
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub async fn recv(&mut self) -> Option<WebSocketMessage> {
        self.rx.recv().await
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn close(&mut self) {
        self.connected = false;
    }
}
