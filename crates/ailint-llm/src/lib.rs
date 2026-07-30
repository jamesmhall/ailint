//! Optional LLM-powered analysis for ailint. Rules AIL900–AIL999 live here.
//!
//! The crate is only pulled in when the CLI is invoked with `--llm-provider`
//! or a `.ailint.yaml` `llm:` block.

#![deny(rust_2018_idioms)]

pub mod analyzer;
pub mod provider;

pub use analyzer::{analyze, AIL900};
pub use provider::{
    AnthropicProvider, ChatRequest, ChatResponse, GoogleProvider, LlmProvider, OllamaProvider,
    OpenAiProvider, ProviderKind, ResponseFormat, Usage,
};
