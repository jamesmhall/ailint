//! LLM-driven analyzer that produces `Violation`s for the AIL9xx rules.

use anyhow::Result;

use ailint_core::parser::ParsedDocument;
use ailint_core::rules::{RuleId, Severity, Violation};

use crate::provider::LlmProvider;

/// Rule ID for the general LLM quality-score check.
pub const AIL900: RuleId = RuleId::new(900, "llm-quality-score");

/// Run LLM analysis for a single document and return zero or more violations.
///
/// TODO:
/// - Build a prompt from `doc.raw` with an instruction-quality rubric.
/// - Call `provider.chat()` (async once we swap in `async-trait`).
/// - Parse a structured response (JSON schema) into `Violation`s.
/// - Respect cost / rate-limit config.
pub fn analyze(_provider: &dyn LlmProvider, _doc: &ParsedDocument) -> Result<Vec<Violation>> {
    // Reference the rule ID so it isn't flagged as unused during the scaffold
    // phase. TODO: replace with real logic.
    let _ = (AIL900, Severity::Info);
    Ok(Vec::new())
}
