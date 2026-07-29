//! # ailint-llm
//!
//! Optional LLM-powered analysis for ailint. Rules AIL900–AIL999 live here.
//! The crate is only pulled in when the CLI is invoked with `--llm-provider`
//! or a `.ailint.yaml` `llm:` block.
//!
//! Everything here is a scaffold — see `TODO` markers.

#![deny(rust_2018_idioms)]

pub mod analyzer;
pub mod provider;

pub use analyzer::analyze;
pub use provider::{LlmProvider, ProviderKind};
