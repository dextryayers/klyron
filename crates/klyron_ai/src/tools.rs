pub use crate::chat::{ToolDefinition, ToolFunction};
use crate::chat::{ChatMessage, ToolCall};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub role: String,
    pub tool_call_id: String,
    pub content: String,
}

impl ToolResult {
    pub fn new(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".into(),
            tool_call_id: tool_call_id.into(),
            content: content.into(),
        }
    }
}

pub struct ToolRegistry {
    tools: Vec<ToolDefinition>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register(&mut self, definition: ToolDefinition) {
        self.tools.push(definition);
    }

    pub fn register_fn(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
    ) {
        self.tools.push(ToolDefinition {
            tool_type: "function".into(),
            function: ToolFunction {
                name: name.into(),
                description: description.into(),
                parameters,
            },
        });
    }

    pub fn definitions(&self) -> &[ToolDefinition] {
        &self.tools
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for ToolRegistry {
    type Item = ToolDefinition;
    type IntoIter = std::vec::IntoIter<ToolDefinition>;

    fn into_iter(self) -> Self::IntoIter {
        self.tools.into_iter()
    }
}

pub fn tool_result_message(_tool_call_id: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: "tool".into(),
        content: content.to_string(),
    }
}

pub fn build_tool_response_messages(
    original_messages: Vec<ChatMessage>,
    tool_calls: &[ToolCall],
    tool_results: Vec<ToolResult>,
) -> Vec<ChatMessage> {
    let mut messages = original_messages;

    let assistant_content: Vec<String> = tool_calls
        .iter()
        .map(|tc| {
            format!(
                r#"{{"function":"{}","args":{}}}"#,
                tc.function.name, tc.function.arguments
            )
        })
        .collect();

    messages.push(ChatMessage {
        role: "assistant".into(),
        content: assistant_content.join("\n"),
    });

    for result in tool_results {
        messages.push(ChatMessage {
            role: "tool".into(),
            content: result.content,
        });
    }

    messages
}
