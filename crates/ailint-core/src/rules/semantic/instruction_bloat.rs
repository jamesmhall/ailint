//! AIL106 `detect-instruction-bloat` — flags monolithic prose paragraphs.
//!
//! See: `docs/rules/semantic/AIL106.md`

use serde::Deserialize;

use crate::file_type::FileType;
use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::semantic::AIL106;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

const DEFAULT_MAX_WORDS: usize = 120;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    max_words: Option<usize>,
}

/// AIL106 detect-instruction-bloat: flags oversized prose paragraphs.
#[derive(Debug, Default)]
pub struct DetectInstructionBloatRule;

impl Rule for DetectInstructionBloatRule {
    fn id(&self) -> RuleId {
        AIL106
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Prose paragraph is longer than the configured word budget."
    }

    fn fix_hint(&self) -> &'static str {
        "Break the paragraph into shorter statements or a bulleted list."
    }

    fn applies_to(&self, file_type: FileType) -> bool {
        file_type.has_prose_content()
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
        for p in &md.paragraphs {
            let count = p.text.split_whitespace().count();
            if count <= max_words {
                continue;
            }
            let v = Violation::new(
                AIL106,
                self.default_severity(),
                doc.path.clone(),
                format!("paragraph exceeds {} words", max_words),
            )
            .at(p.line, 1)
            .with_detail(format!("{} words", count));
            out.push(v);
        }
        out
    }
}
