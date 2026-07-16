use anyhow::{Context, Result};
use futures::StreamExt;
use lru::LruCache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    max_tokens: u32,
    stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Choice {
    message: ChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StreamChoice {
    delta: Delta,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Delta {
    content: Option<String>,
}

type CacheKey = String;
type CacheValue = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct AiEngine {
    client: Client,
    config: AiConfig,
    cache: Arc<RwLock<LruCache<CacheKey, CacheValue>>>,
    system_prompt: Option<String>,
}

impl AiEngine {
    pub fn new(config: AiConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        let cache = Arc::new(RwLock::new(LruCache::new(
            NonZeroUsize::new(256).unwrap(),
        )));

        Self {
            client,
            config,
            cache,
            system_prompt: None,
        }
    }

    pub fn provider(&self) -> LlmProvider {
        // Best-effort reconstruction from config
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
        } else if self.config.endpoint.contains("localhost") || self.config.endpoint.contains("127.0.0.1") {
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

    pub fn model(&self) -> &str { &self.config.model }
    pub fn system_prompt(&self) -> Option<&str> { self.system_prompt.as_deref() }

    pub fn from_provider(provider: LlmProvider) -> Self {
        Self::new(provider.to_config())
    }

    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.to_string());
        self
    }

    pub fn with_config(mut self, config: AiConfig) -> Self {
        self.config = config;
        self
    }

    pub fn config(&self) -> &AiConfig {
        &self.config
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

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.endpoint))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send AI request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("AI API error ({}): {}", status, text));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse AI response")?;

        let result = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        self.set_cache(cache_key, result.clone()).await;
        Ok(result)
    }

    async fn chat_stream(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<String>> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.endpoint))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send streaming AI request")?;

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let mut stream = response.bytes_stream();

        tokio::spawn(async move {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
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
    fn test_cache_key_generation() {
        let key = AiEngine::cache_key("system", "user prompt");
        assert_eq!(key.len(), 64);
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
    fn test_ai_config_model() {
        let config = AiConfig {
            model: "gpt-3.5-turbo".into(),
            ..Default::default()
        };
        assert_eq!(config.model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_ai_config_endpoint() {
        let config = AiConfig {
            endpoint: "https://custom.api.com/v1".into(),
            ..Default::default()
        };
        assert_eq!(config.endpoint, "https://custom.api.com/v1");
    }

    #[test]
    fn test_cache_empty_on_new() {
        let config = AiConfig::default();
        let engine = AiEngine::new(config);
        assert_eq!(engine.cache_size(), 0);
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
    fn test_llm_provider_anthropic() {
        let provider = LlmProvider::anthropic_sonnet("sk-ant-test".into());
        let config = provider.to_config();
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert!(config.endpoint.contains("anthropic"));
    }

    #[test]
    fn test_ai_engine_from_provider() {
        let provider = LlmProvider::openai_gpt4("test-key".into());
        let engine = AiEngine::from_provider(provider);
        assert_eq!(engine.model(), "gpt-4");
    }

    #[test]
    fn test_ai_engine_with_system_prompt() {
        let provider = LlmProvider::openai_gpt4("test".into());
        let engine = AiEngine::from_provider(provider).with_system_prompt("Be helpful");
        assert_eq!(engine.system_prompt(), Some("Be helpful"));
    }

    #[test]
    fn test_cache_clear() {
        let config = AiConfig::default();
        let engine = AiEngine::new(config);
        engine.clear_cache();
        assert_eq!(engine.cache_size(), 0);
    }
}
