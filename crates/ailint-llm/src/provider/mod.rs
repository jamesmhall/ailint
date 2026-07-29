//! LLM provider abstraction.

pub mod anthropic;
pub mod google;
pub mod ollama;
pub mod openai;

use anyhow::Result;

/// Supported LLM providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Openai,
    Anthropic,
    Google,
    Ollama,
    /// Any OpenAI-compatible endpoint.
    Compatible,
}

/// A minimal LLM chat request. TODO: expand once we know what the analyzer needs.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub system: Option<String>,
    pub user: String,
    // TODO: temperature, max_tokens, response_format (json_schema), timeout.
}

/// A minimal chat response.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub text: String,
    // TODO: usage (prompt/completion tokens), finish_reason, model.
}

/// Trait implemented by every provider. Currently `Send + Sync` only —
/// TODO: switch to `#[async_trait::async_trait]` and add an async `chat`
/// method once we pull in the `async-trait` crate.
pub trait LlmProvider: Send + Sync {
    fn kind(&self) -> ProviderKind;

    /// Blocking placeholder. TODO: replace with async.
    fn chat(&self, req: &ChatRequest) -> Result<ChatResponse>;
}
