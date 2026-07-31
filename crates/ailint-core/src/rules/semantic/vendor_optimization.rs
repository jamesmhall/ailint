//! AIL105 `vendor-optimization-syntax` — Enforce XML `<conventions>` tags for Anthropic-hosted agents.
//!
//! See: `docs/rules/semantic/AIL105.md`

use crate::file_type::FileType;
use crate::parser::ParsedDocument;
use crate::rules::semantic::AIL105;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

/// AIL105 vendor-optimization-syntax: Claude/Cline files should use XML tags.
#[derive(Debug, Default)]
pub struct VendorOptimizationSyntaxRule;

impl Rule for VendorOptimizationSyntaxRule {
    fn id(&self) -> RuleId {
        AIL105
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Anthropic-hosted agents perform best with explicit XML tags (e.g. <conventions>)."
    }

    fn fix_hint(&self) -> &'static str {
        "Wrap sections in <conventions>...</conventions> or similar XML tags."
    }

    fn applies_to(&self, file_type: FileType) -> bool {
        matches!(file_type, FileType::ClaudeMd | FileType::ClineRules)
    }

    fn run(&self, doc: &ParsedDocument, ctx: &RuleContext<'_>) -> Vec<Violation> {
        if !doc.raw.contains("<conventions>")
            && !doc.raw.contains("<rules>")
            && !doc.raw.contains("</")
        {
            let v = Violation::new(AIL105, ctx.severity, doc.path.clone(), "no XML tags found");
            vec![v]
        } else {
            Vec::new()
        }
    }
}
