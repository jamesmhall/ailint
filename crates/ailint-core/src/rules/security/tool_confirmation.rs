//! AIL203 `tool-confirmation-required` — Ensure critical/destructive rules include "Wait for human confirmation" phrasing.
//!
//! See: `docs/rules/security/AIL203.md`

use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::security::{line_of_offset, AIL203};
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

/// AIL203 tool-confirmation-required: destructive actions need a confirmation step.
#[derive(Debug, Default)]
pub struct ToolConfirmationRequiredRule;

impl Rule for ToolConfirmationRequiredRule {
    fn id(&self) -> RuleId {
        AIL203
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn description(&self) -> &'static str {
        "File describes destructive actions but has no confirmation constraint."
    }

    fn fix_hint(&self) -> &'static str {
        "Require explicit confirmation (\"wait for human confirmation\") before destructive actions."
    }

    fn run(&self, doc: &ParsedDocument, ctx: &RuleContext<'_>) -> Vec<Violation> {
        match &doc.content {
            DocumentContent::Markdown(_) | DocumentContent::Text => {}
            _ => return Vec::new(),
        };

        let destructive_phrases = [
            "delete data",
            "rm -rf",
            "drop table",
            "truncate table",
            "irreversible",
            "destructive action",
        ];
        let confirmation_phrases = [
            "ask",
            "confirm",
            "human",
            "wait",
            "permission",
            "consent",
            "approval",
        ];

        let lower_raw = doc.raw.to_lowercase();

        let mut violations = Vec::new();
        for d in &destructive_phrases {
            if let Some(idx) = lower_raw.find(d) {
                if !confirmation_phrases.iter().any(|c| lower_raw.contains(c)) {
                    let line = line_of_offset(&doc.raw, idx);
                    let v = Violation::new(
                        AIL203,
                        ctx.severity,
                        doc.path.clone(),
                        "destructive action without confirmation constraint",
                    )
                    .at(line, 1)
                    .with_detail((*d).to_string());
                    violations.push(v);

                    break;
                }
            }
        }

        violations
    }
}
