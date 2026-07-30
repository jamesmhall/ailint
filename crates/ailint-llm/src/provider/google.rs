//! Google (Gemini) provider.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::provider::{
    ChatRequest, ChatResponse, LlmProvider, ProviderKind, ResponseFormat, Usage,
};

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

#[derive(Debug, Clone)]
pub struct GoogleProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl GoogleProvider {
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

    /// Read the API key from `GOOGLE_API_KEY`, `GEMINI_API_KEY`, or `AILINT_LLM_API_KEY`.
    pub fn from_env() -> Result<Self> {
        let key = std::env::var("GOOGLE_API_KEY")
            .or_else(|_| std::env::var("GEMINI_API_KEY"))
            .or_else(|_| std::env::var("AILINT_LLM_API_KEY"))
            .map_err(|_| {
                anyhow!("GOOGLE_API_KEY, GEMINI_API_KEY, or AILINT_LLM_API_KEY must be set")
            })?;
        Ok(Self::new(key))
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Google
    }

    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url.trim_end_matches('/'),
            req.model,
            self.api_key,
        );

        let mut gen_cfg = json!({});
        if let Some(t) = req.temperature {
            gen_cfg["temperature"] = json!(t);
        }
        if let Some(m) = req.max_tokens {
            gen_cfg["maxOutputTokens"] = json!(m);
        }
        if let ResponseFormat::JsonSchema { schema, .. } = &req.response_format {
            gen_cfg["responseMimeType"] = json!("application/json");
            gen_cfg["responseSchema"] = schema.clone();
        }

        let mut body = json!({
            "contents": [{"role": "user", "parts": [{"text": req.user}]}],
            "generationConfig": gen_cfg,
        });
        if let Some(system) = &req.system {
            body["systemInstruction"] = json!({"parts": [{"text": system}]});
        }

        tracing::debug!(target: "ailint_llm::google", model = %req.model, "gemini chat request");

        let resp = self
            .client
            .post(&url)
            .timeout(req.timeout)
            .json(&body)
            .send()
            .await
            .context("gemini request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!(target: "ailint_llm::google", %status, "gemini non-200 response");
            return Err(anyhow!("gemini returned {}: {}", status, text));
        }

        let body: Value = resp.json().await.context("gemini response not JSON")?;
        parse_response(body, &req.model)
    }
}

fn parse_response(body: Value, fallback_model: &str) -> Result<ChatResponse> {
    let candidate = body
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .ok_or_else(|| anyhow!("gemini response missing candidates[0]"))?;

    let text = candidate
        .get("content")
        .and_then(|c| c.get("parts"))
        .and_then(Value::as_array)
        .and_then(|parts| parts.first())
        .and_then(|p| p.get("text"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("gemini response missing candidates[0].content.parts[0].text"))?
        .to_string();

    let finish_reason = candidate
        .get("finishReason")
        .and_then(Value::as_str)
        .map(str::to_string);

    let usage = body
        .get("usageMetadata")
        .map(|u| Usage {
            prompt_tokens: u
                .get("promptTokenCount")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32,
            completion_tokens: u
                .get("candidatesTokenCount")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32,
        })
        .unwrap_or_default();

    let model = body
        .get("modelVersion")
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
