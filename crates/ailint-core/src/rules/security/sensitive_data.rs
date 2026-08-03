//! AIL202 `no-sensitive-data-in-instructions` — flag embedded secret-shaped
//! strings. Matched text is never included in the violation.
//!
//! See: `docs/rules/security/AIL202.md`

use regex::{Regex, RegexSet};
use serde::Deserialize;

use crate::file_type::FileType;
use crate::parser::ParsedDocument;
use crate::rules::security::{line_of_offset, AIL202};
use crate::rules::{dictionary_lines, Rule, RuleContext, RuleId, Severity, Violation};

const DEFAULT_SECRET_PATTERNS: &str = include_str!("secret_patterns.txt");
// Substrings within ±50 chars that mark a match as a documentation/fixture
// placeholder and should be ignored.
const DEFAULT_ALLOWLIST_MARKERS: &str = include_str!("secret_allowlist_markers.txt");

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    patterns: Option<Vec<String>>,
    extra_patterns: Option<Vec<String>>,
    allowlist_markers: Option<Vec<String>>,
    extra_allowlist_markers: Option<Vec<String>>,
}

/// AIL202 no-sensitive-data-in-instructions: flags embedded secrets.
#[derive(Debug, Default)]
pub struct NoSensitiveDataInInstructionsRule;

impl Rule for NoSensitiveDataInInstructionsRule {
    fn id(&self) -> RuleId {
        AIL202
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn description(&self) -> &'static str {
        "Guidance file appears to contain an embedded secret or API key."
    }

    fn fix_hint(&self) -> &'static str {
        "Move the credential to an env var (e.g. AILINT_LLM_API_KEY) or a secret store."
    }

    fn applies_to(&self, file_type: FileType) -> bool {
        file_type.has_prose_content()
    }

    fn run(&self, doc: &ParsedDocument, ctx: &RuleContext<'_>) -> Vec<Violation> {
        let opts: Options = ctx
            .options
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
            .unwrap_or_default();

        let mut patterns: Vec<String> = match opts.patterns {
            Some(p) => p,
            None => dictionary_lines(DEFAULT_SECRET_PATTERNS)
                .into_iter()
                .map(String::from)
                .collect(),
        };
        if let Some(extra) = opts.extra_patterns {
            patterns.extend(extra);
        }
        let mut markers: Vec<String> = match opts.allowlist_markers {
            Some(m) => m,
            None => dictionary_lines(DEFAULT_ALLOWLIST_MARKERS)
                .into_iter()
                .map(String::from)
                .collect(),
        };
        if let Some(extra) = opts.extra_allowlist_markers {
            markers.extend(extra);
        }

        // Compile individually (silently skipping invalid user patterns), then
        // use a RegexSet as a single-pass prefilter over the document.
        let compiled: Vec<Regex> = patterns.iter().filter_map(|p| Regex::new(p).ok()).collect();
        let valid_patterns: Vec<&str> = patterns
            .iter()
            .filter(|p| Regex::new(p).is_ok())
            .map(String::as_str)
            .collect();
        let set = RegexSet::new(&valid_patterns).ok();
        let indices: Vec<usize> = match &set {
            Some(s) => s.matches(&doc.raw).into_iter().collect(),
            None => (0..compiled.len()).collect(),
        };

        let mut out = Vec::new();
        for idx in indices {
            let re = &compiled[idx];
            for m in re.find_iter(&doc.raw) {
                if is_allowlisted(&doc.raw, m.start(), m.end(), &markers)
                    || is_repeated_char(m.as_str())
                {
                    continue;
                }
                let line = line_of_offset(&doc.raw, m.start());
                let v = Violation::new(
                    AIL202,
                    ctx.severity,
                    doc.path.clone(),
                    "possible embedded secret",
                )
                .at(line, 1);
                out.push(v);
            }
        }
        out
    }
}

fn is_allowlisted(raw: &str, start: usize, end: usize, markers: &[String]) -> bool {
    let ctx_start = start.saturating_sub(50);
    let ctx_end = (end + 50).min(raw.len());
    let ctx = &raw[ctx_start..ctx_end];
    markers.iter().any(|m| ctx.contains(m.as_str()))
}

// Treat 5+ repeated chars in the match body as a placeholder (AAAAA..., 00000...).
fn is_repeated_char(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 5 {
        return false;
    }
    let mut run = 1usize;
    for w in chars.windows(2) {
        if w[0] == w[1] {
            run += 1;
            if run >= 5 {
                return true;
            }
        } else {
            run = 1;
        }
    }
    false
}
