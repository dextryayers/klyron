use crate::client::{AiClient, Usage};
use anyhow::{Context, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: AssistantMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub model: Option<String>,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<StreamToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamToolCall {
    pub index: u32,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    pub function: Option<StreamToolCallFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamToolCallFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub model: Option<String>,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
}

impl ChatRequest {
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            top_p: None,
            stream: None,
            tools: None,
            tool_choice: None,
        }
    }
}

pub struct ChatCompletion {
    client: AiClient,
}

impl ChatCompletion {
    pub fn new(client: AiClient) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: ChatRequest) -> Result<ChatResponse> {
        let mut req = request;
        req.stream = Some(false);

        let response = self
            .client
            .post("/chat/completions", &req)
            .send()
            .await
            .context("Failed to send chat completion request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Chat API error ({}): {}", status, text));
        }

        response.json().await.context("Failed to parse chat response")
    }

    pub async fn create_stream(
        &self,
        request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<String>> {
        let mut req = request;
        req.stream = Some(true);

        let response = self
            .client
            .post("/chat/completions", &req)
            .send()
            .await
            .context("Failed to send streaming request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Stream API error ({}): {}", status, text));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let mut stream = response.bytes_stream();

        tokio::spawn(async move {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data.trim() == "[DONE]" {
                                    break;
                                }
                                if let Ok(chunk) =
                                    serde_json::from_str::<StreamChunk>(data)
                                {
                                    if let Some(choice) = chunk.choices.first() {
                                        if let Some(content) = &choice.delta.content {
                                            let _ = tx.send(content.clone()).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Stream error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    pub async fn create_stream_with_tools(
        &self,
        request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>> {
        let mut req = request;
        req.stream = Some(true);

        let response = self
            .client
            .post("/chat/completions", &req)
            .send()
            .await
            .context("Failed to send streaming request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Stream API error ({}): {}", status, text));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let mut stream = response.bytes_stream();

        tokio::spawn(async move {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data.trim() == "[DONE]" {
                                    let _ = tx.send(StreamEvent::Done).await;
                                    break;
                                }
                                if let Ok(chunk) =
                                    serde_json::from_str::<StreamChunk>(data)
                                {
                                    if let Some(choice) = chunk.choices.first() {
                                        if let Some(content) = &choice.delta.content {
                                            let _ = tx
                                                .send(StreamEvent::Content(
                                                    content.clone(),
                                                ))
                                                .await;
                                        }
                                        if let Some(tool_calls) =
                                            &choice.delta.tool_calls
                                        {
                                            for tc in tool_calls {
                                                if let Some(id) = &tc.id {
                                                    if let Some(name) = tc
                                                        .function
                                                        .as_ref()
                                                        .and_then(|f| f.name.as_ref())
                                                    {
                                                        let _ = tx
                                                            .send(
                                                                StreamEvent::ToolCallBegin {
                                                                    id: id.clone(),
                                                                    name: name.clone(),
                                                                },
                                                            )
                                                            .await;
                                                    }
                                                }
                                                if let Some(args) = tc
                                                    .function
                                                    .as_ref()
                                                    .and_then(|f| {
                                                        f.arguments.as_ref()
                                                    })
                                                {
                                                    let _ = tx
                                                        .send(
                                                            StreamEvent::ToolCallArguments(
                                                                args.clone(),
                                                            ),
                                                        )
                                                        .await;
                                                }
                                            }
                                        }
                                        if let Some(reason) =
                                            &choice.finish_reason
                                        {
                                            let _ = tx
                                                .send(StreamEvent::Finish(
                                                    reason.clone(),
                                                ))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Content(String),
    ToolCallBegin { id: String, name: String },
    ToolCallArguments(String),
    Finish(String),
    Error(String),
    Done,
}

impl fmt::Display for StreamEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamEvent::Content(c) => write!(f, "{}", c),
            StreamEvent::ToolCallBegin { id, name } => {
                write!(f, "Tool call: {} ({})", name, id)
            }
            StreamEvent::ToolCallArguments(args) => write!(f, "{}", args),
            StreamEvent::Finish(reason) => write!(f, "[finish: {}]", reason),
            StreamEvent::Error(e) => write!(f, "[error: {}]", e),
            StreamEvent::Done => write!(f, "[done]"),
        }
    }
}
