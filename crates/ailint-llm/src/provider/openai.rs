//! OpenAI provider (and OpenAI-compatible endpoints).
//!
//! TODO: implement `/v1/chat/completions` request against configurable base URL.
//! Read API key from `AILINT_LLM_API_KEY` or `OPENAI_API_KEY`.

use anyhow::{bail, Result};

use crate::provider::{ChatRequest, ChatResponse, LlmProvider, ProviderKind};

#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    pub api_key: String,
    pub base_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".into(),
        }
    }

    /// Point at any OpenAI-compatible endpoint (e.g. LM Studio, vLLM, LiteLLM).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

impl LlmProvider for OpenAiProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Openai
    }

    fn chat(&self, _req: &ChatRequest) -> Result<ChatResponse> {
        // TODO: real HTTP call.
        bail!("openai provider not yet implemented");
    }
}
