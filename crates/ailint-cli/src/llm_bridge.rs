//! Bridge between the core lint pipeline and the optional LLM analyzer.

use std::path::Path;

use ailint_core::config::{Config, LlmConfig, LlmProviderKind};
use ailint_core::discovery;
use ailint_core::parser;
use ailint_core::rules::Violation;
use ailint_llm::{AnthropicProvider, GoogleProvider, LlmProvider, OllamaProvider, OpenAiProvider};
use anyhow::Result;

/// Return `None` if provider construction failed non-fatally (missing env).
/// `Err` is reserved for a truly unknown provider variant.
fn build_provider(cfg: &LlmConfig) -> Result<Option<Box<dyn LlmProvider>>> {
    let provider: Box<dyn LlmProvider> = match cfg.provider {
        LlmProviderKind::Openai | LlmProviderKind::Compatible => {
            let mut p = match OpenAiProvider::from_env() {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("openai provider unavailable: {e:#}");
                    return Ok(None);
                }
            };
            if let Some(url) = cfg.base_url.as_deref() {
                p = p.with_base_url(url.to_string());
            }
            Box::new(p)
        }
        LlmProviderKind::Anthropic => {
            let mut p = match AnthropicProvider::from_env() {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("anthropic provider unavailable: {e:#}");
                    return Ok(None);
                }
            };
            if let Some(url) = cfg.base_url.as_deref() {
                p = p.with_base_url(url.to_string());
            }
            Box::new(p)
        }
        LlmProviderKind::Google => {
            let mut p = match GoogleProvider::from_env() {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("google provider unavailable: {e:#}");
                    return Ok(None);
                }
            };
            if let Some(url) = cfg.base_url.as_deref() {
                p = p.with_base_url(url.to_string());
            }
            Box::new(p)
        }
        LlmProviderKind::Ollama => {
            let mut p = OllamaProvider::new();
            if let Some(url) = cfg.base_url.as_deref() {
                p = p.with_base_url(url.to_string());
            }
            Box::new(p)
        }
    };
    Ok(Some(provider))
}

/// Run the LLM analyzer against every file discovered under `root`. Returns
/// the aggregated AIL9xx violations. No-op when no provider is configured or
/// when the provider fails to initialize.
pub fn run(config: &Config, root: &Path) -> Vec<Violation> {
    let Some(llm_cfg) = config.llm.as_ref() else {
        return Vec::new();
    };
    let provider = match build_provider(llm_cfg) {
        Ok(Some(p)) => p,
        Ok(None) => return Vec::new(),
        Err(e) => {
            tracing::warn!("llm provider config invalid: {e:#}");
            return Vec::new();
        }
    };

    let files = match discovery::walk(root, config) {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("llm discovery failed for {}: {e:#}", root.display());
            return Vec::new();
        }
    };
    let mut docs = Vec::with_capacity(files.len());
    for file in files {
        match parser::parse(&file.path, file.file_type) {
            Ok(doc) => docs.push(doc),
            Err(e) => tracing::warn!("llm parse failed for {}: {e:#}", file.path.display()),
        }
    }

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::error!("failed to build tokio runtime for llm analyzer: {e}");
            return Vec::new();
        }
    };

    let mut out = Vec::new();
    for doc in &docs {
        match rt.block_on(ailint_llm::analyze(provider.as_ref(), &llm_cfg.model, doc)) {
            Ok(mut v) => out.append(&mut v),
            Err(e) => {
                tracing::warn!("AIL900 analyzer failed on {}: {e:#}", doc.path.display());
            }
        }
    }
    out
}
