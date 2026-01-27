//! Ollama integration client
//!
//! Provides native Rust client for Ollama with support for chat completion,
//! streaming, and model management.

use crate::error::{AbcdodafError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ollama client for LLM inference
pub struct OllamaClient {
    config: OllamaConfig,
    http_client: reqwest::Client,
}

/// Ollama client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama server URL
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Default model
    pub default_model: String,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name
    pub name: String,
    /// Model size in bytes
    pub size: u64,
    /// Model family (llama, mistral, etc.)
    pub family: Option<String>,
    /// Parameter count
    pub parameter_size: Option<String>,
    /// Quantization level
    pub quantization: Option<String>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role: system, user, or assistant
    pub role: String,
    /// Message content
    pub content: String,
    /// Optional images (base64 encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

/// Chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model name
    pub model: String,
    /// Conversation messages
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Top-k sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Enable streaming
    #[serde(default)]
    pub stream: bool,
    /// Additional options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Model used
    pub model: String,
    /// Generated message
    pub message: ChatMessage,
    /// Response done flag
    pub done: bool,
    /// Token usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<i32>,
    /// Generation time in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
}

/// Streaming chunk from Ollama
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Model used
    pub model: String,
    /// Message delta
    pub message: ChatMessage,
    /// Done flag
    pub done: bool,
}

/// List models response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListModelsResponse {
    /// Available models
    pub models: Vec<OllamaModelInfo>,
}

/// Model info from Ollama API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelInfo {
    /// Model name
    pub name: String,
    /// Model ID/digest
    pub digest: String,
    /// Size in bytes
    pub size: u64,
    /// Modified timestamp
    pub modified_at: String,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(config: OllamaConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| {
                AbcdodafError::IntegrationError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { config, http_client })
    }

    /// Create client with default configuration
    pub fn default() -> Result<Self> {
        Self::new(OllamaConfig::default())
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.config.base_url);

        let response = self.http_client.get(&url).send().await.map_err(|e| {
            AbcdodafError::IntegrationError(format!("Failed to list models: {}", e))
        })?;

        let list: ListModelsResponse = response.json().await.map_err(|e| {
            AbcdodafError::IntegrationError(format!("Failed to parse response: {}", e))
        })?;

        Ok(list
            .models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name,
                size: m.size,
                family: None,
                parameter_size: None,
                quantization: None,
            })
            .collect())
    }

    /// Send a chat completion request
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.config.base_url);

        let response =
            self.http_client.post(&url).json(&request).send().await.map_err(|e| {
                AbcdodafError::IntegrationError(format!("Chat request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AbcdodafError::IntegrationError(format!(
                "Chat request failed with status {}: {}",
                status, error_text
            )));
        }

        response.json().await.map_err(|e| {
            AbcdodafError::IntegrationError(format!("Failed to parse response: {}", e))
        })
    }

    /// Send a streaming chat completion request
    pub async fn chat_stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamChunk>>> {
        request.stream = true;
        let url = format!("{}/api/chat", self.config.base_url);

        let response = self.http_client.post(&url).json(&request).send().await.map_err(|e| {
            AbcdodafError::IntegrationError(format!("Stream request failed: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AbcdodafError::IntegrationError(format!(
                "Stream request failed with status {}: {}",
                status, error_text
            )));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            use futures::StreamExt;

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);

                        // Process complete lines
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].to_string();
                            buffer = buffer[newline_pos + 1..].to_string();

                            if line.is_empty() {
                                continue;
                            }

                            match serde_json::from_str::<StreamChunk>(&line) {
                                Ok(chunk) => {
                                    let done = chunk.done;
                                    if tx.send(Ok(chunk)).await.is_err() {
                                        return;
                                    }
                                    if done {
                                        return;
                                    }
                                },
                                Err(e) => {
                                    let _ = tx
                                        .send(Err(AbcdodafError::IntegrationError(format!(
                                            "Failed to parse stream chunk: {}",
                                            e
                                        ))))
                                        .await;
                                    return;
                                },
                            }
                        }
                    },
                    Err(e) => {
                        let _ = tx
                            .send(Err(AbcdodafError::IntegrationError(format!(
                                "Stream error: {}",
                                e
                            ))))
                            .await;
                        return;
                    },
                }
            }
        });

        Ok(rx)
    }

    /// Generate a simple completion
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
    ) -> Result<String> {
        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                images: None,
            }],
            temperature,
            top_p: None,
            top_k: None,
            num_predict: None,
            stop: None,
            stream: false,
            options: None,
        };

        let response = self.chat(request).await?;
        Ok(response.message.content)
    }

    /// Pull a model from Ollama registry
    pub async fn pull_model(&self, model_name: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.config.base_url);

        let payload = serde_json::json!({
            "name": model_name,
        });

        let response =
            self.http_client.post(&url).json(&payload).send().await.map_err(|e| {
                AbcdodafError::IntegrationError(format!("Failed to pull model: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(AbcdodafError::IntegrationError(format!(
                "Model pull failed with status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Check if a model exists
    pub async fn model_exists(&self, model_name: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m.name == model_name))
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Get the default model
    pub fn default_model(&self) -> &str {
        &self.config.default_model
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout_secs: 300,
            default_model: "llama3.2:3b".to_string(),
        }
    }
}

impl ChatMessage {
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into(), images: None }
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into(), images: None }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".to_string(), content: content.into(), images: None }
    }
}

impl ChatRequest {
    /// Create a new chat request
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: vec![],
            temperature: None,
            top_p: None,
            top_k: None,
            num_predict: None,
            stop: None,
            stream: false,
            options: None,
        }
    }

    /// Add a message
    pub fn add_message(mut self, message: ChatMessage) -> Self {
        self.messages.push(message);
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: i32) -> Self {
        self.num_predict = Some(max_tokens);
        self
    }

    /// Enable streaming
    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

/// Trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate a completion
    async fn generate(&self, prompt: &str) -> Result<String>;

    /// Generate a chat completion
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String>;

    /// List available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
}

#[async_trait]
impl LlmProvider for OllamaClient {
    async fn generate(&self, prompt: &str) -> Result<String> {
        self.generate(&self.config.default_model, prompt, None).await
    }

    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        let request = ChatRequest {
            model: self.config.default_model.clone(),
            messages,
            temperature: Some(0.7),
            top_p: None,
            top_k: None,
            num_predict: None,
            stop: None,
            stream: false,
            options: None,
        };

        let response = self.chat(request).await?;
        Ok(response.message.content)
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        self.list_models().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_config_default() {
        let config = OllamaConfig::default();
        assert_eq!(config.base_url, "http://localhost:11434");
        assert_eq!(config.default_model, "llama3.2:3b");
    }

    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage::user("Hello, world!");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello, world!");
    }

    #[test]
    fn test_chat_request_builder() {
        let request = ChatRequest::new("llama3.2:3b")
            .add_message(ChatMessage::system("You are helpful"))
            .add_message(ChatMessage::user("Hello"))
            .with_temperature(0.8)
            .with_max_tokens(100);

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.temperature, Some(0.8));
        assert_eq!(request.num_predict, Some(100));
    }
}
