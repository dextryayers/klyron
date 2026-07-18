use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_stream::Stream;

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
    pub id: Option<String>,
    pub retry: Option<u64>,
}

impl SseEvent {
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            event: None,
            data: data.into(),
            id: None,
            retry: None,
        }
    }

    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_retry(mut self, ms: u64) -> Self {
        self.retry = Some(ms);
        self
    }

    pub fn to_string(&self) -> String {
        let mut buf = String::new();
        if let Some(ref id) = self.id {
            buf.push_str(&format!("id: {}\n", id));
        }
        if let Some(ref event) = self.event {
            buf.push_str(&format!("event: {}\n", event));
        }
        if let Some(retry) = self.retry {
            buf.push_str(&format!("retry: {}\n", retry));
        }
        for line in self.data.lines() {
            buf.push_str(&format!("data: {}\n", line));
        }
        buf.push('\n');
        buf
    }
}

pub struct SseStream {
    rx: UnboundedReceiver<SseEvent>,
}

impl SseStream {
    pub fn new() -> (SseSender, Self) {
        let (tx, rx) = mpsc::unbounded_channel();
        (SseSender { tx }, Self { rx })
    }
}

impl Stream for SseStream {
    type Item = SseEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx).poll_recv(cx)
    }
}

#[derive(Clone)]
pub struct SseSender {
    tx: UnboundedSender<SseEvent>,
}

impl SseSender {
    pub fn send(&self, event: SseEvent) -> anyhow::Result<()> {
        self.tx.send(event)?;
        Ok(())
    }

    pub fn send_data(&self, data: impl Into<String>) -> anyhow::Result<()> {
        self.send(SseEvent::new(data))
    }

    pub fn send_event(&self, event: &str, data: impl Into<String>) -> anyhow::Result<()> {
        self.send(SseEvent::new(data).with_event(event))
    }
}

pub async fn parse_sse_stream(
    mut body: impl std::io::Read,
) -> anyhow::Result<Vec<SseEvent>> {
    let mut content = String::new();
    body.read_to_string(&mut content)?;

    let mut events = Vec::new();
    let mut current_event = String::new();
    let mut current_data = String::new();
    let mut current_id = Option::<String>::None;
    let mut current_retry = Option::<u64>::None;

    for line in content.lines() {
        if line.is_empty() {
            if !current_data.is_empty() || current_event.is_empty() {
                let mut event = SseEvent::new(current_data.clone());
                if !current_event.is_empty() {
                    event = event.with_event(current_event.clone());
                }
                if let Some(ref id) = current_id {
                    event = event.with_id(id.clone());
                }
                if let Some(retry) = current_retry {
                    event = event.with_retry(retry);
                }
                events.push(event);
            }
            current_event.clear();
            current_data.clear();
            current_id = None;
            current_retry = None;
            continue;
        }

        if let Some(value) = line.strip_prefix("id:") {
            current_id = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("event:") {
            current_event = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("data:") {
            if !current_data.is_empty() {
                current_data.push('\n');
            }
            current_data.push_str(value.trim());
        } else if let Some(value) = line.strip_prefix("retry:") {
            current_retry = value.trim().parse().ok();
        }
    }

    if !current_data.is_empty() || current_event.is_empty() {
        let mut event = SseEvent::new(current_data);
        if !current_event.is_empty() {
            event = event.with_event(current_event);
        }
        events.push(event);
    }

    Ok(events)
}
