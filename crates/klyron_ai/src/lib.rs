pub mod chat;
pub mod client;
pub mod completion;
pub mod embeddings;
pub mod tools;

use anyhow::Result;
use lru::LruCache;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

pub use chat::{
    ChatCompletion, ChatMessage, ChatRequest, ChatResponse, StreamEvent,
};
pub use client::{AiClient, Usage};
pub use completion::{Completion, CompletionRequest, CompletionResponse};
pub use embeddings::{EmbeddingRequest, EmbeddingResponse, Embeddings};
pub use tools::{ToolDefinition, ToolFunction, ToolRegistry, ToolResult};

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub api_key: String,
    pub endpoint: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            endpoint: "https://api.openai.com/v1".to_string(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LlmProvider {
    OpenAI { api_key: String, model: String },
    Anthropic { api_key: String, model: String },
    Google { api_key: String, model: String },
    Ollama { endpoint: String, model: String },
    Custom { endpoint: String, api_key: Option<String> },
}

impl LlmProvider {
    pub fn openai_gpt4(api_key: String) -> Self {
        Self::OpenAI { api_key, model: "gpt-4".into() }
    }
    pub fn anthropic_sonnet(api_key: String) -> Self {
        Self::Anthropic { api_key, model: "claude-sonnet-4-20250514".into() }
    }
    pub fn to_config(&self) -> AiConfig {
        match self {
            Self::OpenAI { api_key, model } => AiConfig {
                api_key: api_key.clone(),
                endpoint: "https://api.openai.com/v1".into(),
                model: model.clone(),
                temperature: 0.7,
                max_tokens: 4096,
            },
            Self::Anthropic { api_key, model } => AiConfig {
                api_key: api_key.clone(),
                endpoint: "https://api.anthropic.com/v1".into(),
                model: model.clone(),
                temperature: 0.7,
                max_tokens: 4096,
            },
            Self::Google { api_key, model } => AiConfig {
                api_key: api_key.clone(),
                endpoint: "https://generativelanguage.googleapis.com/v1".into(),
                model: model.clone(),
                temperature: 0.7,
                max_tokens: 4096,
            },
            Self::Ollama { endpoint, model } => AiConfig {
                api_key: String::new(),
                endpoint: endpoint.clone(),
                model: model.clone(),
                temperature: 0.7,
                max_tokens: 4096,
            },
            Self::Custom { endpoint, api_key } => AiConfig {
                api_key: api_key.clone().unwrap_or_default(),
                endpoint: endpoint.clone(),
                model: "custom".into(),
                temperature: 0.7,
                max_tokens: 4096,
            },
        }
    }
}

type CacheKey = String;
type CacheValue = String;

pub struct AiEngine {
    client: AiClient,
    config: AiConfig,
    cache: Arc<RwLock<LruCache<CacheKey, CacheValue>>>,
    system_prompt: Option<String>,
}

impl AiEngine {
    pub fn new(config: AiConfig) -> Self {
        let client = AiClient::new(&config.endpoint, &config.api_key);
        let cache = Arc::new(RwLock::new(LruCache::new(
            NonZeroUsize::new(256).unwrap(),
        )));
        Self { client, config, cache, system_prompt: None }
    }

    pub fn from_provider(provider: LlmProvider) -> Self {
        Self::new(provider.to_config())
    }

    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.to_string());
        self
    }

    pub fn with_config(mut self, config: AiConfig) -> Self {
        let endpoint = config.endpoint.clone();
        let api_key = config.api_key.clone();
        self.config = config;
        self.client = AiClient::new(&endpoint, &api_key);
        self
    }

    pub fn client(&self) -> &AiClient {
        &self.client
    }

    pub fn config(&self) -> &AiConfig {
        &self.config
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    pub fn system_prompt(&self) -> Option<&str> {
        self.system_prompt.as_deref()
    }

    pub fn provider(&self) -> LlmProvider {
        if self.config.endpoint.contains("openai") {
            LlmProvider::OpenAI {
                api_key: self.config.api_key.clone(),
                model: self.config.model.clone(),
            }
        } else if self.config.endpoint.contains("anthropic") {
            LlmProvider::Anthropic {
                api_key: self.config.api_key.clone(),
                model: self.config.model.clone(),
            }
        } else if self.config.endpoint.contains("googleapis") {
            LlmProvider::Google {
                api_key: self.config.api_key.clone(),
                model: self.config.model.clone(),
            }
        } else if self.config.endpoint.contains("localhost")
            || self.config.endpoint.contains("127.0.0.1")
        {
            LlmProvider::Ollama {
                endpoint: self.config.endpoint.clone(),
                model: self.config.model.clone(),
            }
        } else {
            LlmProvider::Custom {
                endpoint: self.config.endpoint.clone(),
                api_key: Some(self.config.api_key.clone()),
            }
        }
    }

    fn cache_key(prefix: &str, content: &str) -> CacheKey {
        let mut hasher = Sha256::new();
        hasher.update(prefix.as_bytes());
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    async fn check_cache(&self, key: &CacheKey) -> Option<String> {
        let mut cache = self.cache.write().await;
        cache.get(key).cloned()
    }

    async fn set_cache(&self, key: CacheKey, value: String) {
        let mut cache = self.cache.write().await;
        cache.put(key, value);
    }

    async fn chat_completion(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let cache_key = Self::cache_key(system_prompt, user_prompt);

        if let Some(cached) = self.check_cache(&cache_key).await {
            debug!("AI response cache hit");
            return Ok(cached);
        }

        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_prompt),
        ];

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            top_p: None,
            stream: Some(false),
            tools: None,
            tool_choice: None,
        };

        let chat = ChatCompletion::new(self.client.clone());
        let response = chat.create(request).await?;

        let result = response
            .choices
            .first()
            .map(|c| c.message.content.clone().unwrap_or_default())
            .unwrap_or_default();

        self.set_cache(cache_key, result.clone()).await;
        Ok(result)
    }

    async fn chat_stream(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<String>> {
        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_prompt),
        ];

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            top_p: None,
            stream: Some(true),
            tools: None,
            tool_choice: None,
        };

        let chat = ChatCompletion::new(self.client.clone());
        chat.create_stream(request).await
    }

    pub async fn generate_code(&self, prompt: &str, lang: &str) -> Result<String> {
        let system = format!(
            "You are a code generation assistant. Generate clean, idiomatic {} code. \
             Return ONLY the code without explanation, wrapped in a code block.",
            lang
        );
        self.chat_completion(&system, prompt).await
    }

    pub async fn review_code(&self, code: &str, lang: &str) -> Result<String> {
        let system = format!(
            "You are a code review expert for {}. Analyze the code for:\n\
             - Security vulnerabilities\n\
             - Performance issues\n\
             - Code style and best practices\n\
             - Potential bugs\n\
             Provide specific, actionable feedback.",
            lang
        );
        self.chat_completion(&system, code).await
    }

    pub async fn optimize_code(&self, code: &str) -> Result<String> {
        let system = "\
            You are a code optimization expert. Analyze the code and provide:\n\
            - Performance improvements\n\
            - Memory optimizations\n\
            - Algorithmic improvements\n\
            Return the optimized code with a brief explanation of changes."
            .to_string();
        self.chat_completion(&system, code).await
    }

    pub async fn generate_docs(&self, code: &str, lang: &str) -> Result<String> {
        let system = format!(
            "You are a documentation expert for {}. Generate comprehensive documentation including:\n\
             - Description of what the code does\n\
             - Function signatures and parameters\n\
             - Return values\n\
             - Usage examples\n\
             Use the standard documentation format for {}.",
            lang, lang
        );
        self.chat_completion(&system, code).await
    }

    pub async fn generate_tests(&self, code: &str, lang: &str) -> Result<String> {
        let system = format!(
            "You are a testing expert for {}. Generate comprehensive unit tests including:\n\
             - Test cases for normal inputs\n\
             - Edge cases\n\
             - Error handling tests\n\
             Follow {} testing conventions and best practices.\n\
             Return ONLY the test code without explanation.",
            lang, lang
        );
        self.chat_completion(&system, code).await
    }

    pub async fn migrate_code(&self, code: &str, from: &str, to: &str) -> Result<String> {
        let system = format!(
            "You are a code migration expert. Migrate the following {} code to {}.\n\
             Preserve the original logic and behavior.\n\
             Use idiomatic {} patterns and conventions.\n\
             Return ONLY the migrated code without explanation.",
            from, to, to
        );
        self.chat_completion(&system, code).await
    }

    pub async fn commit_message(&self, diff: &str) -> Result<String> {
        let system = "\
            You are a commit message generator following Conventional Commits.\n\
            Analyze the git diff and generate a concise, descriptive commit message.\n\
            Format: <type>(<scope>): <description>\n\n\
            <body>\n\n\
            <footer>"
            .to_string();
        self.chat_completion(&system, diff).await
    }

    pub async fn suggest_fix(&self, error: &str, code: &str) -> Result<String> {
        let system = "\
            You are a debugging expert. Given a compile error and the relevant code:\n\
            1. Explain the error in simple terms\n\
            2. Provide the corrected code\n\
            3. Explain why the fix works\n\
            Return the fix wrapped in a code block."
            .to_string();
        let prompt = format!("Error: {error}\n\nCode:\n{code}");
        self.chat_completion(&system, &prompt).await
    }

    pub async fn generate_code_stream(
        &self,
        prompt: &str,
        lang: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<String>> {
        let system = format!(
            "You are a code generation assistant. Generate clean, idiomatic {} code. \
             Return ONLY the code without explanation.",
            lang
        );
        self.chat_stream(&system, prompt).await
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.blocking_write();
        cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        let cache = self.cache.blocking_read();
        cache.len()
    }

    pub fn chat_completion_api(&self) -> ChatCompletion {
        ChatCompletion::new(self.client.clone())
    }

    pub fn embeddings_api(&self) -> Embeddings {
        Embeddings::new(self.client.clone())
    }

    pub fn completion_api(&self) -> Completion {
        Completion::new(self.client.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_defaults() {
        let config = AiConfig::default();
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.temperature as i32, 0);
        assert_eq!(config.max_tokens, 4096);
    }

    #[test]
    fn test_cache_key_deterministic() {
        let k1 = AiEngine::cache_key("test", "hello");
        let k2 = AiEngine::cache_key("test", "hello");
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cache_key_different_inputs() {
        let k1 = AiEngine::cache_key("a", "b");
        let k2 = AiEngine::cache_key("a", "c");
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_llm_provider_openai_config() {
        let provider = LlmProvider::openai_gpt4("sk-test".into());
        let config = provider.to_config();
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.api_key, "sk-test");
        assert!(config.endpoint.contains("openai"));
    }

    #[test]
    fn test_cache_empty_on_new() {
        let config = AiConfig::default();
        let engine = AiEngine::new(config);
        assert_eq!(engine.cache_size(), 0);
    }

    #[test]
    fn test_embeddings_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = Embeddings::cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);

        let c = vec![2.0, 0.0, 0.0];
        let sim2 = Embeddings::cosine_similarity(&a, &c);
        assert!((sim2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register_fn("get_weather", "Get weather for a location", serde_json::json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            }
        }));
        assert_eq!(registry.len(), 1);

        let defs = registry.definitions();
        assert_eq!(defs[0].function.name, "get_weather");
    }

    #[test]
    fn test_chat_message_constructors() {
        let msg = ChatMessage::system("be helpful");
        assert_eq!(msg.role, "system");
        assert_eq!(msg.content, "be helpful");

        let msg = ChatMessage::user("hello");
        assert_eq!(msg.role, "user");
    }

    #[test]
    fn test_tool_result() {
        let tr = ToolResult::new("call_123", "sunny");
        assert_eq!(tr.tool_call_id, "call_123");
        assert_eq!(tr.content, "sunny");
    }
}
