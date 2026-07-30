//! AIL202 `no-sensitive-data-in-instructions` — flag embedded secret-shaped
//! strings. Matched text is never included in the violation.
//!
//! See: `docs/rules/security/AIL202.md`

use std::sync::OnceLock;

use regex::{Regex, RegexSet};

use crate::parser::ParsedDocument;
use crate::rules::security::{line_of_offset, AIL202};
use crate::rules::{dictionary_lines, Rule, RuleContext, RuleId, Severity, Violation};

const SECRET_PATTERNS: &str = include_str!("secret_patterns.txt");

// Substrings within ±50 chars that mark a match as a documentation/fixture
// placeholder and should be ignored.
const ALLOWLIST_MARKERS: &str = include_str!("secret_allowlist_markers.txt");

struct Matchers {
    regexes: Vec<Regex>,
    // Single-pass prefilter; None only if the set fails to build.
    set: Option<RegexSet>,
}

fn matchers() -> &'static Matchers {
    static MATCHERS: OnceLock<Matchers> = OnceLock::new();
    MATCHERS.get_or_init(|| {
        let patterns: Vec<&str> = dictionary_lines(SECRET_PATTERNS)
            .into_iter()
            .filter(|p| Regex::new(p).is_ok())
            .collect();
        let regexes = patterns.iter().filter_map(|p| Regex::new(p).ok()).collect();
        Matchers {
            regexes,
            set: RegexSet::new(&patterns).ok(),
        }
    })
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

    fn run(&self, doc: &ParsedDocument, _ctx: &RuleContext<'_>) -> Vec<Violation> {
        let matchers = matchers();
        let indices: Vec<usize> = match &matchers.set {
            Some(s) => s.matches(&doc.raw).into_iter().collect(),
            None => (0..matchers.regexes.len()).collect(),
        };
        let mut out = Vec::new();
        for idx in indices {
            let re = &matchers.regexes[idx];
            for m in re.find_iter(&doc.raw) {
                if is_allowlisted(&doc.raw, m.start(), m.end()) || is_repeated_char(m.as_str()) {
                    continue;
                }
                let line = line_of_offset(&doc.raw, m.start());
                let mut v = Violation::new(
                    AIL202,
                    self.default_severity(),
                    doc.path.clone(),
                    "possible embedded secret / API key detected",
                )
                .at(line, 1);
                v.fix_hint = Some(
                    "remove the credential and reference an env var like AILINT_LLM_API_KEY instead"
                        .to_string(),
                );
                out.push(v);
            }
        }
        out
    }
}

fn is_allowlisted(raw: &str, start: usize, end: usize) -> bool {
    let ctx_start = start.saturating_sub(50);
    let ctx_end = (end + 50).min(raw.len());
    let ctx = &raw[ctx_start..ctx_end];
    dictionary_lines(ALLOWLIST_MARKERS)
        .iter()
        .any(|m| ctx.contains(m))
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
