//! Google (Gemini) provider.
//!
//! TODO: implement Gemini's `generateContent` endpoint. Read API key from
//! `AILINT_LLM_API_KEY` or `GEMINI_API_KEY` / `GOOGLE_API_KEY`.

use anyhow::{bail, Result};

use crate::provider::{ChatRequest, ChatResponse, LlmProvider, ProviderKind};

#[derive(Debug, Clone)]
pub struct GoogleProvider {
    pub api_key: String,
    pub base_url: String,
}

impl GoogleProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".into(),
        }
    }
}

impl LlmProvider for GoogleProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Google
    }

    fn chat(&self, _req: &ChatRequest) -> Result<ChatResponse> {
        // TODO: real HTTP call.
        bail!("google provider not yet implemented");
    }
}
