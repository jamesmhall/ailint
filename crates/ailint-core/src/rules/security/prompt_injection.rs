//! AIL200 `no-prompt-injection-marker` — flag known prompt-injection sentinels.
//!
//! See: `docs/rules/security/AIL200.md`

use regex::{Regex, RegexBuilder, RegexSetBuilder};
use serde::Deserialize;

use crate::parser::ParsedDocument;
use crate::rules::security::{line_containing, line_of_offset, truncate_chars, AIL200};
use crate::rules::{dictionary_lines, Rule, RuleContext, RuleId, Severity, Violation};

const BUILTIN_PATTERNS: &str = include_str!("prompt_injection_patterns.txt");

#[derive(Debug, Default, Deserialize)]
struct Options {
    #[serde(default)]
    patterns: Option<Vec<String>>,
    #[serde(default)]
    extra_patterns: Option<Vec<String>>,
}

/// AIL200 no-prompt-injection-marker: flags known injection markers in text.
#[derive(Debug, Default)]
pub struct NoPromptInjectionMarkerRule;

impl Rule for NoPromptInjectionMarkerRule {
    fn id(&self) -> RuleId {
        AIL200
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn description(&self) -> &'static str {
        "Guidance file contains a phrase commonly used to override system prompts."
    }

    fn fix_hint(&self) -> &'static str {
        "Remove the marker; injected content could exploit it."
    }

    fn run(&self, doc: &ParsedDocument, ctx: &RuleContext<'_>) -> Vec<Violation> {
        let opts: Options = ctx
            .options
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
            .unwrap_or_default();

        let base: Vec<String> = match opts.patterns {
            Some(p) => p,
            None => dictionary_lines(BUILTIN_PATTERNS)
                .into_iter()
                .map(String::from)
                .collect(),
        };
        let extras = opts.extra_patterns.unwrap_or_default();

        // Compile individually (silently skipping invalid user patterns), then
        // use a RegexSet as a single-pass prefilter over the document.
        let compiled: Vec<(&String, Regex)> = base
            .iter()
            .chain(extras.iter())
            .filter_map(|p| {
                RegexBuilder::new(p)
                    .case_insensitive(true)
                    .build()
                    .ok()
                    .map(|re| (p, re))
            })
            .collect();
        let set = RegexSetBuilder::new(compiled.iter().map(|(p, _)| p.as_str()))
            .case_insensitive(true)
            .build()
            .ok();
        let matched: Vec<usize> = match &set {
            Some(s) => s.matches(&doc.raw).into_iter().collect(),
            None => (0..compiled.len()).collect(),
        };

        let mut out = Vec::new();
        for idx in matched {
            let (_, re) = &compiled[idx];
            for m in re.find_iter(&doc.raw) {
                let line = line_of_offset(&doc.raw, m.start());
                let matched = truncate_chars(m.as_str().trim_end(), 60);
                let mut v = Violation::new(
                    AIL200,
                    self.default_severity(),
                    doc.path.clone(),
                    "prompt-injection marker",
                )
                .at(line, 1)
                .with_detail(matched);
                v.snippet = Some(line_containing(&doc.raw, m.start()));
                out.push(v);
            }
        }
        out
    }
}
