use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: String,
    pub retry: Option<u64>,
}

pub struct SseStream {
    rx: mpsc::Receiver<Result<SseEvent, String>>,
}

impl SseStream {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel(64);

        let client = Client::new();
        let resp = client
            .get(url)
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let stream = resp.bytes_stream();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            drain_sse(stream, tx_clone).await;
        });

        Ok(Self { rx })
    }

    pub async fn next_event(&mut self) -> Option<Result<SseEvent, String>> {
        self.rx.recv().await
    }
}

async fn drain_sse(
    mut stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
    tx: mpsc::Sender<Result<SseEvent, String>>,
) {
    let mut buffer = String::new();
    let mut current = SseEvent {
        id: None,
        event: None,
        data: String::new(),
        retry: None,
    };

    while let Some(item) = stream.next().await {
        let chunk = match item {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(Err(e.to_string())).await;
                break;
            }
        };

        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim_end().to_string();
            buffer = buffer[pos + 1..].to_string();

            if line.is_empty() {
                let event = std::mem::replace(
                    &mut current,
                    SseEvent {
                        id: None,
                        event: None,
                        data: String::new(),
                        retry: None,
                    },
                );
                let _ = tx.send(Ok(event)).await;
                continue;
            }

            if let Some(value) = line.strip_prefix("id:") {
                current.id = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("event:") {
                current.event = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("data:") {
                if !current.data.is_empty() {
                    current.data.push('\n');
                }
                current.data.push_str(value.trim());
            } else if let Some(value) = line.strip_prefix("retry:") {
                current.retry = value.trim().parse().ok();
            }
        }
    }
}

impl Stream for SseStream {
    type Item = Result<SseEvent, String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}
