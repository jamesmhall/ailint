//! Ollama provider (local models).
//!
//! Ollama exposes an OpenAI-compatible endpoint at `/v1` and its native API at
//! `/api/chat`. TODO: pick one (probably the native API) and implement.

use anyhow::{bail, Result};

use crate::provider::{ChatRequest, ChatResponse, LlmProvider, ProviderKind};

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    pub base_url: String,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".into(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmProvider for OllamaProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Ollama
    }

    fn chat(&self, _req: &ChatRequest) -> Result<ChatResponse> {
        // TODO: real HTTP call to /api/chat.
        bail!("ollama provider not yet implemented");
    }
}
