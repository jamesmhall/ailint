//! Anthropic (Claude) provider.
//!
//! TODO: implement `/v1/messages` request. Read API key from
//! `AILINT_LLM_API_KEY` or `ANTHROPIC_API_KEY`. Anthropic uses a distinct
//! `x-api-key` header and `anthropic-version` header.

use anyhow::{bail, Result};

use crate::provider::{ChatRequest, ChatResponse, LlmProvider, ProviderKind};

#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    pub api_key: String,
    pub base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com/v1".into(),
        }
    }
}

impl LlmProvider for AnthropicProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    fn chat(&self, _req: &ChatRequest) -> Result<ChatResponse> {
        // TODO: real HTTP call.
        bail!("anthropic provider not yet implemented");
    }
}
