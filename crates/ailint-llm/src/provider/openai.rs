//! OpenAI provider (and OpenAI-compatible endpoints).

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::provider::{
    ChatRequest, ChatResponse, LlmProvider, ProviderKind, ResponseFormat, Usage,
};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Point at any OpenAI-compatible endpoint (e.g. LM Studio, vLLM, LiteLLM).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Read the API key from `OPENAI_API_KEY` or `AILINT_LLM_API_KEY`.
    pub fn from_env() -> Result<Self> {
        let key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("AILINT_LLM_API_KEY"))
            .map_err(|_| anyhow!("OPENAI_API_KEY or AILINT_LLM_API_KEY must be set"))?;
        Ok(Self::new(key))
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Openai
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut messages = Vec::with_capacity(2);
        if let Some(system) = &req.system {
            messages.push(json!({"role": "system", "content": system}));
        }
        messages.push(json!({"role": "user", "content": req.user}));

        let mut body = json!({
            "model": req.model,
            "messages": messages,
        });
        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }
        if let Some(m) = req.max_tokens {
            body["max_tokens"] = json!(m);
        }
        if let ResponseFormat::JsonSchema { schema, name } = &req.response_format {
            body["response_format"] = json!({
                "type": "json_schema",
                "json_schema": {"name": name, "schema": schema, "strict": true},
            });
        }

        tracing::debug!(target: "ailint_llm::openai", url = %url, model = %req.model, "openai chat request");

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .timeout(req.timeout)
            .json(&body)
            .send()
            .await
            .context("openai request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!(target: "ailint_llm::openai", %status, "openai non-200 response");
            return Err(anyhow!("openai returned {}: {}", status, text));
        }

        let body: Value = resp.json().await.context("openai response not JSON")?;
        parse_response(body, &req.model)
    }
}

fn parse_response(body: Value, fallback_model: &str) -> Result<ChatResponse> {
    let text = body
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("openai response missing choices[0].message.content"))?
        .to_string();

    let finish_reason = body
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .and_then(|c| c.get("finish_reason"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let usage = body
        .get("usage")
        .map(|u| Usage {
            prompt_tokens: u.get("prompt_tokens").and_then(Value::as_u64).unwrap_or(0) as u32,
            completion_tokens: u
                .get("completion_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32,
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
