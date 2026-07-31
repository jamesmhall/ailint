//! AIL001 `no-frontmatter-schema-error` — YAML frontmatter fails to parse.
//!
//! See: `docs/rules/structural/AIL001.md`

use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::structural::AIL001;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

/// AIL001 no-frontmatter-schema-error: frontmatter must match the file type's schema.
#[derive(Debug, Default)]
pub struct FrontmatterSchemaRule;

impl Rule for FrontmatterSchemaRule {
    fn id(&self) -> RuleId {
        AIL001
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn description(&self) -> &'static str {
        "YAML frontmatter fails to parse."
    }

    fn fix_hint(&self) -> &'static str {
        "Check indentation, quote strings containing colons, and align list items."
    }

    fn run(&self, doc: &ParsedDocument, _ctx: &RuleContext<'_>) -> Vec<Violation> {
        let DocumentContent::Markdown(md) = &doc.content else {
            return Vec::new();
        };
        let Some(fm) = md.frontmatter.as_ref() else {
            return Vec::new();
        };
        let Err(err) = serde_yaml::from_str::<serde_yaml::Value>(&fm.raw) else {
            return Vec::new();
        };
        let line = line_at_offset(&doc.raw, fm.byte_range.start);
        let v = Violation::new(
            AIL001,
            self.default_severity(),
            doc.path.clone(),
            "invalid YAML frontmatter",
        )
        .at(line, 1)
        .with_detail(err.to_string());
        vec![v]
    }
}

fn line_at_offset(raw: &str, offset: usize) -> usize {
    let end = offset.min(raw.len());
    raw.as_bytes()[..end]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}
