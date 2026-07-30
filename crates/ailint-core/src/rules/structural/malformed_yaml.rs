//! AIL041 `malformed-yaml` — a `.yaml` / `.yml` file failed to parse.
//!
//! See: `docs/rules/structural/AIL041.md`

use crate::file_type::FileType;
use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::structural::AIL041;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

/// AIL041 malformed-yaml: YAML file or frontmatter fails to parse.
#[derive(Debug, Default)]
pub struct MalformedYamlRule;

impl Rule for MalformedYamlRule {
    fn id(&self) -> RuleId {
        AIL041
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    /// Applies to any file we treat as YAML, guidance or generic.
    fn applies_to(&self, file_type: FileType) -> bool {
        matches!(
            file_type,
            FileType::GenericYaml
                | FileType::GenericSystemPrompt
                | FileType::CursorRules
                | FileType::WindsurfRules
                | FileType::ClineRules
                | FileType::ContinueRules
        )
    }

    fn run(&self, doc: &ParsedDocument, _ctx: &RuleContext<'_>) -> Vec<Violation> {
        let msg = match &doc.content {
            DocumentContent::ParseError(m) => m,
            _ => return Vec::new(),
        };
        let (line, column) = extract_location(msg).unwrap_or((1, 1));
        let mut v = Violation::new(
            AIL041,
            self.default_severity(),
            doc.path.clone(),
            format!("YAML/JSON parse error: {msg}"),
        )
        .at(line, column);
        v.fix_hint = Some("fix the YAML/JSON syntax error at the reported location".to_string());
        vec![v]
    }
}

/// Best-effort extraction of `line: N column: M` from serde_yaml errors.
fn extract_location(msg: &str) -> Option<(usize, usize)> {
    let lower = msg.to_ascii_lowercase();
    let line = extract_after(&lower, "line ").or_else(|| extract_after(&lower, "line: "))?;
    let col = extract_after(&lower, "column ")
        .or_else(|| extract_after(&lower, "column: "))
        .unwrap_or(1);
    Some((line, col))
}

fn extract_after(haystack: &str, needle: &str) -> Option<usize> {
    let idx = haystack.find(needle)?;
    let rest = &haystack[idx + needle.len()..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_serde_yaml_style_location() {
        assert_eq!(
            extract_location("mapping values are not allowed in this context at line 3 column 5"),
            Some((3, 5))
        );
    }

    #[test]
    fn falls_back_when_no_location() {
        assert_eq!(extract_location("some other error"), None);
    }
}
