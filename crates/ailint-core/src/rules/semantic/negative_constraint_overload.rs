//! AIL104 `negative-constraint-overload` — negative constraints dominate affirmative guidance.
//!
//! See: `docs/rules/semantic/AIL104.md`

use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::semantic::AIL104;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

/// AIL104 negative-constraint-overload: list dominated by "do not" constraints.
#[derive(Debug, Default)]
pub struct NegativeConstraintOverloadRule;

impl Rule for NegativeConstraintOverloadRule {
    fn id(&self) -> RuleId {
        AIL104
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn run(&self, doc: &ParsedDocument, ctx: &RuleContext<'_>) -> Vec<Violation> {
        let md = match &doc.content {
            DocumentContent::Markdown(m) => m,
            _ => return Vec::new(),
        };

        if md.list_items.len() < 5 {
            return Vec::new();
        }

        // Prefix match only: mid-sentence negations ("prefer X, not Y") are fine.
        let negative_prefixes = [
            "do not", "don't", "never", "avoid", "stop ", "no ", "must not",
        ];

        let mut negative_count = 0;
        for item in &md.list_items {
            let lower_text = item
                .text
                .trim_start_matches(['*', '_', '`'])
                .trim()
                .to_lowercase();
            if negative_prefixes.iter().any(|p| lower_text.starts_with(p)) {
                negative_count += 1;
            }
        }

        if negative_count > md.list_items.len() / 2 {
            let mut v = Violation::new(
                AIL104,
                ctx.severity,
                doc.path.clone(),
                format!(
                    "Negative constraint overload: {} out of {} list items are negative constraints. LLMs perform better with affirmative instruction phrasing.",
                    negative_count,
                    md.list_items.len()
                ),
            );
            v.fix_hint = Some("Refactor constraints to state what the agent *should* do, rather than exhaustive lists of prohibitions.".to_string());
            vec![v]
        } else {
            Vec::new()
        }
    }
}
