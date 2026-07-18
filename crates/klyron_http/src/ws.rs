use futures_util::StreamExt;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub struct WebSocketClient {
    pub outgoing: Sender<String>,
    pub incoming: Receiver<String>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl WebSocketClient {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let (ws_stream, _) = connect_async(url).await?;
        let (write, read) = ws_stream.split();

        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<String>(256);
        let (incoming_tx, incoming_rx) = mpsc::channel::<String>(256);

        let write_handle = tokio::spawn(async move {
            use futures_util::SinkExt;
            let mut write = write;
            while let Some(msg) = outgoing_rx.recv().await {
                if write.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        let read_handle = tokio::spawn(async move {
            use futures_util::StreamExt;
            let mut read = read;
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    if incoming_tx.send(text).await.is_err() {
                        break;
                    }
                }
            }
        });

        let handle = tokio::spawn(async move {
            let _ = tokio::join!(write_handle, read_handle);
        });

        Ok(Self {
            outgoing: outgoing_tx,
            incoming: incoming_rx,
            handle: Some(handle),
        })
    }

    pub async fn send(&self, msg: impl Into<String>) -> anyhow::Result<()> {
        self.outgoing.send(msg.into()).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Option<String> {
        self.incoming.recv().await
    }

    pub fn close(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl Drop for WebSocketClient {
    fn drop(&mut self) {
        self.close();
    }
}

pub async fn ws_connect_echo(url: &str, message: &str) -> anyhow::Result<String> {
    let mut client = WebSocketClient::connect(url).await?;
    client.send(message).await?;
    let response = client.recv().await.unwrap_or_default();
    client.close();
    Ok(response)
}
