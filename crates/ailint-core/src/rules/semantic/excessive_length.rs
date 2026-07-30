//! AIL102 `excessive-rule-length` — list items exceeding a word cap.
//!
//! See: `docs/rules/semantic/AIL102.md`

use serde::Deserialize;

use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::semantic::AIL102;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

const DEFAULT_MAX_WORDS: usize = 60;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    max_words: Option<usize>,
}

/// AIL102 excessive-rule-length: flags documents past the context-length budget.
#[derive(Debug, Default)]
pub struct ExcessiveRuleLengthRule;

impl Rule for ExcessiveRuleLengthRule {
    fn id(&self) -> RuleId {
        AIL102
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn run(&self, doc: &ParsedDocument, ctx: &RuleContext<'_>) -> Vec<Violation> {
        let md = match &doc.content {
            DocumentContent::Markdown(m) => m,
            _ => return Vec::new(),
        };
        let opts: Options = ctx
            .options
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
            .unwrap_or_default();
        let max_words = opts.max_words.unwrap_or(DEFAULT_MAX_WORDS);

        let mut out = Vec::new();
        for item in &md.list_items {
            let count = item.text.split_whitespace().count();
            if count <= max_words {
                continue;
            }
            let mut v = Violation::new(
                AIL102,
                self.default_severity(),
                doc.path.clone(),
                format!("rule exceeds {} words ({} words)", max_words, count),
            )
            .at(item.line, 1);
            v.fix_hint =
                Some("split into multiple smaller rules or move detail to a sub-list".into());
            out.push(v);
        }
        out
    }
}
