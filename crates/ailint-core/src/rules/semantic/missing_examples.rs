//! AIL101 `no-missing-examples` — required sections lacking concrete examples.
//!
//! See: `docs/rules/semantic/AIL101.md`

use serde::Deserialize;

use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::semantic::AIL101;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

const DEFAULT_REQUIRED_IN: &[&str] = &["Examples", "Usage", "Example"];
const DEFAULT_MIN_WORDS: usize = 20;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    required_in: Option<Vec<String>>,
    min_words: Option<usize>,
}

/// AIL101 no-missing-examples: behavioral rules should show concrete examples.
#[derive(Debug, Default)]
pub struct NoMissingExamplesRule;

impl Rule for NoMissingExamplesRule {
    fn id(&self) -> RuleId {
        AIL101
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Behavioral section is missing a concrete example."
    }

    fn fix_hint(&self) -> &'static str {
        "Add a fenced code block or an `e.g.` clause with a concrete example."
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
        let required_in: Vec<String> = opts
            .required_in
            .unwrap_or_else(|| {
                DEFAULT_REQUIRED_IN
                    .iter()
                    .map(|s| s.to_lowercase())
                    .collect()
            })
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        let min_words = opts.min_words.unwrap_or(DEFAULT_MIN_WORDS);

        let mut out = Vec::new();
        for section in &md.sections {
            let heading_idx = match section.heading_index {
                Some(i) => i,
                None => continue,
            };
            let heading = match md.headings.get(heading_idx) {
                Some(h) => h,
                None => continue,
            };
            let heading_lc = heading.text.to_lowercase();
            if !required_in.iter().any(|r| heading_lc.contains(r)) {
                continue;
            }
            let body = doc.raw.get(section.byte_range.clone()).unwrap_or("");
            if body.split_whitespace().count() < min_words {
                continue;
            }
            let has_code = md.code_blocks.iter().any(|cb| {
                cb.byte_range.start >= section.byte_range.start
                    && cb.byte_range.end <= section.byte_range.end
            });
            let body_lc = body.to_lowercase();
            let has_eg = body_lc.contains("e.g.") || body_lc.contains("example:");
            if has_code || has_eg {
                continue;
            }
            let v = Violation::new(
                AIL101,
                self.default_severity(),
                doc.path.clone(),
                "section lacks concrete examples",
            )
            .at(heading.line, 1)
            .with_detail(heading.text.clone());
            out.push(v);
        }
        out
    }
}
