//! AIL300 `no-conflicting-rules` — negation-pair detection across files.
//!
//! See: `docs/rules/consistency/AIL300.md`

use std::collections::HashMap;

use aho_corasick::{AhoCorasick, Anchored, Input, MatchKind, StartKind};
use serde::Deserialize;
use unicode_segmentation::UnicodeSegmentation;

use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::consistency::AIL300;
use crate::rules::{dictionary_lines, BatchRule, RuleContext, RuleId, Severity, Violation};

const NEGATION_PREFIXES: &str = include_str!("negation_prefixes.txt");

const DEFAULT_MIN_CORE_WORDS: usize = 3;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    negation_prefixes: Option<Vec<String>>,
    min_core_words: Option<usize>,
}

/// AIL300 no-conflicting-rules: flags contradictory instructions across files.
#[derive(Debug, Default)]
pub struct NoConflictingRulesRule;

impl BatchRule for NoConflictingRulesRule {
    fn id(&self) -> RuleId {
        AIL300
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn run_batch(&self, docs: &[ParsedDocument], ctx: &RuleContext<'_>) -> Vec<Violation> {
        let opts: Options = ctx
            .options
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
            .unwrap_or_default();
        let prefixes: Vec<String> = opts
            .negation_prefixes
            .unwrap_or_else(|| {
                dictionary_lines(NEGATION_PREFIXES)
                    .into_iter()
                    .map(String::from)
                    .collect()
            })
            .into_iter()
            .map(|p| normalize(&p))
            .filter(|p| !p.is_empty())
            .collect();
        let Some(matcher) = prefix_matcher(&prefixes) else {
            return Vec::new();
        };
        let min_core_words = opts.min_core_words.unwrap_or(DEFAULT_MIN_CORE_WORDS);

        // (doc_index, item_index_in_doc)
        #[derive(Debug, Clone)]
        struct Entry {
            doc_idx: usize,
            item_idx: usize,
            negated: bool,
        }

        let mut groups: HashMap<String, Vec<Entry>> = HashMap::new();
        for (doc_idx, doc) in docs.iter().enumerate() {
            let md = match &doc.content {
                DocumentContent::Markdown(m) => m,
                _ => continue,
            };
            for (item_idx, item) in md.list_items.iter().enumerate() {
                let base = normalize(&item.text);
                if base.is_empty() {
                    continue;
                }
                let (core, negated) = strip_negation(&base, &matcher);
                if core.split_whitespace().count() < min_core_words {
                    continue;
                }
                groups.entry(core.to_string()).or_default().push(Entry {
                    doc_idx,
                    item_idx,
                    negated,
                });
            }
        }

        let mut out = Vec::new();
        for entries in groups.values() {
            // Need at least two files represented and both polarities.
            let mut has_pos = None::<&Entry>;
            let mut has_neg = None::<&Entry>;
            for e in entries {
                if e.negated {
                    if has_neg.is_none() {
                        has_neg = Some(e);
                    }
                } else if has_pos.is_none() {
                    has_pos = Some(e);
                }
            }
            let (Some(pos), Some(neg)) = (has_pos, has_neg) else {
                continue;
            };
            if docs[pos.doc_idx].path == docs[neg.doc_idx].path {
                continue;
            }
            for e in entries {
                let counter = if e.negated { pos } else { neg };
                if docs[e.doc_idx].path == docs[counter.doc_idx].path {
                    continue;
                }
                let doc = &docs[e.doc_idx];
                let other = &docs[counter.doc_idx];
                let md = match &doc.content {
                    DocumentContent::Markdown(m) => m,
                    _ => continue,
                };
                let other_md = match &other.content {
                    DocumentContent::Markdown(m) => m,
                    _ => continue,
                };
                let item = &md.list_items[e.item_idx];
                let other_item = &other_md.list_items[counter.item_idx];
                let other_text = truncate_chars(other_item.text.trim(), 80);
                let other_name = other
                    .path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("<unknown>");
                let mut v = Violation::new(
                    AIL300,
                    self.default_severity(),
                    doc.path.clone(),
                    format!("conflicts with '{}' in {}", other_text, other_name),
                )
                .at(item.line, 1);
                v.snippet = Some(truncate_chars(&item.text, 120));
                v.fix_hint = Some(
                    "reconcile the two files so they don't give contradictory instructions".into(),
                );
                out.push(v);
            }
        }
        out
    }
}

// Lowercase and collapse to space-joined words (UAX#29 segmentation keeps
// in-word apostrophes like "don't" intact).
fn normalize(s: &str) -> String {
    s.unicode_words()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

// Anchored leftmost-longest automaton over the negation prefixes.
fn prefix_matcher(prefixes: &[String]) -> Option<AhoCorasick> {
    AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostLongest)
        .start_kind(StartKind::Anchored)
        .build(prefixes)
        .ok()
}

// If `s` starts with a negation prefix (as a whole word/phrase), return
// (core, true). Otherwise return (s, false).
fn strip_negation<'a>(s: &'a str, matcher: &AhoCorasick) -> (&'a str, bool) {
    if let Some(m) = matcher.find(Input::new(s).anchored(Anchored::Yes)) {
        let rest = &s[m.end()..];
        if rest.is_empty() {
            return ("", true);
        }
        if rest.starts_with(' ') {
            return (rest.trim_start(), true);
        }
    }
    (s, false)
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_matcher() -> AhoCorasick {
        let prefixes: Vec<String> = dictionary_lines(NEGATION_PREFIXES)
            .into_iter()
            .map(normalize)
            .collect();
        prefix_matcher(&prefixes).expect("default prefixes must compile")
    }

    #[test]
    fn normalize_strips_punct_and_lowercases() {
        assert_eq!(normalize("Use TABS, please!"), "use tabs please");
    }

    #[test]
    fn normalize_keeps_inword_apostrophe() {
        assert_eq!(normalize("Don't panic"), "don't panic");
    }

    #[test]
    fn strip_negation_detects_dont() {
        let (core, neg) = strip_negation("don't use tabs for indentation", &default_matcher());
        assert!(neg);
        assert_eq!(core, "use tabs for indentation");
    }

    #[test]
    fn strip_negation_leaves_positive_alone() {
        let (core, neg) = strip_negation("use tabs for indentation", &default_matcher());
        assert!(!neg);
        assert_eq!(core, "use tabs for indentation");
    }

    #[test]
    fn strip_negation_requires_word_boundary() {
        // "no" must not strip inside "notice".
        let (core, neg) = strip_negation("notice the pattern here", &default_matcher());
        assert!(!neg);
        assert_eq!(core, "notice the pattern here");
    }
}
