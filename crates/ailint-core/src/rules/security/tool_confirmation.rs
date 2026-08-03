//! AIL203 `tool-confirmation-required` — Ensure critical/destructive rules include "Wait for human confirmation" phrasing.
//!
//! See: `docs/rules/security/AIL203.md`

use serde::Deserialize;

use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::security::{line_of_offset, AIL203};
use crate::rules::{dictionary_lines, Rule, RuleContext, RuleId, Severity, Violation};

const DEFAULT_DESTRUCTIVE_PHRASES: &str = include_str!("destructive_phrases.txt");
const DEFAULT_CONFIRMATION_PHRASES: &str = include_str!("confirmation_phrases.txt");

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    destructive_phrases: Option<Vec<String>>,
    extra_destructive_phrases: Option<Vec<String>>,
    confirmation_phrases: Option<Vec<String>>,
    extra_confirmation_phrases: Option<Vec<String>>,
}

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

        let opts: Options = ctx
            .options
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
            .unwrap_or_default();

        let mut destructive_phrases: Vec<String> = match opts.destructive_phrases {
            Some(p) => p,
            None => dictionary_lines(DEFAULT_DESTRUCTIVE_PHRASES)
                .into_iter()
                .map(String::from)
                .collect(),
        };
        if let Some(extra) = opts.extra_destructive_phrases {
            destructive_phrases.extend(extra);
        }
        let mut confirmation_phrases: Vec<String> = match opts.confirmation_phrases {
            Some(p) => p,
            None => dictionary_lines(DEFAULT_CONFIRMATION_PHRASES)
                .into_iter()
                .map(String::from)
                .collect(),
        };
        if let Some(extra) = opts.extra_confirmation_phrases {
            confirmation_phrases.extend(extra);
        }
        for p in &mut destructive_phrases {
            *p = p.to_lowercase();
        }
        for p in &mut confirmation_phrases {
            *p = p.to_lowercase();
        }

        let lower_raw = doc.raw.to_lowercase();

        let mut violations = Vec::new();
        for d in &destructive_phrases {
            if let Some(idx) = lower_raw.find(d.as_str()) {
                if !confirmation_phrases
                    .iter()
                    .any(|c| lower_raw.contains(c.as_str()))
                {
                    let line = line_of_offset(&doc.raw, idx);
                    let v = Violation::new(
                        AIL203,
                        ctx.severity,
                        doc.path.clone(),
                        "destructive action without confirmation constraint",
                    )
                    .at(line, 1)
                    .with_detail(d.clone());
                    violations.push(v);

                    break;
                }
            }
        }

        violations
    }
}
