//! LLM-driven analyzer that produces `Violation`s for the AIL9xx rules.
//!
//! See: `docs/rules/llm/AIL900.md`

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

use ailint_core::parser::ParsedDocument;
use ailint_core::rules::{RuleId, Severity, Violation};

use crate::provider::{ChatRequest, LlmProvider, ResponseFormat};

/// Rule ID for the general LLM quality-score check.
pub const AIL900: RuleId = RuleId::new(900, "llm-quality-score");
/// Rule ID for the LLM-graded actionability check.
pub const AIL901: RuleId = RuleId::new(901, "llm-actionability-check");

const MAX_USER_CHARS: usize = 8000;
const SYSTEM_PROMPT: &str = include_str!("../prompts/quality_system.md");
const ACTIONABILITY_PROMPT: &str = include_str!("../prompts/actionability_system.md");

#[derive(Deserialize, Debug)]
struct LlmResponse {
    #[serde(default)]
    issues: Vec<LlmIssue>,
}

#[derive(Deserialize, Debug)]
struct LlmIssue {
    severity: String,
    #[serde(default)]
    line: Option<usize>,
    message: String,
    #[serde(default)]
    fix_hint: Option<String>,
}

/// Run LLM analysis for a single document and return zero or more violations.
///
/// The caller is responsible for supplying a configured provider and the model
/// name to use; degradation when no provider is available happens above this
/// layer (in the CLI shim).
pub async fn analyze(
    provider: &dyn LlmProvider,
    model: &str,
    doc: &ParsedDocument,
) -> Result<Vec<Violation>> {
    let req = ChatRequest {
        model: model.to_string(),
        system: Some(SYSTEM_PROMPT.to_string()),
        user: build_user_prompt(doc),
        response_format: ResponseFormat::JsonSchema {
            schema: response_schema(),
            name: "ailint_quality".into(),
        },
        ..Default::default()
    };

    let resp = provider.chat(&req).await?;
    let parsed: LlmResponse = serde_json::from_str(&resp.text)
        .with_context(|| format!("LLM response was not valid JSON: {}", resp.text))?;

    let mut out = Vec::with_capacity(parsed.issues.len());
    for issue in parsed.issues {
        let severity = match issue.severity.as_str() {
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            _ => Severity::Info,
        };
        let mut v = Violation::new(AIL900, severity, doc.path.clone(), issue.message);
        if let Some(line) = issue.line {
            v = v.at(line, 1);
        }
        v.fix_hint = issue.fix_hint;
        out.push(v);
    }
    Ok(out)
}

/// Run the LLM-graded actionability check (AIL901) for a single document.
///
/// Emits one violation per non-actionable directive the model identifies.
/// All findings are `Warning` severity regardless of what the LLM returns.
pub async fn analyze_actionability(
    provider: &dyn LlmProvider,
    model: &str,
    doc: &ParsedDocument,
) -> Result<Vec<Violation>> {
    let req = ChatRequest {
        model: model.to_string(),
        system: Some(ACTIONABILITY_PROMPT.to_string()),
        user: build_user_prompt(doc),
        response_format: ResponseFormat::JsonSchema {
            schema: response_schema(),
            name: "ailint_actionability".into(),
        },
        ..Default::default()
    };

    let resp = provider.chat(&req).await?;
    let parsed: LlmResponse = serde_json::from_str(&resp.text)
        .with_context(|| format!("LLM response was not valid JSON: {}", resp.text))?;

    let mut out = Vec::with_capacity(parsed.issues.len());
    for issue in parsed.issues {
        let mut v = Violation::new(AIL901, Severity::Warning, doc.path.clone(), issue.message);
        if let Some(line) = issue.line {
            v = v.at(line, 1);
        }
        v.fix_hint = issue.fix_hint;
        out.push(v);
    }
    Ok(out)
}

fn build_user_prompt(doc: &ParsedDocument) -> String {
    let body: String = doc.raw.chars().take(MAX_USER_CHARS).collect();
    let truncated = doc.raw.chars().count() > MAX_USER_CHARS;
    let mut prompt = format!(
        "File path: {}\nFile type: {}\n\n--- BEGIN CONTENT ---\n{}\n--- END CONTENT ---",
        doc.path.display(),
        doc.file_type.as_str(),
        body,
    );
    if truncated {
        prompt.push_str("\n\n(Content truncated to first 8000 characters.)");
    }
    prompt
}

fn response_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "issues": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "severity": {"type": "string", "enum": ["error", "warning", "info"]},
                        "line": {"type": "integer", "minimum": 1},
                        "message": {"type": "string"},
                        "fix_hint": {"type": "string"}
                    },
                    "required": ["severity", "message"]
                }
            }
        },
        "required": ["issues"]
    })
}
