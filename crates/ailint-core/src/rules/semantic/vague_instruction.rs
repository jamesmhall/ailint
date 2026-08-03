//! AIL100 `no-vague-instruction` — list items containing hand-wavy phrases.
//!
//! See: `docs/rules/semantic/AIL100.md`

use aho_corasick::AhoCorasick;
use serde::Deserialize;

use crate::file_type::FileType;
use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::semantic::AIL100;
use crate::rules::{dictionary_lines, Rule, RuleContext, RuleId, Severity, Violation};

const DEFAULT_PHRASES: &str = include_str!("vague_phrases.txt");

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    phrases: Option<Vec<String>>,
    extra_phrases: Option<Vec<String>>,
    case_sensitive: Option<bool>,
}

/// AIL100 no-vague-instruction: flags unactionable phrasing like "be careful".
#[derive(Debug, Default)]
pub struct NoVagueInstructionRule;

impl Rule for NoVagueInstructionRule {
    fn id(&self) -> RuleId {
        AIL100
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "List item contains vague, unactionable phrasing."
    }

    fn fix_hint(&self) -> &'static str {
        "Replace with a concrete verb and target the agent can act on."
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
        let case_sensitive = opts.case_sensitive.unwrap_or(false);

        let mut phrases: Vec<String> = match opts.phrases {
            Some(p) => p,
            None => dictionary_lines(DEFAULT_PHRASES)
                .into_iter()
                .map(String::from)
                .collect(),
        };
        if let Some(extra) = opts.extra_phrases {
            phrases.extend(extra);
        }
        if !case_sensitive {
            for p in &mut phrases {
                *p = p.to_lowercase();
            }
        }
        // Single-pass search over all phrases at once.
        let Ok(ac) = AhoCorasick::new(&phrases) else {
            return Vec::new();
        };

        let mut out = Vec::new();
        for item in &md.list_items {
            let haystack = if case_sensitive {
                item.text.clone()
            } else {
                item.text.to_lowercase()
            };
            if let Some(m) = ac.find(&haystack) {
                let phrase = &phrases[m.pattern().as_usize()];
                let snippet: String = item.text.chars().take(120).collect();
                let mut v = Violation::new(
                    AIL100,
                    self.default_severity(),
                    doc.path.clone(),
                    "vague phrase",
                )
                .at(item.line, 1)
                .with_detail(phrase.clone());
                v.snippet = Some(snippet);
                out.push(v);
            }
        }
        out
    }
}
