//! AIL003 `missing-required-section` — required top-level heading is missing.
//!
//! See: `docs/rules/structural/AIL003.md`

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::file_type::FileType;
use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::structural::AIL003;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

/// AIL003 missing-required-section: file type mandates a section that is absent.
#[derive(Debug, Default)]
pub struct MissingRequiredSectionRule;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RuleOptions {
    required: Vec<String>,
    per_file_type: BTreeMap<String, Vec<String>>,
}

impl Rule for MissingRequiredSectionRule {
    fn id(&self) -> RuleId {
        AIL003
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Required top-level heading is not present."
    }

    fn fix_hint(&self) -> &'static str {
        "Add a top-level heading that matches the required section name."
    }

    fn run(&self, doc: &ParsedDocument, ctx: &RuleContext<'_>) -> Vec<Violation> {
        let DocumentContent::Markdown(md) = &doc.content else {
            return Vec::new();
        };
        let opts: RuleOptions = match ctx.options {
            Some(v) => match serde_yaml::from_value(v.clone()) {
                Ok(c) => c,
                Err(_) => return Vec::new(),
            },
            None => RuleOptions::default(),
        };
        let key = file_type_key(doc.file_type);
        let required = opts
            .per_file_type
            .get(key)
            .cloned()
            .unwrap_or(opts.required);
        if required.is_empty() {
            return Vec::new();
        }
        let top_headings: Vec<String> = md
            .headings
            .iter()
            .filter(|h| h.level <= 2)
            .map(|h| h.text.to_ascii_lowercase())
            .collect();
        let mut out = Vec::new();
        for entry in required {
            let needle = entry.to_ascii_lowercase();
            if top_headings.iter().any(|h| h.contains(&needle)) {
                continue;
            }
            let v = Violation::new(
                AIL003,
                self.default_severity(),
                doc.path.clone(),
                "missing required section",
            )
            .with_detail(entry);
            out.push(v);
        }
        out
    }
}

fn file_type_key(ft: FileType) -> &'static str {
    match ft {
        FileType::ClaudeMd => "claudemd",
        FileType::AgentsMd => "agentsmd",
        FileType::CopilotCustomization => "copilotcustomization",
        FileType::CopilotInstructions => "copilotinstructions",
        FileType::CursorRules => "cursorrules",
        FileType::WindsurfRules => "windsurfrules",
        FileType::ClineRules => "clinerules",
        FileType::JunieGuidelines => "junieguidelines",
        FileType::GenericSystemPrompt => "genericsystemprompt",
        FileType::AiderConventions => "aiderconventions",
        FileType::ContinueRules => "continuerules",
        FileType::GitHubSkill => "githubskill",
        FileType::CustomProjectRules => "customprojectrules",
        FileType::GenericMarkdown => "genericmarkdown",
        FileType::GenericYaml => "genericyaml",
        FileType::Unknown => "unknown",
    }
}
