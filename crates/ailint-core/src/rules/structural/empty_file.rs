//! AIL002 `instructions-file-empty` — file is empty or whitespace only.
//!
//! See: `docs/rules/structural/AIL002.md`

use crate::parser::ParsedDocument;
use crate::rules::structural::AIL002;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

/// AIL002 instructions-file-empty: guidance file has no content.
#[derive(Debug, Default)]
pub struct EmptyFileRule;

impl Rule for EmptyFileRule {
    fn id(&self) -> RuleId {
        AIL002
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn run(&self, doc: &ParsedDocument, _ctx: &RuleContext<'_>) -> Vec<Violation> {
        if !doc.raw.trim().is_empty() {
            return Vec::new();
        }
        vec![Violation::new(
            AIL002,
            self.default_severity(),
            doc.path.clone(),
            "file is empty or contains only whitespace",
        )
        .at(1, 1)]
    }
}
