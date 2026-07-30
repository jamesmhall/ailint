//! AIL201 `no-unrestricted-tool-grant` — flag phrases that grant blanket
//! tool/permission access.
//!
//! See: `docs/rules/security/AIL201.md`

use regex::{Regex, RegexBuilder, RegexSetBuilder};
use serde::Deserialize;

use crate::parser::ParsedDocument;
use crate::rules::security::{line_of_offset, truncate_chars, AIL201};
use crate::rules::{dictionary_lines, Rule, RuleContext, RuleId, Severity, Violation};

const BUILTIN_PATTERNS: &str = include_str!("unrestricted_tool_patterns.txt");

#[derive(Debug, Default, Deserialize)]
struct Options {
    #[serde(default)]
    patterns: Option<Vec<String>>,
    #[serde(default)]
    extra_patterns: Option<Vec<String>>,
}

/// AIL201 no-unrestricted-tool-grant: flags blanket or auto-approved tool access.
#[derive(Debug, Default)]
pub struct NoUnrestrictedToolGrantRule;

impl Rule for NoUnrestrictedToolGrantRule {
    fn id(&self) -> RuleId {
        AIL201
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
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
                let matched = truncate_chars(m.as_str(), 60);
                let mut v = Violation::new(
                    AIL201,
                    self.default_severity(),
                    doc.path.clone(),
                    format!("unrestricted tool/permission grant: '{}'", matched),
                )
                .at(line, 1);
                v.fix_hint = Some(
                    "scope tool access explicitly (list allowed tools or commands) rather than granting blanket access"
                        .to_string(),
                );
                out.push(v);
            }
        }
        out
    }
}
