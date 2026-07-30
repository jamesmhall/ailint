//! AIL103 `no-duplicate-rules` — list items that normalize to identical text.
//!
//! See: `docs/rules/semantic/AIL103.md`

use std::collections::HashMap;

use serde::Deserialize;

use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::semantic::AIL103;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    normalize: Option<bool>,
}

/// AIL103 no-duplicate-rules: flags a rule stated more than once in a document.
#[derive(Debug, Default)]
pub struct NoDuplicateRulesRule;

impl Rule for NoDuplicateRulesRule {
    fn id(&self) -> RuleId {
        AIL103
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
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
        let _normalize = opts.normalize.unwrap_or(true);

        let mut first_seen: HashMap<String, usize> = HashMap::new();
        let mut out = Vec::new();
        for item in &md.list_items {
            let norm = normalize(&item.text);
            if norm.chars().count() < 5 {
                continue;
            }
            match first_seen.get(&norm) {
                Some(&first_line) => {
                    let snippet: String = item.text.chars().take(120).collect();
                    let mut v = Violation::new(
                        AIL103,
                        self.default_severity(),
                        doc.path.clone(),
                        format!("duplicate of rule at line {}", first_line),
                    )
                    .at(item.line, 1);
                    v.snippet = Some(snippet);
                    out.push(v);
                }
                None => {
                    first_seen.insert(norm, item.line);
                }
            }
        }
        out
    }
}

fn normalize(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut buf = String::with_capacity(lower.len());
    let mut prev_space = true;
    for ch in lower.chars() {
        if ch.is_alphanumeric() {
            buf.push(ch);
            prev_space = false;
        } else if (ch.is_whitespace() || ch.is_ascii_punctuation()) && !prev_space {
            buf.push(' ');
            prev_space = true;
        }
    }
    buf.trim().to_string()
}
