//! Structural rules: schema / frontmatter / required-section validation.
//! Range: **AIL001 – AIL099**.

pub mod broken_local_link;
pub mod empty_file;
pub mod frontmatter_schema;
pub mod malformed_yaml;
pub mod mcp_schema;
pub mod required_section;

pub use broken_local_link::BrokenLocalLinkRule;
pub use empty_file::EmptyFileRule;
pub use frontmatter_schema::FrontmatterSchemaRule;
pub use malformed_yaml::MalformedYamlRule;
pub use mcp_schema::McpSchemaValidationRule;
pub use required_section::MissingRequiredSectionRule;

use crate::rules::{Rule, RuleId};

/// AIL001: frontmatter fails the schema expected for its file type.
pub const AIL001: RuleId = RuleId::new(1, "no-frontmatter-schema-error");
/// AIL002: guidance file is empty or whitespace-only.
pub const AIL002: RuleId = RuleId::new(2, "instructions-file-empty");
/// AIL003: a section required for this file type is missing.
pub const AIL003: RuleId = RuleId::new(3, "missing-required-section");
/// AIL004: MCP server config file fails the minimum schema.
pub const AIL004: RuleId = RuleId::new(4, "mcp-schema-validation");
/// AIL040: relative link points at a file that does not exist.
pub const AIL040: RuleId = RuleId::new(40, "broken-local-link");
/// AIL041: YAML file (or frontmatter) fails to parse.
pub const AIL041: RuleId = RuleId::new(41, "malformed-yaml");

/// All structural rules, in registration order.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(EmptyFileRule),
        Box::new(FrontmatterSchemaRule),
        Box::new(MissingRequiredSectionRule),
        Box::new(BrokenLocalLinkRule),
        Box::new(MalformedYamlRule),
        Box::new(McpSchemaValidationRule),
    ]
}
