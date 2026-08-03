//! AIL104 `negative-constraint-overload` — negative constraints dominate affirmative guidance.
//!
//! See: `docs/rules/semantic/AIL104.md`

use serde::Deserialize;

use crate::file_type::FileType;
use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::semantic::AIL104;
use crate::rules::{dictionary_lines, Rule, RuleContext, RuleId, Severity, Violation};

const DEFAULT_PREFIXES: &str = include_str!("negation_prefixes.txt");
const DEFAULT_MIN_LIST_ITEMS: usize = 5;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    prefixes: Option<Vec<String>>,
    extra_prefixes: Option<Vec<String>>,
    min_list_items: Option<usize>,
}

/// AIL104 negative-constraint-overload: list dominated by "do not" constraints.
#[derive(Debug, Default)]
pub struct NegativeConstraintOverloadRule;

impl Rule for NegativeConstraintOverloadRule {
    fn id(&self) -> RuleId {
        AIL104
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "List is dominated by negative constraints; LLMs perform better with affirmative phrasing."
    }

    fn fix_hint(&self) -> &'static str {
        "Rewrite as positive directives (\"Do X\" over \"Don't Y\")."
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

        let min_list_items = opts.min_list_items.unwrap_or(DEFAULT_MIN_LIST_ITEMS);
        if md.list_items.len() < min_list_items {
            return Vec::new();
        }

        // Prefix match only: mid-sentence negations ("prefer X, not Y") are fine.
        let mut prefixes: Vec<String> = match opts.prefixes {
            Some(p) => p,
            None => dictionary_lines(DEFAULT_PREFIXES)
                .into_iter()
                .map(String::from)
                .collect(),
        };
        if let Some(extra) = opts.extra_prefixes {
            prefixes.extend(extra);
        }
        for p in &mut prefixes {
            *p = p.to_lowercase();
        }

        let mut negative_count = 0;
        for item in &md.list_items {
            let lower_text = item
                .text
                .trim_start_matches(['*', '_', '`'])
                .trim()
                .to_lowercase();
            if prefixes.iter().any(|p| lower_text.starts_with(p.as_str())) {
                negative_count += 1;
            }
        }

        if negative_count > md.list_items.len() / 2 {
            let v = Violation::new(
                AIL104,
                ctx.severity,
                doc.path.clone(),
                "list dominated by negative constraints",
            )
            .with_detail(format!(
                "{} of {} items are negative",
                negative_count,
                md.list_items.len()
            ));
            vec![v]
        } else {
            Vec::new()
        }
    }
}
