//! Ollama provider (local models).

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::provider::{
    ChatRequest, ChatResponse, LlmProvider, ProviderKind, ResponseFormat, Usage,
};

const DEFAULT_BASE_URL: &str = "http://localhost:11434";

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.into(),
            client: reqwest::Client::new(),
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

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Ollama
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let mut messages = Vec::with_capacity(2);
        if let Some(system) = &req.system {
            messages.push(json!({"role": "system", "content": system}));
        }
        messages.push(json!({"role": "user", "content": req.user}));

        let mut options = json!({});
        if let Some(t) = req.temperature {
            options["temperature"] = json!(t);
        }
        if let Some(m) = req.max_tokens {
            options["num_predict"] = json!(m);
        }

        let mut body = json!({
            "model": req.model,
            "messages": messages,
            "stream": false,
            "options": options,
        });
        if matches!(req.response_format, ResponseFormat::JsonSchema { .. }) {
            body["format"] = json!("json");
        }

        tracing::debug!(target: "ailint_llm::ollama", url = %url, model = %req.model, "ollama chat request");

        let resp = self
            .client
            .post(&url)
            .timeout(req.timeout)
            .json(&body)
            .send()
            .await
            .context("ollama request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!(target: "ailint_llm::ollama", %status, "ollama non-200 response");
            return Err(anyhow!("ollama returned {}: {}", status, text));
        }

        let body: Value = resp.json().await.context("ollama response not JSON")?;
        parse_response(body, &req.model)
    }
}

fn parse_response(body: Value, fallback_model: &str) -> Result<ChatResponse> {
    let text = body
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("ollama response missing message.content"))?
        .to_string();

    let finish_reason = body
        .get("done_reason")
        .and_then(Value::as_str)
        .map(str::to_string);

    let usage = Usage {
        prompt_tokens: body
            .get("prompt_eval_count")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        completion_tokens: body.get("eval_count").and_then(Value::as_u64).unwrap_or(0) as u32,
    };

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
