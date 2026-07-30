//! Anthropic (Claude) provider.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::provider::{
    ChatRequest, ChatResponse, LlmProvider, ProviderKind, ResponseFormat, Usage,
};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Read the API key from `ANTHROPIC_API_KEY` or `AILINT_LLM_API_KEY`.
    pub fn from_env() -> Result<Self> {
        let key = std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| std::env::var("AILINT_LLM_API_KEY"))
            .map_err(|_| anyhow!("ANTHROPIC_API_KEY or AILINT_LLM_API_KEY must be set"))?;
        Ok(Self::new(key))
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/messages", self.base_url.trim_end_matches('/'));

        // Anthropic has no native JSON-schema forcing; ask for JSON in the system prompt.
        let system = match (&req.response_format, req.system.as_deref()) {
            (ResponseFormat::JsonSchema { schema, .. }, existing) => {
                let extra = format!(
                    "Respond with ONLY a JSON object matching this schema (no prose, no code fences): {}",
                    schema
                );
                match existing {
                    Some(s) if !s.is_empty() => Some(format!("{s}\n\n{extra}")),
                    _ => Some(extra),
                }
            }
            (ResponseFormat::Text, existing) => existing.map(str::to_string),
        };

        let mut body = json!({
            "model": req.model,
            "messages": [{"role": "user", "content": req.user}],
            "max_tokens": req.max_tokens.unwrap_or(1024),
        });
        if let Some(s) = system {
            body["system"] = json!(s);
        }
        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }

        tracing::debug!(target: "ailint_llm::anthropic", url = %url, model = %req.model, "anthropic chat request");

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .timeout(req.timeout)
            .json(&body)
            .send()
            .await
            .context("anthropic request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!(target: "ailint_llm::anthropic", %status, "anthropic non-200 response");
            return Err(anyhow!("anthropic returned {}: {}", status, text));
        }

        let body: Value = resp.json().await.context("anthropic response not JSON")?;
        parse_response(body, &req.model)
    }
}

fn parse_response(body: Value, fallback_model: &str) -> Result<ChatResponse> {
    let text = body
        .get("content")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .and_then(|c| c.get("text"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("anthropic response missing content[0].text"))?
        .to_string();

    let finish_reason = body
        .get("stop_reason")
        .and_then(Value::as_str)
        .map(str::to_string);

    let usage = body
        .get("usage")
        .map(|u| Usage {
            prompt_tokens: u.get("input_tokens").and_then(Value::as_u64).unwrap_or(0) as u32,
            completion_tokens: u.get("output_tokens").and_then(Value::as_u64).unwrap_or(0) as u32,
        })
        .unwrap_or_default();

    let model = body
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(fallback_model)
        .to_string();

    Ok(ChatResponse {
        text,
        usage,
        finish_reason,
        model,
    })
}
