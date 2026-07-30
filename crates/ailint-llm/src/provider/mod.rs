//! LLM provider abstraction and shared request/response types.

pub mod anthropic;
pub mod google;
pub mod ollama;
pub mod openai;

use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use anthropic::AnthropicProvider;
pub use google::GoogleProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;

/// Supported LLM providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Openai,
    Anthropic,
    Google,
    Ollama,
    /// Any OpenAI-compatible endpoint.
    Compatible,
}

/// Response shape requested from the provider.
#[derive(Debug, Clone)]
pub enum ResponseFormat {
    Text,
    JsonSchema {
        schema: serde_json::Value,
        name: String,
    },
}

/// A chat request sent to a provider.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub system: Option<String>,
    pub user: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub response_format: ResponseFormat,
    pub timeout: Duration,
}

impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            model: String::new(),
            system: None,
            user: String::new(),
            temperature: Some(0.2),
            max_tokens: Some(1024),
            response_format: ResponseFormat::Text,
            timeout: Duration::from_secs(60),
        }
    }
}

/// A chat response returned by a provider.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub text: String,
    pub usage: Usage,
    pub finish_reason: Option<String>,
    pub model: String,
}

/// Token usage reported by the provider (zeroes if unavailable).
#[derive(Debug, Clone, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// Trait implemented by every LLM provider.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn kind(&self) -> ProviderKind;
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse>;
}
